//! Unix domain socket server for legacy IPC compatibility.

use std::path::PathBuf;
use std::sync::Arc;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::Notify;
use tracing::{debug, error, info, warn};

use crate::daemon::commands::{format_response, parse_command};
use crate::daemon::ipc::{DaemonCommand, DaemonResponse};
use crate::daemon::manager::DaemonManager;
use crate::error::AppResult;

/// Unix domain socket server for the daemon controller.
pub struct DaemonSocketServer {
    /// Path to the socket file
    socket_path: PathBuf,
    /// Reference to the daemon manager
    manager: Arc<DaemonManager>,
    /// Shutdown signal
    shutdown: Arc<Notify>,
}

impl DaemonSocketServer {
    /// Create a new socket server.
    pub fn new(socket_path: PathBuf, manager: Arc<DaemonManager>) -> Self {
        Self {
            socket_path,
            manager,
            shutdown: Arc::new(Notify::new()),
        }
    }

    /// Get the socket path.
    pub fn socket_path(&self) -> &PathBuf {
        &self.socket_path
    }

    /// Signal the server to shutdown.
    pub fn signal_shutdown(&self) {
        self.shutdown.notify_waiters();
    }

    /// Start the socket server.
    ///
    /// This runs until shutdown is signaled.
    pub async fn run(&self) -> AppResult<()> {
        // Remove existing socket file if present
        if self.socket_path.exists() {
            std::fs::remove_file(&self.socket_path)?;
        }

        // Ensure parent directory exists
        if let Some(parent) = self.socket_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Bind to the socket
        let listener = UnixListener::bind(&self.socket_path)?;
        info!("Daemon socket server listening on {:?}", self.socket_path);

        // Set socket permissions (world-readable/writable for compatibility)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o777);
            std::fs::set_permissions(&self.socket_path, perms)?;
        }

        loop {
            tokio::select! {
                result = listener.accept() => {
                    match result {
                        Ok((stream, _addr)) => {
                            let manager = Arc::clone(&self.manager);
                            tokio::spawn(async move {
                                if let Err(e) = handle_client(stream, manager).await {
                                    warn!("Error handling client: {}", e);
                                }
                            });
                        }
                        Err(e) => {
                            error!("Error accepting connection: {}", e);
                        }
                    }
                }
                _ = self.shutdown.notified() => {
                    info!("Socket server shutting down");
                    break;
                }
            }
        }

        // Clean up socket file
        if self.socket_path.exists() {
            let _ = std::fs::remove_file(&self.socket_path);
        }

        Ok(())
    }
}

/// Handle a single client connection.
async fn handle_client(stream: UnixStream, manager: Arc<DaemonManager>) -> AppResult<()> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    // Read a single command line
    match reader.read_line(&mut line).await {
        Ok(0) => {
            debug!("Client disconnected");
            return Ok(());
        }
        Ok(_) => {}
        Err(e) => {
            warn!("Error reading from client: {}", e);
            return Err(e.into());
        }
    }

    let line = line.trim();
    debug!("Received command: {}", line);

    // Detect if client wants JSON response
    let json_format = line.starts_with('{');

    // Parse the command
    let response = match parse_command(line) {
        Ok(cmd) => execute_command(cmd, &manager).await,
        Err(e) => DaemonResponse::error(e),
    };

    // Send response
    let response_text = format_response(&response, json_format);
    writer.write_all(response_text.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;

    Ok(())
}

/// Execute a daemon command and return a response.
async fn execute_command(cmd: DaemonCommand, manager: &Arc<DaemonManager>) -> DaemonResponse {
    match cmd {
        DaemonCommand::Startup => match manager.start_all_daemons().await {
            Ok(resp) => resp,
            Err(e) => DaemonResponse::error(e.to_string()),
        },
        DaemonCommand::Shutdown => match manager.shutdown_all().await {
            Ok(resp) => resp,
            Err(e) => DaemonResponse::error(e.to_string()),
        },
        DaemonCommand::Status => {
            let status = manager.get_status().await;
            DaemonResponse::ok_with_data("Status retrieved", status)
        }
        DaemonCommand::Check => {
            if manager.is_running().await {
                DaemonResponse::ok("running")
            } else {
                DaemonResponse::ok("stopped")
            }
        }
        DaemonCommand::LogRot => {
            // Send SIGHUP to all running daemons
            let ids = manager.list_daemon_ids().await;
            let mut count = 0;
            for id in ids {
                if let Ok(resp) = manager.reload_daemon(&id).await {
                    if resp.success {
                        count += 1;
                    }
                }
            }
            DaemonResponse::ok(format!("Log rotation signal sent to {} daemons", count))
        }
        DaemonCommand::Version => DaemonResponse::ok(env!("CARGO_PKG_VERSION")),
        DaemonCommand::Start { daemon, args } => match manager.start_daemon(&daemon, &args).await {
            Ok(resp) => resp,
            Err(e) => DaemonResponse::error(e.to_string()),
        },
        DaemonCommand::Stop { daemon } => match manager.stop_daemon(&daemon).await {
            Ok(resp) => resp,
            Err(e) => DaemonResponse::error(e.to_string()),
        },
        DaemonCommand::Restart { daemon, args } => {
            match manager.restart_daemon(&daemon, &args).await {
                Ok(resp) => resp,
                Err(e) => DaemonResponse::error(e.to_string()),
            }
        }
        DaemonCommand::Reload { daemon } => match manager.reload_daemon(&daemon).await {
            Ok(resp) => resp,
            Err(e) => DaemonResponse::error(e.to_string()),
        },
        DaemonCommand::PackageStart => {
            // Full system startup - starts all daemons
            match manager.start_all_daemons().await {
                Ok(resp) => resp,
                Err(e) => DaemonResponse::error(e.to_string()),
            }
        }
        DaemonCommand::PackageStop => match manager.shutdown_all().await {
            Ok(resp) => resp,
            Err(e) => DaemonResponse::error(e.to_string()),
        },
        DaemonCommand::PackageRestart => {
            // Stop then start all daemons
            let _ = manager.shutdown_all().await;
            // Small delay to let processes terminate
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            match manager.start_all_daemons().await {
                Ok(resp) => resp,
                Err(e) => DaemonResponse::error(e.to_string()),
            }
        }
        DaemonCommand::ApplyState { state_name } => {
            // State application would query DB for state definition
            // and start/stop monitors accordingly
            DaemonResponse::ok(format!("State '{}' applied (stub)", state_name))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::daemon::config::DaemonConfig;

    #[tokio::test]
    async fn test_execute_command_check() {
        let config = DaemonConfig::default();
        let manager = Arc::new(DaemonManager::new(config, None));

        let resp = execute_command(DaemonCommand::Check, &manager).await;
        assert!(resp.success);
        assert_eq!(resp.message, "stopped");
    }

    #[tokio::test]
    async fn test_execute_command_version() {
        let config = DaemonConfig::default();
        let manager = Arc::new(DaemonManager::new(config, None));

        let resp = execute_command(DaemonCommand::Version, &manager).await;
        assert!(resp.success);
        assert!(!resp.message.is_empty());
    }

    #[tokio::test]
    async fn test_execute_command_startup_without_db() {
        // Without database, startup should fail with informative error
        let config = DaemonConfig::default();
        let manager = Arc::new(DaemonManager::new(config, None));

        let resp = execute_command(DaemonCommand::Startup, &manager).await;
        // Should fail because no database is configured
        assert!(!resp.success);
        assert!(resp.message.contains("Database not configured"));
    }

    #[tokio::test]
    async fn test_execute_command_status() {
        let config = DaemonConfig::default();
        let manager = Arc::new(DaemonManager::new(config, None));

        let resp = execute_command(DaemonCommand::Status, &manager).await;
        assert!(resp.success);
        assert!(resp.data.is_some());
    }
}
