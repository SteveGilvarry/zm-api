//! Daemon manager - core process control and lifecycle management.

use std::collections::HashMap;
use std::sync::Arc;

use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use tokio::process::Command;
use tokio::sync::{mpsc, Notify, RwLock};
use tracing::{debug, error, info, warn};

use crate::daemon::config::DaemonConfig;
use crate::daemon::daemons::DaemonDefinition;
use crate::daemon::ipc::{DaemonResponse, ProcessStatus, SystemStatus};
use crate::daemon::process::{ManagedProcess, ProcessState};
use crate::entity::monitors;
use crate::entity::sea_orm_active_enums::{Capturing, Function, MonitorType};
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
    /// Database connection for querying monitors
    db: Option<Arc<DatabaseConnection>>,
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
            db: None,
        }
    }

    /// Set the database connection for querying monitors.
    pub fn set_database(&mut self, db: Arc<DatabaseConnection>) {
        self.db = Some(db);
    }

    /// Create a new daemon manager with database connection.
    pub fn with_database(
        config: DaemonConfig,
        server_id: Option<u32>,
        db: Arc<DatabaseConnection>,
    ) -> Self {
        Self {
            processes: Arc::new(RwLock::new(HashMap::new())),
            pid_map: Arc::new(RwLock::new(HashMap::new())),
            config: Arc::new(config),
            shutdown: Arc::new(Notify::new()),
            server_id,
            running: Arc::new(RwLock::new(false)),
            db: Some(db),
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

    /// Stop a daemon process gracefully.
    ///
    /// This follows zmdc.pl behavior:
    /// 1. Send SIGTERM to allow graceful shutdown
    /// 2. The health check loop monitors terminating processes
    /// 3. After shutdown_timeout (30s), SIGKILL is sent
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

        // Get the PID for sending signal
        let pid = match process.pid {
            Some(p) => p,
            None => {
                process.set_state(ProcessState::Stopped);
                return Ok(DaemonResponse::ok(format!("Stopped {} (no PID)", id)));
            }
        };

        // Send SIGTERM for graceful shutdown (like zmdc.pl's send_stop)
        #[cfg(unix)]
        {
            use nix::sys::signal::{kill, Signal};
            use nix::unistd::Pid;

            match kill(Pid::from_raw(pid as i32), Signal::SIGTERM) {
                Ok(_) => {
                    info!("Sent SIGTERM to daemon {} (PID {})", id, pid);
                    process.mark_term_sent();
                }
                Err(e) => {
                    // Process may have already exited
                    warn!(
                        "Failed to send SIGTERM to daemon {} (PID {}): {}",
                        id, pid, e
                    );
                    process.set_state(ProcessState::Stopped);

                    // Clean up PID map
                    drop(processes); // Release lock before acquiring pid_map lock
                    let mut pid_map = self.pid_map.write().await;
                    pid_map.remove(&pid);

                    return Ok(DaemonResponse::ok(format!(
                        "Stopped {} (process already gone)",
                        id
                    )));
                }
            }
        }

        #[cfg(not(unix))]
        {
            // On non-Unix, fall back to tokio's kill (which is SIGKILL)
            if let Some(child) = process.child_mut() {
                let _ = child.kill().await;
            }
            process.set_state(ProcessState::Stopped);

            drop(processes);
            let mut pid_map = self.pid_map.write().await;
            pid_map.remove(&pid);

            return Ok(DaemonResponse::ok(format!("Stopped {}", id)));
        }

        Ok(DaemonResponse::ok(format!(
            "Sent stop signal to {} (PID {}), waiting for graceful shutdown",
            id, pid
        )))
    }

    /// Force kill a daemon process immediately with SIGKILL.
    ///
    /// Use this when graceful shutdown has timed out or immediate termination is needed.
    pub async fn kill_daemon(&self, id: &str) -> AppResult<DaemonResponse> {
        let mut processes = self.processes.write().await;

        let process = match processes.get_mut(id) {
            Some(p) => p,
            None => {
                return Ok(DaemonResponse::error(format!("Daemon {} not found", id)));
            }
        };

        let pid = process.pid;

        // Try to kill via child handle first
        if let Some(child) = process.child_mut() {
            match child.kill().await {
                Ok(_) => {
                    info!("Sent SIGKILL to daemon {} via child handle", id);
                }
                Err(e) => {
                    warn!("Failed to kill daemon {} via child handle: {}", id, e);
                }
            }
        }

        // Also try sending SIGKILL directly to PID (in case child handle is stale)
        #[cfg(unix)]
        if let Some(p) = pid {
            use nix::sys::signal::{kill, Signal};
            use nix::unistd::Pid;

            let _ = kill(Pid::from_raw(p as i32), Signal::SIGKILL);
        }

        // Update state
        process.set_state(ProcessState::Stopped);
        process.term_sent_at = None;

        // Clean up PID map
        if let Some(p) = pid {
            drop(processes);
            let mut pid_map = self.pid_map.write().await;
            pid_map.remove(&p);
        }

        Ok(DaemonResponse::ok(format!("Killed {}", id)))
    }

    /// Check for processes that need to be force-killed after timeout.
    ///
    /// This matches zmdc.pl's check_for_processes_to_kill behavior:
    /// processes that were sent SIGTERM but haven't died after KILL_DELAY
    /// get sent SIGKILL.
    pub async fn check_terminating_processes(&self) {
        let timeout = self.config.shutdown_timeout();
        let mut to_kill = Vec::new();

        {
            let processes = self.processes.read().await;
            for (id, process) in processes.iter() {
                if process.state == ProcessState::Stopping && process.term_timeout_expired(timeout)
                {
                    to_kill.push(id.clone());
                }
            }
        }

        for id in to_kill {
            warn!(
                "Daemon {} has not stopped after {:?}, sending SIGKILL",
                id, timeout
            );
            if let Err(e) = self.kill_daemon(&id).await {
                error!("Failed to force kill daemon {}: {}", id, e);
            }
        }
    }

    /// Restart a daemon process.
    ///
    /// If `provided_args` is non-empty, those args are used. Otherwise, we try
    /// to use the args from the existing process entry, falling back to parsing
    /// args from the daemon ID string (e.g., "zmc -m 1").
    pub async fn restart_daemon(
        &self,
        id: &str,
        provided_args: &[String],
    ) -> AppResult<DaemonResponse> {
        // Determine which args to use
        let args = if !provided_args.is_empty() {
            // Use provided args (from socket command or API call)
            provided_args.to_vec()
        } else {
            // Try to get args from existing process entry
            let processes = self.processes.read().await;
            processes
                .get(id)
                .map(|p| p.args.clone())
                .unwrap_or_default()
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

    /// Shutdown all daemons gracefully.
    ///
    /// This matches zmdc.pl's shutdownAll behavior:
    /// 1. Send SIGTERM to all processes
    /// 2. Wait for processes to exit (up to shutdown_timeout)
    /// 3. Force kill any remaining processes with SIGKILL
    pub async fn shutdown_all(&self) -> AppResult<DaemonResponse> {
        info!("Shutting down all daemons");

        let timeout = self.config.shutdown_timeout();
        let check_interval = std::time::Duration::from_millis(500);

        // Step 1: Send SIGTERM to all running processes
        let ids: Vec<String> = {
            let processes = self.processes.read().await;
            processes
                .iter()
                .filter(|(_, p)| p.is_running() || p.state == ProcessState::Starting)
                .map(|(id, _)| id.clone())
                .collect()
        };

        let total = ids.len();
        info!("Sending stop signal to {} daemons", total);

        for id in &ids {
            let _ = self.stop_daemon(id).await;
        }

        // Step 2: Wait for processes to exit, checking periodically
        let start = std::time::Instant::now();
        loop {
            // Check for exited processes
            self.check_daemons().await;

            // Count how many are still stopping
            let still_stopping = {
                let processes = self.processes.read().await;
                processes
                    .values()
                    .filter(|p| p.state == ProcessState::Stopping)
                    .count()
            };

            if still_stopping == 0 {
                info!("All daemons have stopped");
                break;
            }

            if start.elapsed() >= timeout {
                warn!(
                    "{} daemons still running after {:?}, sending SIGKILL",
                    still_stopping, timeout
                );
                // Force kill remaining processes
                self.check_terminating_processes().await;
                break;
            }

            debug!(
                "Waiting for {} daemons to stop ({:?} elapsed)",
                still_stopping,
                start.elapsed()
            );
            tokio::time::sleep(check_interval).await;
        }

        // Mark as not running
        *self.running.write().await = false;

        // Signal shutdown to background tasks
        self.signal_shutdown();

        // Count final results
        let stopped = {
            let processes = self.processes.read().await;
            processes
                .values()
                .filter(|p| p.state == ProcessState::Stopped)
                .count()
        };

        if stopped == total {
            Ok(DaemonResponse::ok(format!(
                "Shutdown complete: {} daemons stopped",
                stopped
            )))
        } else {
            Ok(DaemonResponse::ok(format!(
                "Shutdown complete: {} of {} daemons stopped",
                stopped, total
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

    /// Start all ZoneMinder daemons.
    ///
    /// This matches the behavior of zmpkg.pl startup:
    /// 1. Query monitors that are not deleted and have Capturing != None
    /// 2. Skip WebSite type monitors (they don't need capture daemons)
    /// 3. For Local type monitors: start `zmc -d <device>`
    /// 4. For other types: start `zmc -m <id>`
    /// 5. Start zma for monitors requiring motion detection (Modect/Mocord)
    /// 6. Start zmcontrol.pl for controllable monitors
    /// 7. Start zmtrack.pl for monitors with motion tracking enabled
    /// 8. Start singleton daemons (zmfilter.pl, zmstats.pl, etc.)
    pub async fn start_all_daemons(self: &Arc<Self>) -> AppResult<DaemonResponse> {
        let db = match &self.db {
            Some(db) => db.clone(),
            None => {
                return Ok(DaemonResponse::error(
                    "Database not configured - cannot query monitors",
                ));
            }
        };

        // First ensure we're running (starts health check loop)
        if !self.is_running().await {
            self.startup().await?;
        }

        let mut started = 0;
        let mut failed = 0;
        let mut errors: Vec<String> = Vec::new();

        // Query monitors from database
        // Match zmpkg.pl logic: Deleted => 0, Capturing != 'None'
        let monitor_list = monitors::Entity::find()
            .filter(monitors::Column::Deleted.eq(false))
            .all(db.as_ref())
            .await?;

        // Filter monitors that need capture (Capturing != None)
        // Also filter by server_id if configured (multi-server support)
        let monitors_to_start: Vec<_> = monitor_list
            .iter()
            .filter(|m| {
                // Skip if Capturing is None
                if matches!(m.capturing, Capturing::None) {
                    return false;
                }
                // Skip WebSite type monitors (they don't need zmc)
                if matches!(m.r#type, MonitorType::WebSite) {
                    return false;
                }
                // Multi-server support: if server_id is configured, filter by it
                if let Some(our_server_id) = self.server_id {
                    if let Some(monitor_server_id) = m.server_id {
                        return monitor_server_id == our_server_id;
                    }
                    // If monitor has no server_id, include it (backwards compatibility)
                }
                true
            })
            .collect();

        info!(
            "Found {} monitors to start (of {} total)",
            monitors_to_start.len(),
            monitor_list.len()
        );

        // Start per-monitor daemons
        for monitor in &monitors_to_start {
            let monitor_id = monitor.id;
            let function = &monitor.function;
            let monitor_type = &monitor.r#type;

            // Determine if we need analysis daemon (motion detection)
            let needs_analysis = matches!(function, Function::Modect | Function::Mocord);

            debug!(
                "Monitor {} ({}): type={:?}, function={:?}, analysis={}",
                monitor_id, monitor.name, monitor_type, function, needs_analysis
            );

            // Start zmc (capture daemon)
            // For Local type: use -d <device>, otherwise use -m <id>
            let (daemon_id, daemon_desc) =
                if matches!(monitor_type, MonitorType::Local) && !monitor.device.is_empty() {
                    (
                        format!("zmc -d {}", monitor.device),
                        format!("zmc for device {}", monitor.device),
                    )
                } else {
                    (
                        format!("zmc -m {}", monitor_id),
                        format!("zmc for monitor {}", monitor_id),
                    )
                };

            match self.start_daemon(&daemon_id, &[]).await {
                Ok(resp) if resp.success => {
                    started += 1;
                    info!("Started {}", daemon_desc);
                }
                Ok(resp) => {
                    if !resp.message.contains("already running") {
                        failed += 1;
                        errors.push(format!("{}: {}", daemon_id, resp.message));
                        warn!("Failed to start {}: {}", daemon_desc, resp.message);
                    }
                }
                Err(e) => {
                    failed += 1;
                    errors.push(format!("{}: {}", daemon_id, e));
                    error!("Error starting {}: {}", daemon_desc, e);
                }
            }

            // Start zma (analysis daemon) if needed for motion detection
            if needs_analysis {
                let daemon_id = format!("zma -m {}", monitor_id);
                match self.start_daemon(&daemon_id, &[]).await {
                    Ok(resp) if resp.success => {
                        started += 1;
                        info!("Started zma for monitor {}", monitor_id);
                    }
                    Ok(resp) => {
                        if !resp.message.contains("already running") {
                            failed += 1;
                            errors.push(format!("zma -m {}: {}", monitor_id, resp.message));
                            warn!(
                                "Failed to start zma for monitor {}: {}",
                                monitor_id, resp.message
                            );
                        }
                    }
                    Err(e) => {
                        failed += 1;
                        errors.push(format!("zma -m {}: {}", monitor_id, e));
                        error!("Error starting zma for monitor {}: {}", monitor_id, e);
                    }
                }
            }

            // Start zmcontrol.pl for controllable monitors (PTZ control)
            if monitor.controllable != 0 {
                let daemon_id = format!("zmcontrol.pl --id {}", monitor_id);
                match self.start_daemon(&daemon_id, &[]).await {
                    Ok(resp) if resp.success => {
                        started += 1;
                        info!("Started zmcontrol.pl for monitor {}", monitor_id);
                    }
                    Ok(resp) => {
                        if !resp.message.contains("already running")
                            && !resp.message.contains("not found")
                        {
                            warn!(
                                "Could not start zmcontrol.pl for monitor {}: {}",
                                monitor_id, resp.message
                            );
                        }
                    }
                    Err(e) => {
                        // zmcontrol.pl may not be installed - just warn
                        warn!(
                            "Could not start zmcontrol.pl for monitor {}: {}",
                            monitor_id, e
                        );
                    }
                }
            }

            // Start zmtrack.pl for monitors with motion tracking
            if monitor.track_motion != 0 && needs_analysis {
                let daemon_id = format!("zmtrack.pl -m {}", monitor_id);
                match self.start_daemon(&daemon_id, &[]).await {
                    Ok(resp) if resp.success => {
                        started += 1;
                        info!("Started zmtrack.pl for monitor {}", monitor_id);
                    }
                    Ok(resp) => {
                        if !resp.message.contains("already running")
                            && !resp.message.contains("not found")
                        {
                            warn!(
                                "Could not start zmtrack.pl for monitor {}: {}",
                                monitor_id, resp.message
                            );
                        }
                    }
                    Err(e) => {
                        // zmtrack.pl may not be installed - just warn
                        warn!(
                            "Could not start zmtrack.pl for monitor {}: {}",
                            monitor_id, e
                        );
                    }
                }
            }
        }

        // Start singleton daemons in priority order
        let mut singletons: Vec<_> = DaemonDefinition::singletons()
            .filter(|d| d.requires_db)
            .collect();
        singletons.sort_by_key(|d| d.priority);

        for daemon in singletons {
            debug!(
                "Starting singleton daemon: {} (priority {})",
                daemon.name, daemon.priority
            );

            match self.start_daemon(daemon.command, &[]).await {
                Ok(resp) if resp.success => {
                    started += 1;
                    info!("Started {}", daemon.name);
                }
                Ok(resp) => {
                    if !resp.message.contains("already running") {
                        warn!("Could not start {}: {}", daemon.name, resp.message);
                    }
                }
                Err(e) => {
                    warn!("Could not start {}: {}", daemon.name, e);
                }
            }
        }

        let message = if failed > 0 {
            format!(
                "System startup completed: {} daemons started, {} failed. Errors: {}",
                started,
                failed,
                errors.join("; ")
            )
        } else {
            format!("System startup completed: {} daemons started", started)
        };

        info!("{}", message);

        if failed > 0 && started == 0 {
            Ok(DaemonResponse::error(message))
        } else {
            Ok(DaemonResponse::ok(message))
        }
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

        // Check for processes that need SIGKILL after graceful shutdown timeout
        // (like zmdc.pl's check_for_processes_to_kill)
        self.check_terminating_processes().await;

        // Check for hung processes
        let hung_processes = self.check_for_hung_processes(max_delay).await;

        for id in hung_processes {
            warn!(
                "Process {} appears hung (no CPU activity for {:?}), restarting",
                id, max_delay
            );
            // Pass empty args - restart_daemon will get args from existing process entry
            if let Err(e) = self.restart_daemon(&id, &[]).await {
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
        let mut pids_to_remove = Vec::new();

        {
            let mut processes = self.processes.write().await;

            for (id, process) in processes.iter_mut() {
                // Check if the process has exited
                if let Some(child) = process.child_mut() {
                    match child.try_wait() {
                        Ok(Some(status)) => {
                            // Process exited
                            let was_stopping = process.state == ProcessState::Stopping;

                            if was_stopping {
                                info!("Daemon {} gracefully stopped with status: {:?}", id, status);
                            } else {
                                info!("Daemon {} exited with status: {:?}", id, status);
                            }

                            // Collect PID for removal (will remove after releasing lock)
                            if let Some(pid) = process.pid {
                                pids_to_remove.push(pid);
                            }

                            // Clear term_sent_at since process has exited
                            process.term_sent_at = None;

                            // If the process was being stopped (Stopping state) or
                            // auto_restart is disabled, just mark it stopped
                            if was_stopping || !process.auto_restart {
                                process.set_state(ProcessState::Stopped);
                            } else {
                                // Prepare for restart with backoff
                                process.prepare_restart(
                                    self.config.min_backoff(),
                                    self.config.max_backoff(),
                                );
                                debug!(
                                    "Daemon {} will restart in {:?}",
                                    id, process.current_backoff
                                );
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

        // Remove PIDs from map (now that processes lock is released)
        if !pids_to_remove.is_empty() {
            let mut pid_map = self.pid_map.write().await;
            for pid in pids_to_remove {
                pid_map.remove(&pid);
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
