//! Daemon controller service layer.

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};
use tracing::{debug, error, info, warn};

use crate::daemon::daemons::DaemonDefinition;
use crate::daemon::ipc::DaemonResponse;
use crate::dto::response::daemon::{
    DaemonActionResponse, DaemonListResponse, DaemonStatusResponse, SystemStatusResponse,
};
use crate::entity::monitors;
use crate::entity::sea_orm_active_enums::Function;
use crate::error::{AppError, AppResult};
use crate::server::state::AppState;

/// Get the status of all daemons.
pub async fn get_all_daemons(state: &AppState) -> AppResult<DaemonListResponse> {
    let manager = state
        .daemon_manager
        .as_ref()
        .ok_or_else(|| AppError::ServiceUnavailableError("Daemon manager not available".into()))?;

    let status = manager.get_status().await;
    Ok(DaemonListResponse {
        daemons: status.daemons.into_iter().map(Into::into).collect(),
    })
}

/// Get the status of a specific daemon.
pub async fn get_daemon(state: &AppState, id: &str) -> AppResult<DaemonStatusResponse> {
    let manager = state
        .daemon_manager
        .as_ref()
        .ok_or_else(|| AppError::ServiceUnavailableError("Daemon manager not available".into()))?;

    manager
        .get_daemon_status(id)
        .await
        .map(Into::into)
        .ok_or_else(|| {
            AppError::NotFoundError(crate::error::Resource {
                resource_type: crate::error::ResourceType::Config,
                details: vec![("daemon_id".to_string(), id.to_string())],
            })
        })
}

/// Start a daemon.
pub async fn start_daemon(
    state: &AppState,
    id: &str,
    args: &[String],
) -> AppResult<DaemonActionResponse> {
    let manager = state
        .daemon_manager
        .as_ref()
        .ok_or_else(|| AppError::ServiceUnavailableError("Daemon manager not available".into()))?;

    info!("Starting daemon: {} with args: {:?}", id, args);
    let resp = manager.start_daemon(id, args).await?;
    Ok(response_to_action(resp))
}

/// Stop a daemon.
pub async fn stop_daemon(state: &AppState, id: &str) -> AppResult<DaemonActionResponse> {
    let manager = state
        .daemon_manager
        .as_ref()
        .ok_or_else(|| AppError::ServiceUnavailableError("Daemon manager not available".into()))?;

    info!("Stopping daemon: {}", id);
    let resp = manager.stop_daemon(id).await?;
    Ok(response_to_action(resp))
}

/// Restart a daemon.
pub async fn restart_daemon(state: &AppState, id: &str) -> AppResult<DaemonActionResponse> {
    let manager = state
        .daemon_manager
        .as_ref()
        .ok_or_else(|| AppError::ServiceUnavailableError("Daemon manager not available".into()))?;

    info!("Restarting daemon: {}", id);
    let resp = manager.restart_daemon(id).await?;
    Ok(response_to_action(resp))
}

/// Reload a daemon (SIGHUP).
pub async fn reload_daemon(state: &AppState, id: &str) -> AppResult<DaemonActionResponse> {
    let manager = state
        .daemon_manager
        .as_ref()
        .ok_or_else(|| AppError::ServiceUnavailableError("Daemon manager not available".into()))?;

    info!("Reloading daemon: {}", id);
    let resp = manager.reload_daemon(id).await?;
    Ok(response_to_action(resp))
}

/// Get system status.
pub async fn get_system_status(state: &AppState) -> AppResult<SystemStatusResponse> {
    let manager = state
        .daemon_manager
        .as_ref()
        .ok_or_else(|| AppError::ServiceUnavailableError("Daemon manager not available".into()))?;

    let status = manager.get_status().await;
    Ok(status.into())
}

/// Perform full system startup.
///
/// This function starts all ZoneMinder daemons in the correct order:
/// 1. Per-monitor capture daemons (zmc) for enabled monitors
/// 2. Per-monitor analysis daemons (zma) for monitors requiring motion detection
/// 3. Singleton daemons (zmfilter.pl, zmwatch.pl, zmstats.pl, etc.)
pub async fn system_startup(state: &AppState) -> AppResult<DaemonActionResponse> {
    let manager = state
        .daemon_manager
        .as_ref()
        .ok_or_else(|| AppError::ServiceUnavailableError("Daemon manager not available".into()))?;

    info!("Starting ZoneMinder system");

    // Mark manager as running
    manager.startup().await?;

    let mut started = 0;
    let mut failed = 0;
    let mut errors: Vec<String> = Vec::new();

    // Query enabled monitors from database
    let monitors = monitors::Entity::find()
        .filter(monitors::Column::Enabled.eq(1_u8))
        .filter(monitors::Column::Deleted.eq(false))
        .all(state.db.as_ref())
        .await?;

    info!("Found {} enabled monitors", monitors.len());

    // Start per-monitor daemons based on Function
    for monitor in &monitors {
        let monitor_id = monitor.id;
        let function = &monitor.function;

        // Determine which daemons this monitor needs
        let needs_capture = needs_capture_daemon(function);
        let needs_analysis = needs_analysis_daemon(function);

        debug!(
            "Monitor {} ({}): function={:?}, capture={}, analysis={}",
            monitor_id, monitor.name, function, needs_capture, needs_analysis
        );

        // Start zmc (capture daemon) if needed
        if needs_capture {
            let daemon_id = format!("zmc -m {}", monitor_id);
            match manager.start_daemon(&daemon_id, &[]).await {
                Ok(resp) if resp.success => {
                    started += 1;
                    info!("Started zmc for monitor {}", monitor_id);
                }
                Ok(resp) => {
                    // If already running, that's ok
                    if !resp.message.contains("already running") {
                        failed += 1;
                        errors.push(format!("zmc -m {}: {}", monitor_id, resp.message));
                        warn!(
                            "Failed to start zmc for monitor {}: {}",
                            monitor_id, resp.message
                        );
                    }
                }
                Err(e) => {
                    failed += 1;
                    errors.push(format!("zmc -m {}: {}", monitor_id, e));
                    error!("Error starting zmc for monitor {}: {}", monitor_id, e);
                }
            }
        }

        // Start zma (analysis daemon) if needed
        if needs_analysis {
            let daemon_id = format!("zma -m {}", monitor_id);
            match manager.start_daemon(&daemon_id, &[]).await {
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
    }

    // Start singleton daemons in priority order
    let singletons: Vec<_> = DaemonDefinition::singletons()
        .filter(|d| d.requires_db) // Only start DB-dependent singletons
        .collect();

    // Sort by priority
    let mut singletons = singletons;
    singletons.sort_by_key(|d| d.priority);

    for daemon in singletons {
        debug!(
            "Starting singleton daemon: {} (priority {})",
            daemon.name, daemon.priority
        );

        match manager.start_daemon(daemon.command, &[]).await {
            Ok(resp) if resp.success => {
                started += 1;
                info!("Started {}", daemon.name);
            }
            Ok(resp) => {
                if !resp.message.contains("already running") {
                    // Some daemons may not exist on all systems, treat as warning
                    warn!("Could not start {}: {}", daemon.name, resp.message);
                }
            }
            Err(e) => {
                // Non-critical - some daemons may not be installed
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
        Ok(DaemonActionResponse::error(message))
    } else {
        Ok(DaemonActionResponse::success(message))
    }
}

/// Determine if a monitor needs a capture daemon (zmc).
///
/// All functions except None need capture.
fn needs_capture_daemon(function: &Function) -> bool {
    !matches!(function, Function::None)
}

/// Determine if a monitor needs an analysis daemon (zma).
///
/// Modect and Mocord need analysis for motion detection.
fn needs_analysis_daemon(function: &Function) -> bool {
    matches!(function, Function::Modect | Function::Mocord)
}

/// Perform full system shutdown.
pub async fn system_shutdown(state: &AppState) -> AppResult<DaemonActionResponse> {
    let manager = state
        .daemon_manager
        .as_ref()
        .ok_or_else(|| AppError::ServiceUnavailableError("Daemon manager not available".into()))?;

    info!("Shutting down all daemons");
    let resp = manager.shutdown_all().await?;
    Ok(response_to_action(resp))
}

/// Perform full system restart (stop all, then start all).
pub async fn system_restart(state: &AppState) -> AppResult<DaemonActionResponse> {
    info!("Restarting ZoneMinder system");

    // First shutdown
    let shutdown_result = system_shutdown(state).await;
    if let Err(e) = &shutdown_result {
        warn!("Shutdown phase had issues: {}", e);
    }

    // Small delay to let processes fully terminate
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Then startup
    let startup_result = system_startup(state).await?;

    Ok(DaemonActionResponse::success(format!(
        "System restarted: {}",
        startup_result.message
    )))
}

/// Trigger log rotation for all daemons (SIGHUP).
pub async fn system_logrot(state: &AppState) -> AppResult<DaemonActionResponse> {
    let manager = state
        .daemon_manager
        .as_ref()
        .ok_or_else(|| AppError::ServiceUnavailableError("Daemon manager not available".into()))?;

    info!("Triggering log rotation for all daemons");

    let daemon_ids = manager.list_daemon_ids().await;
    let mut reloaded = 0;
    let mut failed = 0;

    for id in daemon_ids {
        match manager.reload_daemon(&id).await {
            Ok(resp) if resp.success => {
                reloaded += 1;
                debug!("Sent SIGHUP to {}", id);
            }
            Ok(resp) => {
                // Daemon might not be running
                debug!("Could not reload {}: {}", id, resp.message);
            }
            Err(e) => {
                failed += 1;
                warn!("Error reloading {}: {}", id, e);
            }
        }
    }

    Ok(DaemonActionResponse::success(format!(
        "Log rotation triggered: {} daemons signaled, {} failed",
        reloaded, failed
    )))
}

/// Apply a named system state from the database.
///
/// States define monitor configurations (Function, Enabled settings).
/// Applying a state:
/// 1. Looks up the state by name in the States table
/// 2. Parses the Definition to get monitor configurations
/// 3. Updates monitor settings in the database
/// 4. Restarts affected daemons
pub async fn apply_state(state: &AppState, state_name: &str) -> AppResult<DaemonActionResponse> {
    use crate::entity::states;
    use sea_orm::ActiveValue::Set;

    info!("Applying state: {}", state_name);

    // Look up the state by name
    let zm_state = states::Entity::find()
        .filter(states::Column::Name.eq(state_name))
        .one(state.db.as_ref())
        .await?
        .ok_or_else(|| {
            AppError::NotFoundError(crate::error::Resource {
                resource_type: crate::error::ResourceType::Config,
                details: vec![("state_name".to_string(), state_name.to_string())],
            })
        })?;

    // Parse the state definition
    // Format is: "monitor_id:function:enabled,monitor_id:function:enabled,..."
    // Example: "1:Monitor:1,2:Modect:1,3:None:0"
    let definition = &zm_state.definition;
    let mut updated_monitors = 0;

    for entry in definition.split(',') {
        let parts: Vec<&str> = entry.trim().split(':').collect();
        if parts.len() >= 3 {
            if let Ok(monitor_id) = parts[0].parse::<u32>() {
                let function_str = parts[1];
                let enabled_str = parts[2];

                // Parse function
                let function = match function_str {
                    "None" => Function::None,
                    "Monitor" => Function::Monitor,
                    "Modect" => Function::Modect,
                    "Record" => Function::Record,
                    "Mocord" => Function::Mocord,
                    "Nodect" => Function::Nodect,
                    _ => {
                        warn!(
                            "Unknown function '{}' for monitor {}",
                            function_str, monitor_id
                        );
                        continue;
                    }
                };

                let enabled: u8 = enabled_str.parse().unwrap_or(1);

                // Update monitor in database
                let monitor = monitors::Entity::find_by_id(monitor_id)
                    .one(state.db.as_ref())
                    .await?;

                if let Some(monitor) = monitor {
                    let mut active: monitors::ActiveModel = monitor.into();
                    active.function = Set(function);
                    active.enabled = Set(enabled);
                    active.update(state.db.as_ref()).await?;
                    updated_monitors += 1;
                    debug!(
                        "Updated monitor {}: function={}, enabled={}",
                        monitor_id, function_str, enabled
                    );
                }
            }
        }
    }

    // Mark this state as active
    let mut active_state: states::ActiveModel = zm_state.into();
    active_state.is_active = Set(1);
    active_state.update(state.db.as_ref()).await?;

    // Deactivate other states
    // Note: This is a simplification - in production you'd use a proper UPDATE statement
    let all_states = states::Entity::find().all(state.db.as_ref()).await?;
    for s in all_states {
        if s.name != state_name && s.is_active == 1 {
            let mut active: states::ActiveModel = s.into();
            active.is_active = Set(0);
            active.update(state.db.as_ref()).await?;
        }
    }

    // Restart to apply changes
    info!(
        "State '{}' applied to {} monitors, restarting system",
        state_name, updated_monitors
    );
    let restart_result = system_restart(state).await?;

    Ok(DaemonActionResponse::success(format!(
        "State '{}' applied: {} monitors updated. {}",
        state_name, updated_monitors, restart_result.message
    )))
}

/// Convert daemon response to action response.
fn response_to_action(resp: DaemonResponse) -> DaemonActionResponse {
    if resp.success {
        DaemonActionResponse::success(resp.message)
    } else {
        DaemonActionResponse::error(resp.message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_to_action_success() {
        let resp = DaemonResponse {
            success: true,
            message: "OK".to_string(),
            data: None,
        };
        let action = response_to_action(resp);
        assert!(action.success);
        assert_eq!(action.message, "OK");
    }

    #[test]
    fn test_response_to_action_error() {
        let resp = DaemonResponse {
            success: false,
            message: "Failed".to_string(),
            data: None,
        };
        let action = response_to_action(resp);
        assert!(!action.success);
        assert_eq!(action.message, "Failed");
    }

    #[test]
    fn test_needs_capture_daemon() {
        // All functions except None need capture
        assert!(!needs_capture_daemon(&Function::None));
        assert!(needs_capture_daemon(&Function::Monitor));
        assert!(needs_capture_daemon(&Function::Modect));
        assert!(needs_capture_daemon(&Function::Record));
        assert!(needs_capture_daemon(&Function::Mocord));
        assert!(needs_capture_daemon(&Function::Nodect));
    }

    #[test]
    fn test_needs_analysis_daemon() {
        // Only Modect and Mocord need analysis
        assert!(!needs_analysis_daemon(&Function::None));
        assert!(!needs_analysis_daemon(&Function::Monitor));
        assert!(needs_analysis_daemon(&Function::Modect));
        assert!(!needs_analysis_daemon(&Function::Record));
        assert!(needs_analysis_daemon(&Function::Mocord));
        assert!(!needs_analysis_daemon(&Function::Nodect));
    }
}
