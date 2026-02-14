//! Daemon controller service layer.

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};
use tracing::{debug, info, warn};

use crate::daemon::ipc::DaemonResponse;
use crate::dto::response::daemon::{
    DaemonActionResponse, DaemonListResponse, DaemonStatusResponse, SystemStatusResponse,
};
use crate::entity::monitors;
use crate::entity::sea_orm_active_enums::{Analysing, Capturing, Function, Recording};
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
/// States define monitor configurations. Two formats are supported:
/// - Legacy (3-part): "id:function:enabled" - Updates Function and Enabled fields
/// - New (4-part): "id:capturing:analysing:recording" - Updates Capturing, Analysing, Recording fields
///
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
    // Two formats supported:
    // - Legacy (3-part): "id:function:enabled" e.g., "1:Monitor:1,2:Modect:1,3:None:0"
    // - New (4-part): "id:capturing:analysing:recording" e.g., "1:Always:Always:OnMotion"
    let definition = &zm_state.definition;
    let mut updated_monitors = 0;

    for entry in definition.split(',') {
        let entry = entry.trim();
        if entry.is_empty() {
            continue;
        }

        let parts: Vec<&str> = entry.split(':').collect();

        match parts.len() {
            3 => {
                // Legacy format: id:function:enabled
                let monitor_id = match parts[0].parse::<u32>() {
                    Ok(id) => id,
                    Err(_) => {
                        warn!("Invalid monitor ID in state definition: {}", parts[0]);
                        continue;
                    }
                };

                let function = match parse_function(parts[1]) {
                    Some(f) => f,
                    None => {
                        warn!("Unknown function '{}' for monitor {}", parts[1], monitor_id);
                        continue;
                    }
                };

                let enabled: u8 = parts[2].parse().unwrap_or(1);

                // Map legacy enabled field to Capturing:
                // enabled=0 → Capturing::None (disabled)
                // enabled=1 → Capturing::Always (default active state)
                let capturing = if enabled == 0 {
                    Capturing::None
                } else {
                    Capturing::Always
                };

                // Update monitor in database
                let monitor = monitors::Entity::find_by_id(monitor_id)
                    .one(state.db.as_ref())
                    .await?;

                if let Some(monitor) = monitor {
                    let mut active: monitors::ActiveModel = monitor.into();
                    active.function = Set(function.clone());
                    active.capturing = Set(capturing.clone());
                    active.update(state.db.as_ref()).await?;
                    updated_monitors += 1;
                    debug!(
                        "Updated monitor {} (legacy): function={:?}, capturing={:?}",
                        monitor_id, function, capturing
                    );
                }
            }
            4 => {
                // New format: id:capturing:analysing:recording
                let monitor_id = match parts[0].parse::<u32>() {
                    Ok(id) => id,
                    Err(_) => {
                        warn!("Invalid monitor ID in state definition: {}", parts[0]);
                        continue;
                    }
                };

                let capturing = match parse_capturing(parts[1]) {
                    Some(c) => c,
                    None => {
                        warn!(
                            "Unknown capturing value '{}' for monitor {}",
                            parts[1], monitor_id
                        );
                        continue;
                    }
                };

                let analysing = match parse_analysing(parts[2]) {
                    Some(a) => a,
                    None => {
                        warn!(
                            "Unknown analysing value '{}' for monitor {}",
                            parts[2], monitor_id
                        );
                        continue;
                    }
                };

                let recording = match parse_recording(parts[3]) {
                    Some(r) => r,
                    None => {
                        warn!(
                            "Unknown recording value '{}' for monitor {}",
                            parts[3], monitor_id
                        );
                        continue;
                    }
                };

                // Update monitor in database
                let monitor = monitors::Entity::find_by_id(monitor_id)
                    .one(state.db.as_ref())
                    .await?;

                if let Some(monitor) = monitor {
                    let mut active: monitors::ActiveModel = monitor.into();
                    active.capturing = Set(capturing.clone());
                    active.analysing = Set(analysing.clone());
                    active.recording = Set(recording.clone());
                    active.update(state.db.as_ref()).await?;
                    updated_monitors += 1;
                    debug!(
                        "Updated monitor {} (new): capturing={:?}, analysing={:?}, recording={:?}",
                        monitor_id, capturing, analysing, recording
                    );
                }
            }
            _ => {
                if !entry.is_empty() {
                    warn!(
                        "Invalid state definition entry (expected 3 or 4 parts): {}",
                        entry
                    );
                }
                continue;
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

/// Parse a function string to the Function enum.
fn parse_function(s: &str) -> Option<Function> {
    match s {
        "None" => Some(Function::None),
        "Monitor" => Some(Function::Monitor),
        "Modect" => Some(Function::Modect),
        "Record" => Some(Function::Record),
        "Mocord" => Some(Function::Mocord),
        "Nodect" => Some(Function::Nodect),
        _ => None,
    }
}

/// Parse a capturing string to the Capturing enum.
fn parse_capturing(s: &str) -> Option<Capturing> {
    match s {
        "None" => Some(Capturing::None),
        "Ondemand" => Some(Capturing::Ondemand),
        "Always" => Some(Capturing::Always),
        _ => None,
    }
}

/// Parse an analysing string to the Analysing enum.
fn parse_analysing(s: &str) -> Option<Analysing> {
    match s {
        "None" => Some(Analysing::None),
        "Always" => Some(Analysing::Always),
        _ => None,
    }
}

/// Parse a recording string to the Recording enum.
fn parse_recording(s: &str) -> Option<Recording> {
    match s {
        "None" => Some(Recording::None),
        "OnMotion" => Some(Recording::OnMotion),
        "Always" => Some(Recording::Always),
        _ => None,
    }
}

/// Ensure the States table is sane.
///
/// This function ensures:
/// 1. A "default" state exists (creates one if missing)
/// 2. Exactly one state is marked as active (fixes if not)
///
/// This matches the behavior of zmpkg.pl which ensures the states table
/// has a default state and exactly one active state at system startup.
pub async fn ensure_state_sanity(db: &sea_orm::DatabaseConnection) -> AppResult<()> {
    use crate::entity::states;
    use sea_orm::{ActiveModelTrait, ActiveValue::Set, QueryOrder};

    info!("Checking state table sanity");

    // Step 1: Ensure "default" state exists
    let default_states: Vec<states::Model> = states::Entity::find()
        .filter(states::Column::Name.eq("default"))
        .order_by_asc(states::Column::Id)
        .all(db)
        .await?;

    // Remove duplicate "default" states if any (keep the first one)
    if default_states.len() > 1 {
        warn!(
            "Found {} duplicate 'default' states, removing extras",
            default_states.len() - 1
        );
        for state in default_states.iter().skip(1) {
            states::Entity::delete_by_id(state.id).exec(db).await?;
        }
    }

    // Create default state if missing
    let default_state = if default_states.is_empty() {
        info!("Creating missing 'default' state");
        let new_default = states::ActiveModel {
            name: Set("default".to_string()),
            definition: Set(String::new()),
            is_active: Set(1),
            ..Default::default()
        };
        new_default.insert(db).await?
    } else {
        default_states.into_iter().next().unwrap()
    };

    // Step 2: Ensure exactly one active state
    let active_states: Vec<states::Model> = states::Entity::find()
        .filter(states::Column::IsActive.eq(1u8))
        .all(db)
        .await?;

    let active_count = active_states.len();
    if active_count != 1 {
        info!("Found {} active states (expected 1), fixing", active_count);

        // Reset all states to inactive
        use sea_orm::sea_query::Expr;
        states::Entity::update_many()
            .col_expr(states::Column::IsActive, Expr::value(0u8))
            .exec(db)
            .await?;

        // Set default state as active
        let mut active_default: states::ActiveModel = default_state.into();
        active_default.is_active = Set(1);
        active_default.update(db).await?;

        info!("Set 'default' state as the active state");
    }

    debug!("State table sanity check complete");
    Ok(())
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
    fn test_parse_function() {
        assert_eq!(parse_function("None"), Some(Function::None));
        assert_eq!(parse_function("Monitor"), Some(Function::Monitor));
        assert_eq!(parse_function("Modect"), Some(Function::Modect));
        assert_eq!(parse_function("Record"), Some(Function::Record));
        assert_eq!(parse_function("Mocord"), Some(Function::Mocord));
        assert_eq!(parse_function("Nodect"), Some(Function::Nodect));
        assert_eq!(parse_function("invalid"), None);
        assert_eq!(parse_function(""), None);
    }

    #[test]
    fn test_parse_capturing() {
        assert_eq!(parse_capturing("None"), Some(Capturing::None));
        assert_eq!(parse_capturing("Ondemand"), Some(Capturing::Ondemand));
        assert_eq!(parse_capturing("Always"), Some(Capturing::Always));
        assert_eq!(parse_capturing("invalid"), None);
        assert_eq!(parse_capturing(""), None);
    }

    #[test]
    fn test_parse_analysing() {
        assert_eq!(parse_analysing("None"), Some(Analysing::None));
        assert_eq!(parse_analysing("Always"), Some(Analysing::Always));
        assert_eq!(parse_analysing("invalid"), None);
        assert_eq!(parse_analysing(""), None);
    }

    #[test]
    fn test_parse_recording() {
        assert_eq!(parse_recording("None"), Some(Recording::None));
        assert_eq!(parse_recording("OnMotion"), Some(Recording::OnMotion));
        assert_eq!(parse_recording("Always"), Some(Recording::Always));
        assert_eq!(parse_recording("invalid"), None);
        assert_eq!(parse_recording(""), None);
    }
}
