//! Daemon manager - core process control and lifecycle management.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::process::Command;
use tokio::sync::{mpsc, Notify, RwLock};
use tracing::{debug, error, info, warn};

use crate::daemon::config::DaemonConfig;
use crate::daemon::ipc::{DaemonResponse, ProcessStatus, SystemStatus};
use crate::daemon::process::{ManagedProcess, ProcessState};
use crate::error::AppResult;

/// Internal command for the daemon manager.
#[derive(Debug)]
pub enum ManagerCommand {
    /// Start a daemon
    Start { id: String, respond: ResponseSender },
    /// Stop a daemon
    Stop { id: String, respond: ResponseSender },
    /// Restart a daemon
    Restart { id: String, respond: ResponseSender },
    /// Reload a daemon (SIGHUP)
    Reload { id: String, respond: ResponseSender },
    /// Get status of all daemons
    Status { respond: ResponseSender },
    /// Shutdown all daemons
    Shutdown { respond: ResponseSender },
}

type ResponseSender = mpsc::Sender<DaemonResponse>;

/// Daemon manager for controlling ZoneMinder daemons.
#[derive(Clone)]
pub struct DaemonManager {
    /// Map of daemon ID to process info
    processes: Arc<RwLock<HashMap<String, ManagedProcess>>>,
    /// Map of PID to daemon ID for signal handling
    pid_map: Arc<RwLock<HashMap<u32, String>>>,
    /// Configuration
    config: Arc<DaemonConfig>,
    /// Shutdown signal
    shutdown: Arc<Notify>,
    /// Server ID for ZoneMinder distributed mode
    server_id: Option<u32>,
    /// Whether the manager is running
    running: Arc<RwLock<bool>>,
}

impl DaemonManager {
    /// Create a new daemon manager.
    pub fn new(config: DaemonConfig, server_id: Option<u32>) -> Self {
        Self {
            processes: Arc::new(RwLock::new(HashMap::new())),
            pid_map: Arc::new(RwLock::new(HashMap::new())),
            config: Arc::new(config),
            shutdown: Arc::new(Notify::new()),
            server_id,
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Get the server ID.
    pub fn server_id(&self) -> Option<u32> {
        self.server_id
    }

    /// Check if the manager is running.
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }

    /// Get the configuration.
    pub fn config(&self) -> &DaemonConfig {
        &self.config
    }

    /// Signal shutdown to all background tasks.
    pub fn signal_shutdown(&self) {
        self.shutdown.notify_waiters();
    }

    /// Start a daemon process.
    pub async fn start_daemon(&self, id: &str, args: &[String]) -> AppResult<DaemonResponse> {
        // Check if already running
        {
            let processes = self.processes.read().await;
            if let Some(process) = processes.get(id) {
                if process.is_running() {
                    return Ok(DaemonResponse::error(format!(
                        "Daemon {} is already running",
                        id
                    )));
                }
            }
        }

        // Parse the daemon command
        let (command, daemon_args) = parse_daemon_command(id, args);
        let full_path = self.config.resolve_daemon_path(&command);

        info!("Starting daemon: {} {:?}", full_path.display(), daemon_args);

        // Check if command exists
        if !full_path.exists() {
            return Ok(DaemonResponse::error(format!(
                "Daemon executable not found: {}",
                full_path.display()
            )));
        }

        // Spawn the process
        let child = match Command::new(&full_path).args(&daemon_args).spawn() {
            Ok(child) => child,
            Err(e) => {
                error!("Failed to spawn {}: {}", id, e);
                return Ok(DaemonResponse::error(format!(
                    "Failed to start {}: {}",
                    id, e
                )));
            }
        };

        let pid = child.id();

        // Create or update the process entry
        let mut processes = self.processes.write().await;
        let process = processes.entry(id.to_string()).or_insert_with(|| {
            let monitor_id = extract_monitor_id(&daemon_args);
            ManagedProcess::new(id, id, &command, daemon_args.clone(), true, monitor_id)
        });

        process.set_child(child);

        // Update PID map
        if let Some(p) = pid {
            let mut pid_map = self.pid_map.write().await;
            pid_map.insert(p, id.to_string());
        }

        info!("Started daemon {} with PID {:?}", id, pid);
        Ok(DaemonResponse::ok(format!(
            "Started {} (PID: {:?})",
            id, pid
        )))
    }

    /// Stop a daemon process.
    pub async fn stop_daemon(&self, id: &str) -> AppResult<DaemonResponse> {
        let mut processes = self.processes.write().await;

        let process = match processes.get_mut(id) {
            Some(p) => p,
            None => {
                return Ok(DaemonResponse::error(format!("Daemon {} not found", id)));
            }
        };

        if !process.can_stop() {
            return Ok(DaemonResponse::error(format!(
                "Daemon {} is not running (state: {})",
                id, process.state
            )));
        }

        // Disable auto-restart during intentional stop
        process.auto_restart = false;

        // Try to kill the child process
        if let Some(child) = process.child_mut() {
            match child.kill().await {
                Ok(_) => {
                    info!("Sent SIGKILL to daemon {}", id);
                }
                Err(e) => {
                    warn!("Failed to kill daemon {}: {}", id, e);
                }
            }
        }

        // Update PID map
        if let Some(pid) = process.pid {
            let mut pid_map = self.pid_map.write().await;
            pid_map.remove(&pid);
        }

        process.set_state(ProcessState::Stopped);

        Ok(DaemonResponse::ok(format!("Stopped {}", id)))
    }

    /// Restart a daemon process.
    pub async fn restart_daemon(&self, id: &str) -> AppResult<DaemonResponse> {
        // Get the current args before stopping
        let args = {
            let processes = self.processes.read().await;
            processes.get(id).map(|p| p.args.clone())
        };

        // Stop if running
        {
            let processes = self.processes.read().await;
            if let Some(process) = processes.get(id) {
                if process.is_running() {
                    drop(processes);
                    self.stop_daemon(id).await?;
                }
            }
        }

        // Re-enable auto-restart
        {
            let mut processes = self.processes.write().await;
            if let Some(process) = processes.get_mut(id) {
                process.auto_restart = true;
            }
        }

        // Start with the same args (or empty if not found)
        let args = args.unwrap_or_default();
        self.start_daemon(id, &args).await
    }

    /// Send SIGHUP to reload daemon configuration.
    pub async fn reload_daemon(&self, id: &str) -> AppResult<DaemonResponse> {
        let processes = self.processes.read().await;

        let process = match processes.get(id) {
            Some(p) => p,
            None => {
                return Ok(DaemonResponse::error(format!("Daemon {} not found", id)));
            }
        };

        if !process.is_running() {
            return Ok(DaemonResponse::error(format!(
                "Daemon {} is not running",
                id
            )));
        }

        let pid = match process.pid {
            Some(p) => p,
            None => {
                return Ok(DaemonResponse::error(format!("Daemon {} has no PID", id)));
            }
        };

        // Send SIGHUP
        #[cfg(unix)]
        {
            use nix::sys::signal::{kill, Signal};
            use nix::unistd::Pid;

            match kill(Pid::from_raw(pid as i32), Signal::SIGHUP) {
                Ok(_) => {
                    info!("Sent SIGHUP to daemon {} (PID {})", id, pid);
                    Ok(DaemonResponse::ok(format!(
                        "Sent reload signal to {} (PID {})",
                        id, pid
                    )))
                }
                Err(e) => Ok(DaemonResponse::error(format!(
                    "Failed to send SIGHUP to {}: {}",
                    id, e
                ))),
            }
        }

        #[cfg(not(unix))]
        {
            Ok(DaemonResponse::error(
                "SIGHUP not supported on this platform",
            ))
        }
    }

    /// Get status of all daemons.
    pub async fn get_status(&self) -> SystemStatus {
        let processes = self.processes.read().await;
        let running = *self.running.read().await;

        let daemons: Vec<ProcessStatus> = processes
            .values()
            .map(|p| ProcessStatus {
                id: p.id.clone(),
                name: p.name.clone(),
                state: p.state,
                pid: p.pid,
                uptime_seconds: p.uptime().map(|d| d.as_secs()),
                restart_count: p.restart_count,
                monitor_id: p.monitor_id,
            })
            .collect();

        SystemStatus {
            running,
            daemons,
            stats: None, // Stats are updated separately
        }
    }

    /// Get status of a specific daemon.
    pub async fn get_daemon_status(&self, id: &str) -> Option<ProcessStatus> {
        let processes = self.processes.read().await;
        processes.get(id).map(|p| ProcessStatus {
            id: p.id.clone(),
            name: p.name.clone(),
            state: p.state,
            pid: p.pid,
            uptime_seconds: p.uptime().map(|d| d.as_secs()),
            restart_count: p.restart_count,
            monitor_id: p.monitor_id,
        })
    }

    /// Shutdown all daemons.
    pub async fn shutdown_all(&self) -> AppResult<DaemonResponse> {
        info!("Shutting down all daemons");

        let mut stopped = 0;
        let mut failed = 0;

        let ids: Vec<String> = {
            let processes = self.processes.read().await;
            processes.keys().cloned().collect()
        };

        for id in ids {
            match self.stop_daemon(&id).await {
                Ok(resp) if resp.success => stopped += 1,
                _ => failed += 1,
            }
        }

        // Mark as not running
        *self.running.write().await = false;

        // Signal shutdown
        self.signal_shutdown();

        if failed > 0 {
            Ok(DaemonResponse::ok(format!(
                "Shutdown complete: {} stopped, {} failed",
                stopped, failed
            )))
        } else {
            Ok(DaemonResponse::ok(format!(
                "Shutdown complete: {} daemons stopped",
                stopped
            )))
        }
    }

    /// Start the daemon manager and background health check task.
    pub async fn startup(self: &Arc<Self>) -> AppResult<DaemonResponse> {
        let mut running = self.running.write().await;
        if *running {
            return Ok(DaemonResponse::error("Daemon manager already running"));
        }

        *running = true;
        drop(running); // Release lock before spawning

        // Start background health check task
        let manager = Arc::clone(self);
        let check_interval = self.config.watch_check_interval();
        let max_delay = self.config.watch_max_delay();

        tokio::spawn(async move {
            manager
                .run_health_check_loop(check_interval, max_delay)
                .await;
        });

        info!("Daemon manager started with health monitoring");

        Ok(DaemonResponse::ok("Daemon manager started"))
    }

    /// Run the background health check loop.
    async fn run_health_check_loop(
        &self,
        check_interval: std::time::Duration,
        max_delay: std::time::Duration,
    ) {
        // Initial delay before first check (like zmwatch.pl's 30 second delay)
        let startup_delay = std::time::Duration::from_secs(30);

        info!(
            "Health monitor starting (startup delay: {:?}, check interval: {:?}, max delay: {:?})",
            startup_delay, check_interval, max_delay
        );

        tokio::select! {
            _ = tokio::time::sleep(startup_delay) => {},
            _ = self.shutdown.notified() => {
                info!("Health monitor shutdown during startup delay");
                return;
            }
        }

        let mut interval = tokio::time::interval(check_interval);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    self.perform_health_check(max_delay).await;
                }
                _ = self.shutdown.notified() => {
                    info!("Health monitor received shutdown signal");
                    break;
                }
            }
        }

        info!("Health monitor stopped");
    }

    /// Perform a single health check cycle.
    async fn perform_health_check(&self, max_delay: std::time::Duration) {
        debug!("Performing health check");

        // Check for exited processes and handle restarts
        self.check_daemons().await;

        // Check for hung processes
        let hung_processes = self.check_for_hung_processes(max_delay).await;

        for id in hung_processes {
            warn!(
                "Process {} appears hung (no CPU activity for {:?}), restarting",
                id, max_delay
            );
            if let Err(e) = self.restart_daemon(&id).await {
                error!("Failed to restart hung daemon {}: {}", id, e);
            }
        }
    }

    /// Check for processes that appear to be hung (no CPU activity).
    async fn check_for_hung_processes(&self, max_delay: std::time::Duration) -> Vec<String> {
        let mut hung = Vec::new();
        let mut processes = self.processes.write().await;

        for (id, process) in processes.iter_mut() {
            // Update activity tracking
            process.check_activity();

            // Check if process appears hung
            if process.appears_hung(max_delay) {
                hung.push(id.clone());
            }
        }

        hung
    }

    /// Check daemon health and restart if needed.
    pub async fn check_daemons(&self) {
        let mut to_restart = Vec::new();

        {
            let mut processes = self.processes.write().await;

            for (id, process) in processes.iter_mut() {
                // Check if the process has exited
                if let Some(child) = process.child_mut() {
                    match child.try_wait() {
                        Ok(Some(status)) => {
                            // Process exited
                            info!("Daemon {} exited with status: {:?}", id, status);

                            // Remove from PID map
                            if let Some(pid) = process.pid {
                                let mut pid_map = self.pid_map.blocking_write();
                                pid_map.remove(&pid);
                            }

                            if process.auto_restart {
                                // Prepare for restart with backoff
                                process.prepare_restart(
                                    self.config.min_backoff(),
                                    self.config.max_backoff(),
                                );
                                debug!(
                                    "Daemon {} will restart in {:?}",
                                    id, process.current_backoff
                                );
                            } else {
                                process.set_state(ProcessState::Stopped);
                            }
                        }
                        Ok(None) => {
                            // Process still running
                        }
                        Err(e) => {
                            warn!("Error checking daemon {} status: {}", id, e);
                        }
                    }
                }

                // Check for pending restarts
                if process.state == ProcessState::Restarting && process.backoff_elapsed() {
                    to_restart.push((id.clone(), process.args.clone()));
                }
            }
        }

        // Restart pending daemons
        for (id, args) in to_restart {
            info!("Restarting daemon {} after backoff", id);
            if let Err(e) = self.start_daemon(&id, &args).await {
                error!("Failed to restart {}: {}", id, e);
            }
        }
    }

    /// Register a daemon without starting it.
    pub async fn register_daemon(&self, process: ManagedProcess) {
        let mut processes = self.processes.write().await;
        processes.insert(process.id.clone(), process);
    }

    /// Get list of all daemon IDs.
    pub async fn list_daemon_ids(&self) -> Vec<String> {
        let processes = self.processes.read().await;
        processes.keys().cloned().collect()
    }
}

/// Parse a daemon command string into command and args.
fn parse_daemon_command(id: &str, extra_args: &[String]) -> (String, Vec<String>) {
    let parts: Vec<&str> = id.split_whitespace().collect();
    if parts.is_empty() {
        return (id.to_string(), extra_args.to_vec());
    }

    let command = parts[0].to_string();
    let mut args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();
    args.extend(extra_args.iter().cloned());

    (command, args)
}

/// Extract monitor ID from daemon arguments.
fn extract_monitor_id(args: &[String]) -> Option<u32> {
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if arg == "-m" {
            if let Some(id_str) = iter.next() {
                return id_str.parse().ok();
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_daemon_command_simple() {
        let (cmd, args) = parse_daemon_command("zmfilter.pl", &[]);
        assert_eq!(cmd, "zmfilter.pl");
        assert!(args.is_empty());
    }

    #[test]
    fn test_parse_daemon_command_with_args() {
        let (cmd, args) = parse_daemon_command("zmc -m 1", &[]);
        assert_eq!(cmd, "zmc");
        assert_eq!(args, vec!["-m", "1"]);
    }

    #[test]
    fn test_parse_daemon_command_with_extra_args() {
        let (cmd, args) = parse_daemon_command("zmc", &["-m".to_string(), "5".to_string()]);
        assert_eq!(cmd, "zmc");
        assert_eq!(args, vec!["-m", "5"]);
    }

    #[test]
    fn test_extract_monitor_id() {
        assert_eq!(
            extract_monitor_id(&["-m".to_string(), "5".to_string()]),
            Some(5)
        );
        assert_eq!(
            extract_monitor_id(&["-d".to_string(), "/dev/video0".to_string()]),
            None
        );
        assert_eq!(extract_monitor_id(&[]), None);
    }

    #[tokio::test]
    async fn test_daemon_manager_creation() {
        let config = DaemonConfig::default();
        let manager = DaemonManager::new(config, Some(1));

        assert_eq!(manager.server_id(), Some(1));
        assert!(!manager.is_running().await);
    }

    #[tokio::test]
    async fn test_daemon_manager_startup() {
        let config = DaemonConfig::default();
        let manager = Arc::new(DaemonManager::new(config, None));

        let resp = manager.startup().await.unwrap();
        assert!(resp.success);
        assert!(manager.is_running().await);

        // Signal shutdown to stop background task
        manager.signal_shutdown();
    }

    #[tokio::test]
    async fn test_daemon_manager_double_startup() {
        let config = DaemonConfig::default();
        let manager = Arc::new(DaemonManager::new(config, None));

        manager.startup().await.unwrap();
        let resp = manager.startup().await.unwrap();
        assert!(!resp.success); // Already running

        // Signal shutdown to stop background task
        manager.signal_shutdown();
    }

    #[tokio::test]
    async fn test_get_status_empty() {
        let config = DaemonConfig::default();
        let manager = DaemonManager::new(config, None);

        let status = manager.get_status().await;
        assert!(!status.running);
        assert!(status.daemons.is_empty());
    }

    #[tokio::test]
    async fn test_register_daemon() {
        let config = DaemonConfig::default();
        let manager = DaemonManager::new(config, None);

        let process = ManagedProcess::new("test", "Test", "test", vec![], true, None);
        manager.register_daemon(process).await;

        let ids = manager.list_daemon_ids().await;
        assert_eq!(ids, vec!["test"]);
    }
}
