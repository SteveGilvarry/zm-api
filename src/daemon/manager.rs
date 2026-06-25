//! Daemon manager - core process control and lifecycle management.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use std::path::PathBuf;

use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder,
};
use tokio::process::Command;
use tokio::sync::{mpsc, Notify, RwLock};
use tracing::{debug, error, info, warn};

use crate::configure::zmnext::ZmNextConfig;
use crate::daemon::config::DaemonConfig;
use crate::daemon::daemons::DaemonDefinition;
use crate::daemon::ipc::{DaemonResponse, ProcessStatus, SystemStats, SystemStatus};
use crate::daemon::process::{ManagedProcess, ProcessState};
use crate::daemon::stats;
use crate::entity::sea_orm_active_enums::{Capturing, Function, MonitorType, Status};
use crate::entity::{filters, monitors, servers, storage, zones};
use crate::error::AppResult;
use crate::service::zmnext::pipeline;

/// Runtime context the manager needs to drive zm-next workers: the validated
/// config plus the stream-socket directory (from the streaming config) used to
/// build each worker's `--socket` path.
#[derive(Debug, Clone)]
struct ZmNextRuntime {
    config: ZmNextConfig,
    socks_path: String,
    /// Monitor ids whose pipeline emits motion-synopsis ingredients.
    synopsis_monitors: std::collections::HashSet<u32>,
}

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
    /// Latched shutdown flag checked by start/reconcile paths (the `Notify`
    /// signal alone is lost on a task that is mid-tick).
    shutting_down: Arc<AtomicBool>,
    /// Server ID for ZoneMinder distributed mode
    server_id: Option<u32>,
    /// Whether the manager is running
    running: Arc<RwLock<bool>>,
    /// Database connection for querying monitors
    db: Option<Arc<DatabaseConnection>>,
    /// zm-next worker runtime; `None` (the default) means every monitor stays
    /// on legacy zmc/zma regardless of any per-monitor flag.
    zmnext: Option<Arc<ZmNextRuntime>>,
}

impl DaemonManager {
    /// Create a new daemon manager.
    pub fn new(config: DaemonConfig, server_id: Option<u32>) -> Self {
        Self {
            processes: Arc::new(RwLock::new(HashMap::new())),
            pid_map: Arc::new(RwLock::new(HashMap::new())),
            config: Arc::new(config),
            shutdown: Arc::new(Notify::new()),
            shutting_down: Arc::new(AtomicBool::new(false)),
            server_id,
            running: Arc::new(RwLock::new(false)),
            db: None,
            zmnext: None,
        }
    }

    /// Set the database connection for querying monitors.
    pub fn set_database(&mut self, db: Arc<DatabaseConnection>) {
        self.db = Some(db);
    }

    /// Enable zm-next worker control. Call before `startup()` when
    /// `[zmnext].enabled` is set; `socks_path` is the streaming stream-socket
    /// directory used to build each worker's `--socket` path. A disabled config
    /// is ignored, keeping every monitor on legacy zmc/zma.
    pub fn set_zmnext(
        &mut self,
        config: ZmNextConfig,
        socks_path: String,
        synopsis_monitors: std::collections::HashSet<u32>,
    ) {
        if config.enabled {
            self.zmnext = Some(Arc::new(ZmNextRuntime {
                config,
                socks_path,
                synopsis_monitors,
            }));
        }
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
            shutting_down: Arc::new(AtomicBool::new(false)),
            server_id,
            running: Arc::new(RwLock::new(false)),
            db: Some(db),
            zmnext: None,
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
        // Latch before waking waiters: a loop mid-tick isn't awaiting
        // `notified()`, so it must observe the flag to avoid re-spawning.
        self.shutting_down.store(true, Ordering::SeqCst);
        self.shutdown.notify_waiters();
    }

    /// Whether shutdown has been signaled.
    pub fn is_shutting_down(&self) -> bool {
        self.shutting_down.load(Ordering::SeqCst)
    }

    /// Start a daemon process.
    pub async fn start_daemon(&self, id: &str, args: &[String]) -> AppResult<DaemonResponse> {
        // Refuse to spawn during shutdown so a reconcile/health restart can't
        // orphan a daemon that never receives the stop wave.
        if self.is_shutting_down() {
            return Ok(DaemonResponse::error(format!(
                "Manager is shutting down, refusing to start {}",
                id
            )));
        }

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

        // Security boundary: reject anything that isn't a known ZM daemon
        // shape before we resolve a path and exec. Without this, an absolute
        // path in `id` would bypass `resolve_daemon_path` (PathBuf::join
        // replaces the base on absolute input) and run arbitrary commands.
        if let Err(reason) = validate_daemon_spec(&command, &daemon_args) {
            warn!(
                "Rejected daemon spawn for id={:?} args={:?}: {}",
                id, args, reason
            );
            return Ok(DaemonResponse::error(format!(
                "Invalid daemon spec: {}",
                reason
            )));
        }

        let full_path = self.config.resolve_daemon_path(&command);

        info!("Starting daemon: {} {:?}", full_path.display(), daemon_args);

        // Check if command exists
        if !full_path.exists() {
            return Ok(DaemonResponse::error(format!(
                "Daemon executable not found: {}",
                full_path.display()
            )));
        }

        // Spawn the process with PR_SET_PDEATHSIG on Linux so children die when parent dies
        let child = spawn_daemon(&full_path, &daemon_args).map_err(|e| {
            error!("Failed to spawn {}: {}", id, e);
            crate::error::AppError::InternalServerError(format!("Failed to start {}: {}", id, e))
        })?;

        let pid = child.id();

        // Create or update the process entry
        let mut processes = self.processes.write().await;
        let process = processes.entry(id.to_string()).or_insert_with(|| {
            let monitor_id = extract_monitor_id(&daemon_args);
            ManagedProcess::new(id, id, &command, daemon_args.clone(), true, monitor_id)
        });

        // Always sync args so existing entries (e.g. previously corrupted by
        // restart arg duplication) heal back to the canonical exec list.
        process.args = daemon_args.clone();
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
    /// SIGTERM is sent and we wait for the child to actually exit (escalating
    /// to SIGKILL after `shutdown_timeout`) before spawning the replacement;
    /// otherwise the new spawn overwrites the manager's record while the old
    /// process keeps running, leaking it as an orphan.
    ///
    /// If `provided_args` is non-empty, those args are used. Otherwise, we
    /// derive the "extras" (args not already encoded in `id`) from the existing
    /// process entry so they don't get duplicated when `start_daemon` re-parses
    /// `id` and concatenates.
    pub async fn restart_daemon(
        &self,
        id: &str,
        provided_args: &[String],
    ) -> AppResult<DaemonResponse> {
        let args = if !provided_args.is_empty() {
            provided_args.to_vec()
        } else {
            let processes = self.processes.read().await;
            processes
                .get(id)
                .map(|p| extra_args_from_id(id, &p.args))
                .unwrap_or_default()
        };

        // Stop the running process and wait for it to actually exit.
        let needs_stop = {
            let processes = self.processes.read().await;
            processes.get(id).is_some_and(|p| p.is_running())
        };

        if needs_stop {
            self.stop_daemon(id).await?;
            self.wait_for_stop(id).await;
        }

        // Re-enable auto-restart for the fresh spawn (stop_daemon disabled it).
        {
            let mut processes = self.processes.write().await;
            if let Some(process) = processes.get_mut(id) {
                process.auto_restart = true;
            }
        }

        self.start_daemon(id, &args).await
    }

    /// Wait for a daemon to exit after `stop_daemon` sent SIGTERM, escalating
    /// to SIGKILL once `shutdown_timeout` elapses. Mirrors the wait loop in
    /// `shutdown_all` but scoped to a single daemon. Returns once the process
    /// is no longer in `Stopping` state (whether it exited, was killed, or was
    /// never present).
    async fn wait_for_stop(&self, id: &str) {
        let timeout = self.config.shutdown_timeout();
        let tick = std::time::Duration::from_millis(100);
        let start = std::time::Instant::now();
        let mut escalated = false;

        loop {
            // Drive try_wait() ourselves so we don't have to wait for the
            // health-check loop's longer interval (default 10s).
            self.check_daemons().await;

            let still_stopping = {
                let processes = self.processes.read().await;
                processes
                    .get(id)
                    .is_some_and(|p| p.state == ProcessState::Stopping)
            };

            if !still_stopping {
                return;
            }

            if !escalated && start.elapsed() >= timeout {
                warn!(
                    "Daemon {} has not stopped after {:?}, escalating to SIGKILL",
                    id, timeout
                );
                self.check_terminating_processes().await;
                escalated = true;
            }

            if start.elapsed() >= timeout * 2 {
                warn!(
                    "Daemon {} still in Stopping state after SIGKILL escalation; giving up wait",
                    id
                );
                return;
            }

            tokio::time::sleep(tick).await;
        }
    }

    /// Send SIGHUP to reload daemon configuration.
    pub async fn reload_daemon(&self, id: &str) -> AppResult<DaemonResponse> {
        // zm-next has no live config reload: "reload" means regenerate the
        // pipeline JSON from the current DB rows and restart the worker so it
        // re-reads the file. SIGHUP would be a no-op for it.
        if let Some(monitor_id) = zmnext_monitor_id_from_id(id) {
            let Some(rt) = self.zmnext.clone() else {
                return Ok(DaemonResponse::error("zm-next is not configured"));
            };
            let monitor = match &self.db {
                Some(db) => {
                    monitors::Entity::find_by_id(monitor_id)
                        .one(db.as_ref())
                        .await?
                }
                None => None,
            };
            let Some(monitor) = monitor else {
                return Ok(DaemonResponse::error(format!(
                    "Monitor {monitor_id} not found for zm-next reload"
                )));
            };
            self.write_zmnext_pipeline(&monitor, &rt).await?;
            info!("Regenerated zm-next pipeline for monitor {monitor_id}, restarting worker");
            return self.restart_daemon(id, &[]).await;
        }

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

        // Collect current system stats
        let stats: Option<SystemStats> = stats::collect_stats().ok();

        SystemStatus {
            running,
            daemons,
            stats,
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

        // Quiesce background loops first so reconciliation/health checks can't
        // re-spawn the daemons we're about to stop.
        self.signal_shutdown();

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
        self.shutting_down.store(false, Ordering::SeqCst);
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

        // Start server status update loop if server_id is configured
        if let (Some(server_id), Some(db)) = (self.server_id, &self.db) {
            let manager = Arc::clone(self);
            let db = Arc::clone(db);
            info!(
                "Starting server status update loop for server_id={}",
                server_id
            );
            tokio::spawn(async move {
                manager.run_server_status_loop(db, server_id).await;
            });
        }

        // Start monitor reconciliation loop (syncs DB state with running daemons)
        if self.db.is_some() {
            let manager = Arc::clone(self);
            info!("Starting monitor reconciliation loop");
            tokio::spawn(async move {
                manager.run_reconciliation_loop().await;
            });
        }

        info!("Daemon manager started with health monitoring");

        Ok(DaemonResponse::ok("Daemon manager started"))
    }

    /// Start all ZoneMinder daemons.
    ///
    /// This matches the behavior of zmpkg.pl startup:
    /// 1. Ensure state table sanity (default state exists, one active)
    /// 2. Query monitors that are not deleted and have Capturing != None
    /// 3. Skip WebSite type monitors (they don't need capture daemons)
    /// 4. For Local type monitors: start `zmc -d <device>`
    /// 5. For other types: start `zmc -m <id>`
    /// 6. Start zma for monitors requiring motion detection (Modect/Mocord)
    /// 7. Start zmcontrol.pl + zmtrack.pl for controllable monitors (gated on `ZM_OPT_CONTROL`)
    /// 8. Start one `zmfilter.pl --filter_id=<id> --daemon` per Background=1 filter
    /// 9. Start singleton daemons, each gated by the matching `ZM_*` Config key
    ///    and (in multi-server mode) the corresponding `Servers` column.
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

        // Kill any orphaned daemons from previous crashes before starting fresh
        kill_orphan_daemons().await;

        // Ensure state table sanity (matches zmpkg.pl behavior)
        if let Err(e) = crate::service::daemon::ensure_state_sanity(db.as_ref()).await {
            warn!("Failed to ensure state sanity: {}", e);
            // Continue anyway - this is not fatal
        }

        // Resolve all upstream-equivalent gates (Config keys + per-server overrides)
        // up front so each daemon decision is a cheap field lookup.
        let gates = self.load_startup_gates(db.as_ref()).await;

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

            // zm-next monitors run a single self-contained worker (capture +
            // analysis + record) instead of zmc/zma/zmcontrol. PTZ for these is
            // handled out-of-band via zm-api's ONVIF client, not a daemon.
            if self.use_zmnext(monitor_id).await {
                match self.start_zmnext_worker(monitor).await {
                    Ok(resp) if resp.success => {
                        started += 1;
                        info!("Started zm-next worker for monitor {}", monitor_id);
                    }
                    Ok(resp) => {
                        if !resp.message.contains("already running") {
                            failed += 1;
                            errors.push(format!("zm-core[{}]: {}", monitor_id, resp.message));
                            warn!(
                                "Failed to start zm-next worker for monitor {}: {}",
                                monitor_id, resp.message
                            );
                        }
                    }
                    Err(e) => {
                        failed += 1;
                        errors.push(format!("zm-core[{}]: {}", monitor_id, e));
                        error!(
                            "Error starting zm-next worker for monitor {}: {}",
                            monitor_id, e
                        );
                    }
                }
                continue;
            }

            // Determine if we need analysis daemon (motion detection)
            let needs_analysis = matches!(function, Function::Modect | Function::Mocord);

            debug!(
                "Monitor {} ({}): type={:?}, function={:?}, analysis={}",
                monitor_id, monitor.name, monitor_type, function, needs_analysis
            );

            // Start zmc (capture daemon). The id form (-d <device> vs -m <id>)
            // must match every other call site — see `zmc_daemon_id`.
            let daemon_id = zmc_daemon_id(monitor_type, &monitor.device, monitor_id);
            let daemon_desc = format!("zmc for monitor {}", monitor_id);

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

            // PTZ control + motion tracking - both gated on ZM_OPT_CONTROL and
            // a controllable monitor (matches zmpkg.pl:222-233 nesting).
            if gates.zm_opt_control && monitor.controllable != 0 {
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

                if monitor.track_motion != 0 {
                    if !needs_analysis {
                        warn!(
                            "Monitor {} is set to track motion but motion detection is not enabled",
                            monitor_id
                        );
                    } else {
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
            }
        }

        // Per-filter zmfilter.pl processes (mirrors zmpkg.pl: one --daemon
        // process per Background=1 filter, pinned with --filter_id).
        match filters::Entity::find()
            .filter(filters::Column::Background.eq(1u8))
            .all(db.as_ref())
            .await
        {
            Ok(bg_filters) => {
                info!("Found {} background filters to run", bg_filters.len());
                for f in bg_filters {
                    let daemon_id = format!("zmfilter.pl --filter_id={}", f.id);
                    let args = vec!["--daemon".to_string()];
                    match self.start_daemon(&daemon_id, &args).await {
                        Ok(resp) if resp.success => {
                            started += 1;
                            info!("Started zmfilter for filter {} ({})", f.id, f.name);
                        }
                        Ok(resp) => {
                            if !resp.message.contains("already running") {
                                warn!(
                                    "Could not start zmfilter for filter {}: {}",
                                    f.id, resp.message
                                );
                            }
                        }
                        Err(e) => {
                            warn!("Error starting zmfilter for filter {}: {}", f.id, e);
                        }
                    }
                }
            }
            Err(e) => {
                warn!("Could not query background filters: {}", e);
            }
        }

        // Start singleton daemons in priority order, gated to match zmpkg.pl.
        let mut singletons: Vec<_> = DaemonDefinition::singletons()
            .filter(|d| d.requires_db)
            .collect();
        singletons.sort_by_key(|d| d.priority);

        for daemon in singletons {
            // Per-daemon gating that mirrors zmpkg.pl. Each `continue` includes
            // a debug log so operators can see why a daemon was skipped.
            match daemon.name {
                // zmfilter is started per-filter above; skip the global instance.
                "zmfilter" => continue,
                "zmaudit" => {
                    if !gates.zm_run_audit {
                        debug!("Skipping zmaudit.pl: ZM_RUN_AUDIT is disabled");
                        continue;
                    }
                    if !gates.server_zmaudit {
                        debug!("Skipping zmaudit.pl: disabled for this server");
                        continue;
                    }
                }
                "zmtrigger" => {
                    if !gates.zm_opt_triggers {
                        debug!("Skipping zmtrigger.pl: ZM_OPT_TRIGGERS is disabled");
                        continue;
                    }
                    if !gates.server_zmtrigger {
                        debug!("Skipping zmtrigger.pl: disabled for this server");
                        continue;
                    }
                }
                "zmtelemetry" if !gates.zm_telemetry_data => {
                    debug!("Skipping zmtelemetry.pl: ZM_TELEMETRY_DATA is disabled");
                    continue;
                }
                "zmeventnotification" => {
                    if !gates.zm_opt_use_eventnotification {
                        debug!(
                            "Skipping zmeventnotification.pl: ZM_OPT_USE_EVENTNOTIFICATION is disabled"
                        );
                        continue;
                    }
                    if !gates.server_zmeventnotification {
                        debug!("Skipping zmeventnotification.pl: disabled for this server");
                        continue;
                    }
                }
                "zmstats" if !gates.server_zmstats => {
                    debug!("Skipping zmstats.pl: disabled for this server");
                    continue;
                }
                _ => {}
            }

            debug!(
                "Starting singleton daemon: {} (priority {})",
                daemon.name, daemon.priority
            );

            let args: Vec<String> = daemon.default_args.iter().map(|s| s.to_string()).collect();
            match self.start_daemon(daemon.command, &args).await {
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

    /// Resolve the upstream-equivalent startup gates from `Config` and the
    /// per-server `Servers` row (in multi-server mode).
    async fn load_startup_gates(&self, db: &DatabaseConnection) -> StartupGates {
        let (
            zm_opt_control,
            zm_opt_triggers,
            zm_opt_use_eventnotification,
            zm_telemetry_data,
            zm_run_audit,
        ) = tokio::join!(
            read_bool_config(db, "ZM_OPT_CONTROL"),
            read_bool_config(db, "ZM_OPT_TRIGGERS"),
            read_bool_config(db, "ZM_OPT_USE_EVENTNOTIFICATION"),
            read_bool_config(db, "ZM_TELEMETRY_DATA"),
            read_bool_config(db, "ZM_RUN_AUDIT"),
        );

        let (server_zmstats, server_zmaudit, server_zmtrigger, server_zmeventnotification) =
            match self.server_id {
                Some(id) => match servers::Entity::find_by_id(id).one(db).await {
                    Ok(Some(s)) => (
                        s.zmstats != 0,
                        s.zmaudit != 0,
                        s.zmtrigger != 0,
                        s.zmeventnotification != 0,
                    ),
                    Ok(None) => {
                        warn!(
                            "Server id {} not found in DB; treating per-server daemon flags as enabled",
                            id
                        );
                        (true, true, true, true)
                    }
                    Err(e) => {
                        warn!(
                            "Failed to load Server row for id {}: {} (treating per-server daemon flags as enabled)",
                            id, e
                        );
                        (true, true, true, true)
                    }
                },
                None => (true, true, true, true),
            };

        StartupGates {
            zm_opt_control,
            zm_opt_triggers,
            zm_opt_use_eventnotification,
            zm_telemetry_data,
            zm_run_audit,
            server_zmstats,
            server_zmaudit,
            server_zmtrigger,
            server_zmeventnotification,
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

    /// Run periodic server status updates (every 60 seconds).
    ///
    /// This matches the behavior of zmdc.pl which updates the Server record
    /// with CPU load, memory usage, and other statistics.
    async fn run_server_status_loop(&self, db: Arc<DatabaseConnection>, server_id: u32) {
        let update_interval = Duration::from_secs(60);

        info!(
            "Server status loop starting (server_id={}, interval={:?})",
            server_id, update_interval
        );

        let mut interval = tokio::time::interval(update_interval);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        // Immediately set status to Running on first tick
        let _ = interval.tick().await;
        if let Err(e) = self.update_server_status(&db, server_id).await {
            error!("Failed initial server status update: {}", e);
        }

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    if let Err(e) = self.update_server_status(&db, server_id).await {
                        error!("Failed to update server status: {}", e);
                    }
                }
                _ = self.shutdown.notified() => {
                    // Set status to NotRunning on shutdown
                    if let Err(e) = self.set_server_not_running(&db, server_id).await {
                        error!("Failed to set server NotRunning status: {}", e);
                    }
                    break;
                }
            }
        }

        info!("Server status loop stopped");
    }

    /// Update the Server record with current system statistics.
    async fn update_server_status(&self, db: &DatabaseConnection, server_id: u32) -> AppResult<()> {
        let stats = crate::daemon::stats::collect_stats().map_err(|e| {
            crate::error::AppError::InternalServerError(format!("Failed to collect stats: {}", e))
        })?;

        let server = servers::Entity::find_by_id(server_id)
            .one(db)
            .await?
            .ok_or_else(|| {
                crate::error::AppError::NotFoundError(crate::error::Resource {
                    resource_type: crate::error::ResourceType::Config,
                    details: vec![("server_id".to_string(), server_id.to_string())],
                })
            })?;

        let mut active: servers::ActiveModel = server.into();
        active.status = Set(Status::Running);
        active.cpu_load = Set(Decimal::from_f64(stats.cpu_load));
        active.cpu_user_percent = Set(Decimal::from_f64(stats.cpu_user_percent));
        active.cpu_nice_percent = Set(Decimal::from_f64(stats.cpu_nice_percent));
        active.cpu_system_percent = Set(Decimal::from_f64(stats.cpu_system_percent));
        active.cpu_idle_percent = Set(Decimal::from_f64(stats.cpu_idle_percent));
        active.cpu_usage_percent = Set(Decimal::from_f64(stats.cpu_usage_percent));
        active.total_mem = Set(Some(stats.total_mem));
        active.free_mem = Set(Some(stats.free_mem));
        active.total_swap = Set(Some(stats.total_swap));
        active.free_swap = Set(Some(stats.free_swap));
        active.update(db).await?;

        debug!(
            "Updated server {} status: cpu_load={:.1}, cpu_usage={:.1}%, mem_used={:.1}%",
            server_id,
            stats.cpu_load,
            stats.cpu_usage_percent,
            if stats.total_mem > 0 {
                ((stats.total_mem - stats.free_mem) as f64 / stats.total_mem as f64) * 100.0
            } else {
                0.0
            }
        );

        Ok(())
    }

    /// Set the Server status to NotRunning (called on shutdown).
    async fn set_server_not_running(
        &self,
        db: &DatabaseConnection,
        server_id: u32,
    ) -> AppResult<()> {
        let server = servers::Entity::find_by_id(server_id).one(db).await?;

        if let Some(server) = server {
            let mut active: servers::ActiveModel = server.into();
            active.status = Set(Status::NotRunning);
            active.update(db).await?;
            info!("Set server {} status to NotRunning", server_id);
        }

        Ok(())
    }

    /// Run the monitor reconciliation loop.
    ///
    /// This periodically compares the desired state (from database) with the
    /// actual running state (daemons) and starts/stops processes as needed.
    /// This provides self-healing when:
    /// - The API crashes between DB update and daemon control
    /// - Daemons are started/stopped externally
    /// - System restarts
    async fn run_reconciliation_loop(&self) {
        // Reconciliation interval (check every 60 seconds)
        let interval = Duration::from_secs(60);
        // Initial delay before first reconciliation (allow startup to complete)
        let startup_delay = Duration::from_secs(45);

        info!(
            "Reconciliation loop starting (startup delay: {:?}, interval: {:?})",
            startup_delay, interval
        );

        tokio::select! {
            _ = tokio::time::sleep(startup_delay) => {},
            _ = self.shutdown.notified() => {
                info!("Reconciliation loop shutdown during startup delay");
                return;
            }
        }

        let mut ticker = tokio::time::interval(interval);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    if let Err(e) = self.reconcile_monitors().await {
                        error!("Monitor reconciliation failed: {}", e);
                    }
                }
                _ = self.shutdown.notified() => {
                    info!("Reconciliation loop received shutdown signal");
                    break;
                }
            }
        }

        info!("Reconciliation loop stopped");
    }

    /// Reconcile monitor daemon state with database.
    ///
    /// Compares what should be running (capturing monitors in DB) with what is
    /// actually running (tracked daemons) and corrects any discrepancies.
    async fn reconcile_monitors(&self) -> AppResult<()> {
        if self.is_shutting_down() {
            return Ok(());
        }

        let db = match &self.db {
            Some(db) => db.clone(),
            None => return Ok(()), // No DB, nothing to reconcile
        };

        debug!("Running monitor reconciliation");

        // Query monitors from database
        let monitor_list = monitors::Entity::find()
            .filter(monitors::Column::Deleted.eq(false))
            .all(db.as_ref())
            .await?;

        let mut started = 0;
        let mut stopped = 0;

        for monitor in &monitor_list {
            let monitor_id = monitor.id;
            let should_run = !matches!(monitor.capturing, Capturing::None)
                && !matches!(monitor.r#type, MonitorType::WebSite);

            // Check server_id filtering if configured
            let should_run = should_run && {
                if let Some(our_server_id) = self.server_id {
                    monitor
                        .server_id
                        .map(|sid| sid == our_server_id)
                        .unwrap_or(true)
                } else {
                    true
                }
            };

            let is_running = self.is_monitor_running(monitor_id).await;

            if should_run && !is_running {
                // Monitor should be running but isn't - start it
                debug!(
                    "Reconciliation: starting monitor {} (capturing but not running)",
                    monitor_id
                );
                match self.start_monitor(monitor_id).await {
                    Ok(resp) if resp.success => {
                        started += 1;
                        info!("Reconciliation: started monitor {}", monitor_id);
                    }
                    Ok(resp) => {
                        warn!(
                            "Reconciliation: failed to start monitor {}: {}",
                            monitor_id, resp.message
                        );
                    }
                    Err(e) => {
                        error!(
                            "Reconciliation: error starting monitor {}: {}",
                            monitor_id, e
                        );
                    }
                }
            } else if !should_run && is_running {
                // Monitor shouldn't be running but is - stop it
                debug!(
                    "Reconciliation: stopping monitor {} (not capturing but running)",
                    monitor_id
                );
                match self.stop_monitor(monitor_id).await {
                    Ok(resp) if resp.success => {
                        stopped += 1;
                        info!("Reconciliation: stopped monitor {}", monitor_id);
                    }
                    Ok(resp) => {
                        warn!(
                            "Reconciliation: failed to stop monitor {}: {}",
                            monitor_id, resp.message
                        );
                    }
                    Err(e) => {
                        error!(
                            "Reconciliation: error stopping monitor {}: {}",
                            monitor_id, e
                        );
                    }
                }
            }
        }

        if started > 0 || stopped > 0 {
            info!(
                "Reconciliation complete: started {} monitors, stopped {} monitors",
                started, stopped
            );
        } else {
            debug!("Reconciliation complete: no changes needed");
        }

        Ok(())
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
                // Poll for exit, then drop the child borrow before mutating the
                // process below. `try_wait()` is non-consuming and, once a
                // child has exited, tokio caches the status and returns it on
                // every subsequent call — so we MUST take the handle when we
                // first observe the exit, or each cycle would re-detect the
                // same dead child, re-run prepare_restart (resetting the
                // backoff clock) and never actually restart it.
                let wait_result = process.child_mut().map(|child| child.try_wait());

                match wait_result {
                    Some(Ok(Some(status))) => {
                        // Process exited — drop the dead handle so try_wait()
                        // can't re-fire on the cached status next cycle.
                        let _ = process.take_child();

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
                    Some(Ok(None)) => {
                        // Still running — once it has outlived the max backoff
                        // delay it counts as stable, so forgive its crash
                        // history and let a future restart start fresh instead
                        // of inheriting an old escalation.
                        process.reset_backoff_if_stable(
                            self.config.max_backoff(),
                            self.config.min_backoff(),
                        );
                    }
                    Some(Err(e)) => {
                        warn!("Error checking daemon {} status: {}", id, e);
                    }
                    None => {}
                }

                // Check for pending restarts. Pass only the "extras" — the
                // id-parts will be reconstructed by start_daemon, so passing
                // process.args directly would double them up.
                if process.state == ProcessState::Restarting && process.backoff_elapsed() {
                    let extras = extra_args_from_id(id, &process.args);
                    to_restart.push((id.clone(), extras));
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

    // =========================================================================
    // Monitor-specific daemon control
    // =========================================================================

    /// Start all daemons for a specific monitor.
    ///
    /// This starts the appropriate daemons based on the monitor's configuration:
    /// - `zmc -m {id}`: Capture daemon (always started)
    /// - `zma -m {id}`: Analysis daemon (if function is Modect/Mocord)
    ///
    /// # Arguments
    ///
    /// * `monitor_id` - The monitor ID to start daemons for
    ///
    /// # Returns
    ///
    /// A response indicating success/failure and details about what was started.
    /// Resolve the `zmc` daemon id for a monitor by looking up its type/device,
    /// so stop/status derive the same key `start` used. Falls back to the `-m`
    /// form when no DB is configured or the monitor row is missing.
    async fn zmc_id_for_monitor(&self, monitor_id: u32) -> String {
        if let Some(db) = &self.db {
            if let Ok(Some(m)) = monitors::Entity::find_by_id(monitor_id)
                .one(db.as_ref())
                .await
            {
                return zmc_daemon_id(&m.r#type, &m.device, monitor_id);
            }
        }
        format!("zmc -m {}", monitor_id)
    }

    /// Whether this monitor should run on a zm-next worker instead of legacy
    /// zmc/zma: requires zm-next to be enabled in config AND the per-monitor
    /// `UseZmNext` flag to be set. The flag read is isolated so the column's
    /// absence (the ZoneMinder fork has not added it yet) degrades to `false`.
    async fn use_zmnext(&self, monitor_id: u32) -> bool {
        if self.zmnext.is_none() {
            return false;
        }
        match &self.db {
            Some(db) => crate::repo::monitors::use_zmnext(db.as_ref(), monitor_id).await,
            None => false,
        }
    }

    /// Generate the monitor's pipeline JSON and (re)start its `zm-core` worker.
    /// The worker is keyed by the stable id `zm-core --monitor-id N`; the
    /// generated `--pipeline`/`--socket` paths ride as extra args so stop and
    /// restart can re-derive the id without recomputing them.
    async fn start_zmnext_worker(&self, monitor: &monitors::Model) -> AppResult<DaemonResponse> {
        let Some(rt) = self.zmnext.clone() else {
            return Ok(DaemonResponse::error("zm-next is not configured"));
        };
        let monitor_id = monitor.id;

        let pipeline_path = self.write_zmnext_pipeline(monitor, &rt).await?;
        let socket_path = format!(
            "{}/stream_{}.sock",
            rt.socks_path.trim_end_matches('/'),
            monitor_id
        );
        let id = zmnext_daemon_id(monitor_id);
        let extra = vec![
            "--pipeline".to_string(),
            pipeline_path.to_string_lossy().into_owned(),
            "--socket".to_string(),
            socket_path,
        ];
        self.start_daemon(&id, &extra).await
    }

    /// (Re)generate the monitor's pipeline JSON from its current Monitors/Zones
    /// rows and write it to the configured pipeline directory, returning the
    /// file path. This is the unit of "reload" for zm-next — the worker has no
    /// live config reload, so a restart re-reads the regenerated file.
    async fn write_zmnext_pipeline(
        &self,
        monitor: &monitors::Model,
        rt: &ZmNextRuntime,
    ) -> AppResult<PathBuf> {
        let monitor_id = monitor.id;
        let zones = match &self.db {
            Some(db) => {
                zones::Entity::find()
                    .filter(zones::Column::MonitorId.eq(monitor_id))
                    .all(db.as_ref())
                    .await?
            }
            None => Vec::new(),
        };
        let zone_specs = pipeline::zone_specs_from_models(&zones);
        let events_root = self.zmnext_events_root(monitor).await;
        let source_url = monitor.path.clone().unwrap_or_default();
        let mode = pipeline::StoreMode::from_function(&monitor.function);

        let synopsis = rt.synopsis_monitors.contains(&monitor_id);
        let value = pipeline::generate_pipeline(
            monitor_id,
            &source_url,
            &zone_specs,
            &rt.config.pipeline,
            mode,
            &events_root,
            synopsis,
        );
        pipeline::write_pipeline_file(&rt.config.pipeline.dir, monitor_id, &value).map_err(|e| {
            crate::error::AppError::InternalServerError(format!(
                "Failed to write zm-next pipeline for monitor {monitor_id}: {e}"
            ))
        })
    }

    /// Resolve the directory the worker's `store` plugin writes clips to:
    /// the monitor's storage path, else the default (lowest-id) storage, else a
    /// conventional fallback.
    async fn zmnext_events_root(&self, monitor: &monitors::Model) -> PathBuf {
        if let Some(db) = &self.db {
            let by_id = match monitor.storage_id {
                Some(sid) if sid != 0 => storage::Entity::find_by_id(sid)
                    .one(db.as_ref())
                    .await
                    .ok()
                    .flatten(),
                _ => None,
            };
            let resolved = match by_id {
                Some(s) => Some(s),
                None => storage::Entity::find()
                    .order_by_asc(storage::Column::Id)
                    .one(db.as_ref())
                    .await
                    .ok()
                    .flatten(),
            };
            if let Some(s) = resolved {
                if !s.path.is_empty() {
                    return PathBuf::from(s.path);
                }
            }
        }
        PathBuf::from("/var/lib/zm/events")
    }

    pub async fn start_monitor(&self, monitor_id: u32) -> AppResult<DaemonResponse> {
        let db = match &self.db {
            Some(db) => db.clone(),
            None => {
                return Ok(DaemonResponse::error(
                    "Database not configured - cannot query monitor",
                ));
            }
        };

        // Query monitor to determine what daemons to start
        let monitor = monitors::Entity::find_by_id(monitor_id)
            .one(db.as_ref())
            .await?
            .ok_or_else(|| {
                crate::error::AppError::NotFoundError(crate::error::Resource {
                    resource_type: crate::error::ResourceType::Monitor,
                    details: vec![("id".to_string(), monitor_id.to_string())],
                })
            })?;

        // zm-next monitors run a single worker instead of zmc/zma. Tear down
        // any legacy daemons first so flipping the flag swaps cleanly.
        if self.use_zmnext(monitor_id).await {
            let _ = self
                .stop_daemon(&zmc_daemon_id(&monitor.r#type, &monitor.device, monitor_id))
                .await;
            let _ = self.stop_daemon(&format!("zma -m {}", monitor_id)).await;
            return self.start_zmnext_worker(&monitor).await;
        }

        let mut started = Vec::new();
        let mut errors = Vec::new();

        // Not a zm-next monitor: tear down any stale worker before starting the
        // legacy daemons (handles flipping the flag back off).
        if self.zmnext.is_some() {
            let _ = self.stop_daemon(&zmnext_daemon_id(monitor_id)).await;
        }

        // Start zmc (capture daemon). Use the shared id derivation so Local
        // monitors get the same `-d <device>` key that stop/status expect.
        let zmc_id = zmc_daemon_id(&monitor.r#type, &monitor.device, monitor_id);
        match self.start_daemon(&zmc_id, &[]).await {
            Ok(resp) if resp.success => {
                started.push("zmc".to_string());
                info!("Started zmc for monitor {}", monitor_id);
            }
            Ok(resp) => {
                if !resp.message.contains("already running") {
                    errors.push(format!("zmc: {}", resp.message));
                }
            }
            Err(e) => {
                errors.push(format!("zmc: {}", e));
            }
        }

        // Start zma (analysis daemon) if needed for motion detection
        let needs_analysis = matches!(monitor.function, Function::Modect | Function::Mocord);
        if needs_analysis {
            let zma_id = format!("zma -m {}", monitor_id);
            match self.start_daemon(&zma_id, &[]).await {
                Ok(resp) if resp.success => {
                    started.push("zma".to_string());
                    info!("Started zma for monitor {}", monitor_id);
                }
                Ok(resp) => {
                    if !resp.message.contains("already running") {
                        errors.push(format!("zma: {}", resp.message));
                    }
                }
                Err(e) => {
                    errors.push(format!("zma: {}", e));
                }
            }
        }

        if errors.is_empty() {
            Ok(DaemonResponse::ok(format!(
                "Started monitor {}: {}",
                monitor_id,
                started.join(", ")
            )))
        } else if !started.is_empty() {
            Ok(DaemonResponse::ok(format!(
                "Partially started monitor {}: started [{}], errors [{}]",
                monitor_id,
                started.join(", "),
                errors.join("; ")
            )))
        } else {
            Ok(DaemonResponse::error(format!(
                "Failed to start monitor {}: {}",
                monitor_id,
                errors.join("; ")
            )))
        }
    }

    /// Stop all daemons for a specific monitor.
    ///
    /// This stops both zmc and zma daemons for the specified monitor.
    pub async fn stop_monitor(&self, monitor_id: u32) -> AppResult<DaemonResponse> {
        let mut stopped = Vec::new();
        let mut errors = Vec::new();

        // Stop zmc — resolve the same id form `start` used (Local monitors are
        // keyed by `-d <device>`).
        let zmc_id = self.zmc_id_for_monitor(monitor_id).await;
        match self.stop_daemon(&zmc_id).await {
            Ok(resp) if resp.success => {
                stopped.push("zmc".to_string());
                info!("Stopped zmc for monitor {}", monitor_id);
            }
            Ok(resp) => {
                if !resp.message.contains("not found") && !resp.message.contains("not running") {
                    errors.push(format!("zmc: {}", resp.message));
                }
            }
            Err(e) => {
                errors.push(format!("zmc: {}", e));
            }
        }

        // Stop zma
        let zma_id = format!("zma -m {}", monitor_id);
        match self.stop_daemon(&zma_id).await {
            Ok(resp) if resp.success => {
                stopped.push("zma".to_string());
                info!("Stopped zma for monitor {}", monitor_id);
            }
            Ok(resp) => {
                // zma might not be running if monitor doesn't need analysis
                if !resp.message.contains("not found") && !resp.message.contains("not running") {
                    errors.push(format!("zma: {}", resp.message));
                }
            }
            Err(e) => {
                errors.push(format!("zma: {}", e));
            }
        }

        // Stop any zm-next worker (no-op when the monitor is on legacy capture).
        if self.zmnext.is_some() {
            match self.stop_daemon(&zmnext_daemon_id(monitor_id)).await {
                Ok(resp) if resp.success => {
                    stopped.push("zm-core".to_string());
                    info!("Stopped zm-next worker for monitor {}", monitor_id);
                }
                Ok(resp) => {
                    if !resp.message.contains("not found") && !resp.message.contains("not running")
                    {
                        errors.push(format!("zm-core: {}", resp.message));
                    }
                }
                Err(e) => errors.push(format!("zm-core: {}", e)),
            }
        }

        if errors.is_empty() {
            Ok(DaemonResponse::ok(format!(
                "Stopped monitor {}: {}",
                monitor_id,
                if stopped.is_empty() {
                    "no daemons were running".to_string()
                } else {
                    stopped.join(", ")
                }
            )))
        } else {
            Ok(DaemonResponse::error(format!(
                "Errors stopping monitor {}: {}",
                monitor_id,
                errors.join("; ")
            )))
        }
    }

    /// Restart all daemons for a specific monitor.
    ///
    /// This stops then starts the monitor's daemons.
    pub async fn restart_monitor(&self, monitor_id: u32) -> AppResult<DaemonResponse> {
        info!("Restarting daemons for monitor {}", monitor_id);

        // Stop first
        let stop_result = self.stop_monitor(monitor_id).await?;
        if !stop_result.success {
            warn!(
                "Stop phase had issues for monitor {}: {}",
                monitor_id, stop_result.message
            );
        }

        // Brief delay to allow cleanup
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        // Start
        self.start_monitor(monitor_id).await
    }

    /// Check if a specific monitor's daemons are running.
    ///
    /// Returns true if the monitor's primary capture daemon is running — the
    /// `zm-core` worker for a zm-next monitor, otherwise `zmc`. Keying off the
    /// flag-appropriate daemon lets reconciliation swap them when the flag
    /// flips: the now-correct daemon reads as "not running", so `start_monitor`
    /// fires and tears down the other.
    pub async fn is_monitor_running(&self, monitor_id: u32) -> bool {
        let id = if self.use_zmnext(monitor_id).await {
            zmnext_daemon_id(monitor_id)
        } else {
            self.zmc_id_for_monitor(monitor_id).await
        };
        let processes = self.processes.read().await;
        processes
            .get(&id)
            .map(|process| process.is_running())
            .unwrap_or(false)
    }

    /// Get status of a specific monitor's daemons.
    pub async fn get_monitor_daemon_status(&self, monitor_id: u32) -> Vec<ProcessStatus> {
        let zmc_id = self.zmc_id_for_monitor(monitor_id).await;
        let processes = self.processes.read().await;
        let mut statuses = Vec::new();

        // Check zmc
        if let Some(p) = processes.get(&zmc_id) {
            statuses.push(ProcessStatus {
                id: p.id.clone(),
                name: p.name.clone(),
                state: p.state,
                pid: p.pid,
                uptime_seconds: p.uptime().map(|d| d.as_secs()),
                restart_count: p.restart_count,
                monitor_id: p.monitor_id,
            });
        }

        // Check zma
        let zma_id = format!("zma -m {}", monitor_id);
        if let Some(p) = processes.get(&zma_id) {
            statuses.push(ProcessStatus {
                id: p.id.clone(),
                name: p.name.clone(),
                state: p.state,
                pid: p.pid,
                uptime_seconds: p.uptime().map(|d| d.as_secs()),
                restart_count: p.restart_count,
                monitor_id: p.monitor_id,
            });
        }

        statuses
    }
}

/// Whitelist validator for daemon spawn requests.
///
/// `start_daemon` (and therefore every restart path) routes through this
/// before exec. The HTTP `/api/v3/daemons/{id}/...` endpoints and the
/// `/run/zm/zmdc.sock` IPC handler both forward untrusted strings into
/// `start_daemon`; without this check, `PathBuf::join` with an absolute path
/// (e.g. `"/bin/bash"`) would replace the daemon base and run anything.
///
/// Mirrors `zmdc.pl`'s hardcoded daemon-name regex (line 131-138) plus
/// `zmpkg.pl`'s `^/dev/[\w/.\-]+$` device path check (line 214).
fn validate_daemon_spec(command: &str, args: &[String]) -> Result<(), String> {
    use regex::Regex;
    use std::sync::OnceLock;

    static DEVICE_RE: OnceLock<Regex> = OnceLock::new();
    let device_re = DEVICE_RE.get_or_init(|| Regex::new(r"^/dev/[\w/.\-]+$").unwrap());
    static FILTER_ID_RE: OnceLock<Regex> = OnceLock::new();
    let filter_id_re = FILTER_ID_RE.get_or_init(|| Regex::new(r"^--filter_id=\d+$").unwrap());

    let int_arg = |s: &String| s.parse::<u32>().is_ok();

    let result: Result<(), String> = match command {
        // Singletons that take no arguments.
        "zmstats.pl" | "zmtelemetry.pl" | "zmtrigger.pl" | "zmeventnotification.pl" => {
            if args.is_empty() {
                Ok(())
            } else {
                Err(format!("{} takes no arguments", command))
            }
        }
        // zmfilter: global `--daemon`, or per-filter `--filter_id=N --daemon`.
        "zmfilter.pl" => match args {
            [a] if a == "--daemon" => Ok(()),
            [a, b] if filter_id_re.is_match(a) && b == "--daemon" => Ok(()),
            _ => Err(format!("zmfilter.pl args invalid: {:?}", args)),
        },
        // zmaudit: -c / --continuous (upstream's two spellings of the same flag).
        "zmaudit.pl" => match args {
            [a] if a == "-c" || a == "--continuous" => Ok(()),
            _ => Err(format!("zmaudit.pl args invalid: {:?}", args)),
        },
        // zmc: -m <monitor_id> or -d /dev/<safe-path>. The regex matches
        // upstream's `/^\/dev\/[\w\/.\-]+$/` but we additionally reject `..`
        // so the path can't traverse out of /dev.
        "zmc" => match args {
            [m, id] if m == "-m" && int_arg(id) => Ok(()),
            [d, path] if d == "-d" && device_re.is_match(path) && !path.contains("..") => Ok(()),
            _ => Err(format!("zmc args invalid: {:?}", args)),
        },
        "zma" => match args {
            [m, id] if m == "-m" && int_arg(id) => Ok(()),
            _ => Err(format!("zma args invalid: {:?}", args)),
        },
        "zmcontrol.pl" => match args {
            [k, id] if k == "--id" && int_arg(id) => Ok(()),
            _ => Err(format!("zmcontrol.pl args invalid: {:?}", args)),
        },
        "zmtrack.pl" => match args {
            [m, id] if m == "-m" && int_arg(id) => Ok(()),
            _ => Err(format!("zmtrack.pl args invalid: {:?}", args)),
        },
        // zm-next worker: `--monitor-id <int> --pipeline <abs path> --socket
        // <abs path>` in any order. zm-api generates these paths itself; the
        // validator still enforces them as absolute and traversal-free so a
        // forwarded IPC/HTTP request can't smuggle arbitrary args.
        "zm-core" => validate_zmnext_args(args),
        _ => Err(format!("unknown daemon: {:?}", command)),
    };
    result
}

/// Validate the `zm-core` worker arg list (order-independent). Requires exactly
/// `--monitor-id`, `--pipeline` and `--socket`, each with a valid value.
fn validate_zmnext_args(args: &[String]) -> Result<(), String> {
    let safe_path = |p: &str| p.starts_with('/') && !p.contains("..");
    let (mut have_monitor, mut have_pipeline, mut have_socket) = (false, false, false);
    let mut chunks = args.chunks_exact(2);
    for pair in chunks.by_ref() {
        match (pair[0].as_str(), pair[1].as_str()) {
            ("--monitor-id", v) if v.parse::<u32>().is_ok() => have_monitor = true,
            ("--pipeline", v) if safe_path(v) => have_pipeline = true,
            ("--socket", v) if safe_path(v) => have_socket = true,
            _ => return Err(format!("zm-core args invalid: {:?}", args)),
        }
    }
    if !chunks.remainder().is_empty() {
        return Err(format!("zm-core args must be flag/value pairs: {:?}", args));
    }
    if have_monitor && have_pipeline && have_socket {
        Ok(())
    } else {
        Err(format!("zm-core missing required args: {:?}", args))
    }
}

/// Compute the "extras" — args present on a `ManagedProcess` that are NOT
/// already encoded in the daemon id. `parse_daemon_command` will re-derive
/// the id-parts on the next spawn and concatenate, so we have to subtract
/// them on the way back in or they get duplicated each restart.
fn extra_args_from_id(id: &str, full_args: &[String]) -> Vec<String> {
    let id_arg_count = id.split_whitespace().count().saturating_sub(1);
    full_args.iter().skip(id_arg_count).cloned().collect()
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

/// Build the `zmc` capture-daemon id for a monitor. This id doubles as the key
/// in the process map, so **every** call site (start / stop / restart / status)
/// must derive it identically — otherwise a daemon started under one form is
/// invisible to the others.
///
/// Local (V4L2) monitors with a device path are addressed as `zmc -d <device>`
/// to match how ZoneMinder launches local capture; everything else uses
/// `zmc -m <id>`. Previously `start_all_daemons` used the `-d` form while
/// stop/status hard-coded `-m`, so Local monitors were started, never found
/// again, and churn-restarted on every reconcile tick. See REVIEW_FIXES_PLAN §4.1.
///
/// A device containing whitespace falls back to the `-m` form: the id is later
/// re-split on whitespace by [`parse_daemon_command`], so a spaced path would
/// fan out into bogus args and be rejected by `validate_daemon_spec` anyway.
fn zmc_daemon_id(monitor_type: &MonitorType, device: &str, monitor_id: u32) -> String {
    if matches!(monitor_type, MonitorType::Local) && !device.is_empty() {
        if device.chars().any(char::is_whitespace) {
            warn!(
                "Monitor {} device path {:?} contains whitespace; using -m form \
                 (a spaced device would fail daemon-spec validation)",
                monitor_id, device
            );
        } else {
            return format!("zmc -d {}", device);
        }
    }
    format!("zmc -m {}", monitor_id)
}

/// Interpret a ZoneMinder Config `Value` as a boolean.
///
/// ZoneMinder stores booleans as text in the `Config` table, mostly as `"1"`/`"0"`
/// but human edits and older rows can produce `"yes"`/`"no"` or `"true"`/`"false"`.
fn parse_zm_bool(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

/// Read a `ZM_*` config key and interpret it as a boolean.
///
/// Returns false on missing key, parse failure, or DB error — matching upstream
/// Perl truthiness: `if ($Config{KEY})` treats undef/0/empty as false.
async fn read_bool_config(db: &DatabaseConnection, key: &str) -> bool {
    match crate::repo::config::get_config_value(db, key).await {
        Ok(Some(v)) => parse_zm_bool(&v),
        Ok(None) => false,
        Err(e) => {
            warn!("Could not read {}: {} (defaulting to off)", key, e);
            false
        }
    }
}

/// Snapshot of upstream-equivalent startup gates pulled from `Config` and
/// (when multi-server) the current `Servers` row.
///
/// `server_*` fields default to `true` when not in multi-server mode, so
/// single-server installs ignore the per-server columns entirely — matching
/// `zmpkg.pl`'s `if ($Server and exists $$Server{x} and !$$Server{x})` check.
struct StartupGates {
    zm_opt_control: bool,
    zm_opt_triggers: bool,
    zm_opt_use_eventnotification: bool,
    zm_telemetry_data: bool,
    zm_run_audit: bool,
    server_zmstats: bool,
    server_zmaudit: bool,
    server_zmtrigger: bool,
    server_zmeventnotification: bool,
}

/// Extract monitor ID from daemon arguments.
fn extract_monitor_id(args: &[String]) -> Option<u32> {
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        // `-m N` (zmc/zma) or `--monitor-id N` (zm-core worker).
        if arg == "-m" || arg == "--monitor-id" {
            if let Some(id_str) = iter.next() {
                return id_str.parse().ok();
            }
        }
    }
    None
}

/// Stable process-map id for a monitor's zm-next worker. The generated
/// `--pipeline`/`--socket` paths ride as extra args, not in the id, so every
/// call site derives the same key without recomputing those paths.
fn zmnext_daemon_id(monitor_id: u32) -> String {
    format!("zm-core --monitor-id {monitor_id}")
}

/// Recover the monitor id from a `zm-core --monitor-id N` daemon id, or `None`
/// when `id` is not a zm-next worker id.
fn zmnext_monitor_id_from_id(id: &str) -> Option<u32> {
    let parts: Vec<&str> = id.split_whitespace().collect();
    match parts.as_slice() {
        ["zm-core", "--monitor-id", n] => n.parse().ok(),
        _ => None,
    }
}

/// Spawn a daemon process with PR_SET_PDEATHSIG on Linux.
///
/// This ensures that child processes receive SIGTERM when the parent process dies,
/// preventing orphaned daemons when zm_api crashes or is killed.
#[cfg(target_os = "linux")]
fn spawn_daemon(
    path: &std::path::Path,
    args: &[String],
) -> Result<tokio::process::Child, std::io::Error> {
    let mut cmd = Command::new(path);
    cmd.args(args);

    // SAFETY: prctl is async-signal-safe and we're only setting PR_SET_PDEATHSIG
    // which is a simple flag operation with no memory allocation or locks.
    unsafe {
        cmd.pre_exec(|| {
            // PR_SET_PDEATHSIG causes the child to receive the specified signal
            // when its parent process terminates.
            if libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGTERM) == -1 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }

    cmd.spawn()
}

/// Spawn a daemon process (non-Linux fallback).
#[cfg(not(target_os = "linux"))]
fn spawn_daemon(
    path: &std::path::Path,
    args: &[String],
) -> Result<tokio::process::Child, std::io::Error> {
    Command::new(path).args(args).spawn()
}

/// Kill orphaned ZoneMinder daemons that may be left over from a crash.
///
/// This should be called on startup before starting new daemons to ensure
/// a clean slate. Orphaned processes can cause shared memory conflicts
/// and resource contention.
pub async fn kill_orphan_daemons() {
    let daemon_names = [
        "zmc",
        "zma",
        "zmfilter.pl",
        "zmstats.pl",
        "zmtrack.pl",
        "zmcontrol.pl",
    ];

    for daemon in &daemon_names {
        match Command::new("pkill").args(["-9", daemon]).output().await {
            Ok(output) => {
                if output.status.success() {
                    info!("Killed orphaned {} processes", daemon);
                }
                // Exit code 1 means no processes matched - that's fine
            }
            Err(e) => {
                // pkill might not be available, try killall as fallback
                debug!("pkill failed for {}: {}, trying killall", daemon, e);
                if let Err(e2) = Command::new("killall").args(["-9", daemon]).output().await {
                    debug!("killall also failed for {}: {}", daemon, e2);
                }
            }
        }
    }

    // Brief pause to allow processes to terminate and release resources
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    info!("Orphan daemon cleanup complete");
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
    fn zmc_daemon_id_local_with_device_uses_dash_d() {
        assert_eq!(
            zmc_daemon_id(&MonitorType::Local, "/dev/video0", 7),
            "zmc -d /dev/video0"
        );
    }

    #[test]
    fn zmc_daemon_id_local_without_device_uses_dash_m() {
        assert_eq!(zmc_daemon_id(&MonitorType::Local, "", 7), "zmc -m 7");
    }

    #[test]
    fn zmc_daemon_id_non_local_uses_dash_m() {
        // Network camera (Ffmpeg type): always -m, even if a device is set.
        assert_eq!(
            zmc_daemon_id(&MonitorType::Ffmpeg, "/dev/video0", 12),
            "zmc -m 12"
        );
    }

    #[test]
    fn zmc_daemon_id_whitespace_device_falls_back_to_dash_m() {
        // A spaced device would be re-split by parse_daemon_command and rejected
        // by validate_daemon_spec, so we never emit the -d form for it.
        assert_eq!(
            zmc_daemon_id(&MonitorType::Local, "/dev/video 0", 3),
            "zmc -m 3"
        );
    }

    #[test]
    fn zmc_daemon_id_roundtrips_through_validation() {
        // The id `start` produces must parse + validate (so stop/status, which
        // reconstruct from the same id, agree).
        let id = zmc_daemon_id(&MonitorType::Local, "/dev/video0", 1);
        let (cmd, parsed) = parse_daemon_command(&id, &[]);
        assert_eq!(cmd, "zmc");
        assert!(validate_daemon_spec(&cmd, &parsed).is_ok());
    }

    fn args(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn test_validate_daemon_spec_accepts_known_shapes() {
        // Singletons
        assert!(validate_daemon_spec("zmstats.pl", &[]).is_ok());
        assert!(validate_daemon_spec("zmtelemetry.pl", &[]).is_ok());
        assert!(validate_daemon_spec("zmtrigger.pl", &[]).is_ok());
        assert!(validate_daemon_spec("zmeventnotification.pl", &[]).is_ok());

        // zmfilter both shapes
        assert!(validate_daemon_spec("zmfilter.pl", &args(&["--daemon"])).is_ok());
        assert!(validate_daemon_spec("zmfilter.pl", &args(&["--filter_id=7", "--daemon"])).is_ok());

        // zmaudit both spellings
        assert!(validate_daemon_spec("zmaudit.pl", &args(&["-c"])).is_ok());
        assert!(validate_daemon_spec("zmaudit.pl", &args(&["--continuous"])).is_ok());

        // Per-monitor
        assert!(validate_daemon_spec("zmc", &args(&["-m", "3"])).is_ok());
        assert!(validate_daemon_spec("zmc", &args(&["-d", "/dev/video0"])).is_ok());
        assert!(validate_daemon_spec("zma", &args(&["-m", "3"])).is_ok());
        assert!(validate_daemon_spec("zmcontrol.pl", &args(&["--id", "3"])).is_ok());
        assert!(validate_daemon_spec("zmtrack.pl", &args(&["-m", "3"])).is_ok());
    }

    #[test]
    fn test_validate_zmnext_worker_spec() {
        // Valid, order-independent.
        assert!(validate_daemon_spec(
            "zm-core",
            &args(&[
                "--monitor-id",
                "7",
                "--pipeline",
                "/var/lib/zm_api/pipelines/monitor_7.json",
                "--socket",
                "/run/zm/stream_7.sock",
            ])
        )
        .is_ok());
        assert!(validate_daemon_spec(
            "zm-core",
            &args(&[
                "--socket",
                "/run/zm/stream_7.sock",
                "--pipeline",
                "/p/monitor_7.json",
                "--monitor-id",
                "7",
            ])
        )
        .is_ok());

        // Path traversal and relative paths are rejected.
        assert!(validate_daemon_spec(
            "zm-core",
            &args(&[
                "--monitor-id",
                "7",
                "--pipeline",
                "/p/../../etc/passwd",
                "--socket",
                "/run/zm/stream_7.sock",
            ])
        )
        .is_err());
        assert!(validate_daemon_spec(
            "zm-core",
            &args(&[
                "--monitor-id",
                "7",
                "--pipeline",
                "relative.json",
                "--socket",
                "/s.sock"
            ])
        )
        .is_err());

        // Missing a required flag, non-numeric id, and odd arg count all fail.
        assert!(validate_daemon_spec(
            "zm-core",
            &args(&["--monitor-id", "7", "--pipeline", "/p.json"])
        )
        .is_err());
        assert!(validate_daemon_spec(
            "zm-core",
            &args(&[
                "--monitor-id",
                "x",
                "--pipeline",
                "/p.json",
                "--socket",
                "/s.sock"
            ])
        )
        .is_err());
        assert!(validate_daemon_spec("zm-core", &args(&["--monitor-id"])).is_err());
    }

    #[test]
    fn test_zmnext_daemon_id_round_trips() {
        let id = zmnext_daemon_id(42);
        assert_eq!(id, "zm-core --monitor-id 42");
        assert_eq!(zmnext_monitor_id_from_id(&id), Some(42));
        // Non-worker ids are rejected.
        assert_eq!(zmnext_monitor_id_from_id("zmc -m 42"), None);
        assert_eq!(zmnext_monitor_id_from_id("zm-core -m 42"), None);
        // The worker id parses back to the same exec arg list.
        let (command, parsed) = parse_daemon_command(&id, &[]);
        assert_eq!(command, "zm-core");
        assert_eq!(parsed, vec!["--monitor-id".to_string(), "42".to_string()]);
    }

    #[test]
    fn test_validate_daemon_spec_rejects_injection_attempts() {
        // Absolute paths are the main attack vector against PathBuf::join
        assert!(validate_daemon_spec("/bin/bash", &args(&["-c", "id"])).is_err());
        assert!(validate_daemon_spec("/usr/bin/rm", &args(&["-rf", "/"])).is_err());

        // Common system tools that aren't ZM daemons
        assert!(validate_daemon_spec("rm", &args(&["-rf", "/home"])).is_err());
        assert!(validate_daemon_spec("sh", &args(&["-c", "echo pwned"])).is_err());
        assert!(validate_daemon_spec("python3", &[]).is_err());

        // Known daemon, wrong args
        assert!(validate_daemon_spec("zmc", &args(&["-x", "5"])).is_err());
        assert!(validate_daemon_spec("zmc", &args(&["-m", "not_a_number"])).is_err());
        assert!(validate_daemon_spec("zmc", &args(&["-m"])).is_err());
        assert!(validate_daemon_spec("zmstats.pl", &args(&["extra"])).is_err());

        // Bogus device paths (upstream regex blocks anything outside /dev/...)
        assert!(validate_daemon_spec("zmc", &args(&["-d", "/etc/passwd"])).is_err());
        assert!(validate_daemon_spec("zmc", &args(&["-d", "/dev/../etc/passwd"])).is_err());
        assert!(validate_daemon_spec("zmc", &args(&["-d", "/dev/video0; rm -rf /"])).is_err());

        // Filter id must be a number
        assert!(
            validate_daemon_spec("zmfilter.pl", &args(&["--filter_id=evil", "--daemon"])).is_err()
        );
    }

    #[tokio::test]
    async fn test_start_daemon_rejects_invalid_id() {
        // End-to-end: start_daemon refuses arbitrary executables. We use an
        // ID that would otherwise resolve and (potentially) spawn.
        let config = DaemonConfig::default();
        let manager = DaemonManager::new(config, None);
        let resp = manager
            .start_daemon("/bin/sh -c whoami", &[])
            .await
            .unwrap();
        assert!(!resp.success, "expected start_daemon to reject /bin/sh");
        assert!(
            resp.message.contains("Invalid daemon spec"),
            "got: {}",
            resp.message
        );
    }

    #[test]
    fn test_extra_args_from_id() {
        // Plain command id, args are all extras
        assert_eq!(
            extra_args_from_id("zmfilter.pl", &["--daemon".to_string()]),
            vec!["--daemon".to_string()]
        );

        // id encodes the args; nothing extra
        let args = vec!["-m".to_string(), "2".to_string()];
        assert!(extra_args_from_id("zmc -m 2", &args).is_empty());

        // id encodes some, plus an extra
        let args = vec!["--filter_id=5".to_string(), "--daemon".to_string()];
        assert_eq!(
            extra_args_from_id("zmfilter.pl --filter_id=5", &args),
            vec!["--daemon".to_string()]
        );

        // Empty inputs
        assert!(extra_args_from_id("", &[]).is_empty());
        assert!(extra_args_from_id("zmc -m 2", &[]).is_empty());
    }

    #[test]
    fn test_parse_zm_bool() {
        // Truthy values ZoneMinder may write
        for v in ["1", "true", "TRUE", "yes", "Yes", "on", "ON", " 1 "] {
            assert!(parse_zm_bool(v), "expected `{v}` to be true");
        }
        // Falsy / unrecognized values default to false (matches the
        // conservative "off if unsure" behavior of the auto-start gate)
        for v in ["0", "false", "no", "off", "", "garbage", "2"] {
            assert!(!parse_zm_bool(v), "expected `{v}` to be false");
        }
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
        // The zm-core worker uses the long flag form.
        assert_eq!(
            extract_monitor_id(&[
                "--monitor-id".to_string(),
                "9".to_string(),
                "--pipeline".to_string(),
                "/p.json".to_string(),
            ]),
            Some(9)
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

    /// Regression: a daemon that exits once must be handled exactly once.
    ///
    /// The dead `Child` handle used to be left in place after exit detection.
    /// tokio's `try_wait()` caches the exit status and returns it on every
    /// subsequent call, so each health-check cycle re-entered the exit branch,
    /// re-ran `prepare_restart` (which resets the backoff clock via
    /// `set_state`) and therefore never let the backoff elapse — the daemon
    /// stayed dead while the log filled with a phantom "exited" line every
    /// interval. This is what stopped `zmfilter.pl` (PurgeWhenFull) from ever
    /// being restarted and let the disk fill.
    #[tokio::test]
    async fn test_exited_daemon_handled_once_not_re_detected_each_cycle() {
        let config = DaemonConfig::default();
        let manager = DaemonManager::new(config, None);

        // A real child process that has already exited and been reaped.
        let mut child = Command::new("sh")
            .arg("-c")
            .arg("exit 0")
            .spawn()
            .expect("spawn test child");
        let _ = child.wait().await;

        let mut process = ManagedProcess::new("test-daemon", "Test", "test", vec![], true, None);
        process.set_child(child);
        manager.register_daemon(process).await;

        // Several cycles. The default min backoff (5s) has not elapsed, so no
        // real restart is attempted within the test window.
        manager.check_daemons().await;
        manager.check_daemons().await;
        manager.check_daemons().await;

        let processes = manager.processes.read().await;
        let process = processes.get("test-daemon").expect("process present");
        assert_eq!(
            process.restart_count, 1,
            "exit must be handled once, not re-detected every cycle"
        );
        assert_eq!(process.state, ProcessState::Restarting);
        assert!(
            !process.has_child(),
            "dead child handle must be cleared so try_wait() cannot re-fire"
        );
    }

    /// Regression: once shutdown is signaled, `start_daemon` must refuse so a
    /// reconcile/health restart can't orphan a daemon that misses the stop wave
    /// (which previously hung shutdown until systemd SIGKILL'd the unit).
    #[tokio::test]
    async fn test_no_daemon_starts_while_shutting_down() {
        let config = DaemonConfig::default();
        let manager = DaemonManager::new(config, None);

        assert!(!manager.is_shutting_down());
        manager.signal_shutdown();
        assert!(manager.is_shutting_down());

        // A would-be reconcile/health restart must be refused before it can
        // resolve a path or fork a process.
        let resp = manager
            .start_daemon("zmc", &["-m".to_string(), "1".to_string()])
            .await
            .unwrap();

        assert!(
            !resp.success,
            "start must be refused while shutting down to avoid orphaned daemons"
        );
        assert!(
            manager.list_daemon_ids().await.is_empty(),
            "no process entry should be created for a refused start"
        );
    }
}
