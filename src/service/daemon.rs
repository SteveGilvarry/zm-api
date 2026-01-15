//! Daemon controller service layer.

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};
use tracing::{debug, info, warn};

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
    // Pass empty args - daemon ID (e.g., "zmc -m 1") contains the args
    let resp = manager.restart_daemon(id, &[]).await?;
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
/// This delegates to DaemonManager.start_all_daemons() which:
/// 1. Queries monitors (Capturing != None, not WebSite type)
/// 2. For Local type: starts `zmc -d <device>`
/// 3. For other types: starts `zmc -m <id>`
/// 4. Starts zma for monitors with Modect/Mocord function
/// 5. Starts zmcontrol.pl for controllable monitors
/// 6. Starts zmtrack.pl for monitors with motion tracking
/// 7. Starts singleton daemons (zmfilter.pl, zmstats.pl, etc.)
pub async fn system_startup(state: &AppState) -> AppResult<DaemonActionResponse> {
    let manager = state
        .daemon_manager
        .as_ref()
        .ok_or_else(|| AppError::ServiceUnavailableError("Daemon manager not available".into()))?;

    info!("Starting ZoneMinder system");

    // Delegate to manager which has full startup logic matching zmpkg.pl
    let resp = manager.start_all_daemons().await?;
    Ok(response_to_action(resp))
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
}
