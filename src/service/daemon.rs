//! Daemon controller service layer.

use tracing::info;

use crate::daemon::ipc::DaemonResponse;
use crate::dto::response::daemon::{
    DaemonActionResponse, DaemonListResponse, DaemonStatusResponse, SystemStatusResponse,
};
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
pub async fn system_startup(state: &AppState) -> AppResult<DaemonActionResponse> {
    let manager = state
        .daemon_manager
        .as_ref()
        .ok_or_else(|| AppError::ServiceUnavailableError("Daemon manager not available".into()))?;

    info!("Starting daemon controller system");
    let resp = manager.startup().await?;
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
