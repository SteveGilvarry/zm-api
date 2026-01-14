//! HTTP handlers for daemon controller API.

use axum::extract::{Path, State};
use axum::Json;
use tracing::info;

use crate::dto::request::daemon::{ApplyStateRequest, StartDaemonRequest};
use crate::dto::response::daemon::{
    DaemonActionResponse, DaemonListResponse, DaemonStatusResponse, SystemStatusResponse,
};
use crate::error::AppResult;
use crate::server::state::AppState;
use crate::service;

/// List all daemons.
#[utoipa::path(
    get,
    path = "/api/v3/daemons",
    responses(
        (status = 200, description = "List of all daemons", body = DaemonListResponse),
        (status = 503, description = "Daemon manager not available")
    ),
    tag = "Daemons"
)]
pub async fn list_daemons(State(state): State<AppState>) -> AppResult<Json<DaemonListResponse>> {
    info!("Listing all daemons");
    let response = service::daemon::get_all_daemons(&state).await?;
    Ok(Json(response))
}

/// Get a specific daemon's status.
#[utoipa::path(
    get,
    path = "/api/v3/daemons/{id}",
    params(
        ("id" = String, Path, description = "Daemon identifier")
    ),
    responses(
        (status = 200, description = "Daemon status", body = DaemonStatusResponse),
        (status = 404, description = "Daemon not found"),
        (status = 503, description = "Daemon manager not available")
    ),
    tag = "Daemons"
)]
pub async fn get_daemon(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<DaemonStatusResponse>> {
    info!("Getting daemon status: {}", id);
    let response = service::daemon::get_daemon(&state, &id).await?;
    Ok(Json(response))
}

/// Start a daemon.
#[utoipa::path(
    post,
    path = "/api/v3/daemons/{id}/start",
    params(
        ("id" = String, Path, description = "Daemon identifier")
    ),
    request_body = StartDaemonRequest,
    responses(
        (status = 200, description = "Daemon started", body = DaemonActionResponse),
        (status = 503, description = "Daemon manager not available")
    ),
    tag = "Daemons",
    security(("jwt" = []))
)]
pub async fn start_daemon(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<StartDaemonRequest>,
) -> AppResult<Json<DaemonActionResponse>> {
    info!("Starting daemon: {} with args: {:?}", id, request.args);
    let response = service::daemon::start_daemon(&state, &id, &request.args).await?;
    Ok(Json(response))
}

/// Stop a daemon.
#[utoipa::path(
    post,
    path = "/api/v3/daemons/{id}/stop",
    params(
        ("id" = String, Path, description = "Daemon identifier")
    ),
    responses(
        (status = 200, description = "Daemon stopped", body = DaemonActionResponse),
        (status = 503, description = "Daemon manager not available")
    ),
    tag = "Daemons",
    security(("jwt" = []))
)]
pub async fn stop_daemon(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<DaemonActionResponse>> {
    info!("Stopping daemon: {}", id);
    let response = service::daemon::stop_daemon(&state, &id).await?;
    Ok(Json(response))
}

/// Restart a daemon.
#[utoipa::path(
    post,
    path = "/api/v3/daemons/{id}/restart",
    params(
        ("id" = String, Path, description = "Daemon identifier")
    ),
    responses(
        (status = 200, description = "Daemon restarted", body = DaemonActionResponse),
        (status = 503, description = "Daemon manager not available")
    ),
    tag = "Daemons",
    security(("jwt" = []))
)]
pub async fn restart_daemon(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<DaemonActionResponse>> {
    info!("Restarting daemon: {}", id);
    let response = service::daemon::restart_daemon(&state, &id).await?;
    Ok(Json(response))
}

/// Reload a daemon's configuration (SIGHUP).
#[utoipa::path(
    post,
    path = "/api/v3/daemons/{id}/reload",
    params(
        ("id" = String, Path, description = "Daemon identifier")
    ),
    responses(
        (status = 200, description = "Reload signal sent", body = DaemonActionResponse),
        (status = 503, description = "Daemon manager not available")
    ),
    tag = "Daemons",
    security(("jwt" = []))
)]
pub async fn reload_daemon(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<DaemonActionResponse>> {
    info!("Reloading daemon: {}", id);
    let response = service::daemon::reload_daemon(&state, &id).await?;
    Ok(Json(response))
}

/// Get system status.
#[utoipa::path(
    get,
    path = "/api/v3/system/status",
    responses(
        (status = 200, description = "System status", body = SystemStatusResponse),
        (status = 503, description = "Daemon manager not available")
    ),
    tag = "System"
)]
pub async fn get_system_status(
    State(state): State<AppState>,
) -> AppResult<Json<SystemStatusResponse>> {
    info!("Getting system status");
    let response = service::daemon::get_system_status(&state).await?;
    Ok(Json(response))
}

/// Perform full system startup.
#[utoipa::path(
    post,
    path = "/api/v3/system/startup",
    responses(
        (status = 200, description = "System started", body = DaemonActionResponse),
        (status = 503, description = "Daemon manager not available")
    ),
    tag = "System",
    security(("jwt" = []))
)]
pub async fn system_startup(
    State(state): State<AppState>,
) -> AppResult<Json<DaemonActionResponse>> {
    info!("Starting system");
    let response = service::daemon::system_startup(&state).await?;
    Ok(Json(response))
}

/// Perform full system shutdown.
#[utoipa::path(
    post,
    path = "/api/v3/system/shutdown",
    responses(
        (status = 200, description = "System shutdown initiated", body = DaemonActionResponse),
        (status = 503, description = "Daemon manager not available")
    ),
    tag = "System",
    security(("jwt" = []))
)]
pub async fn system_shutdown(
    State(state): State<AppState>,
) -> AppResult<Json<DaemonActionResponse>> {
    info!("Shutting down system");
    let response = service::daemon::system_shutdown(&state).await?;
    Ok(Json(response))
}

/// Apply a system state.
#[utoipa::path(
    post,
    path = "/api/v3/system/state",
    request_body = ApplyStateRequest,
    responses(
        (status = 200, description = "State applied", body = DaemonActionResponse),
        (status = 503, description = "Daemon manager not available")
    ),
    tag = "System",
    security(("jwt" = []))
)]
pub async fn apply_state(
    State(_state): State<AppState>,
    Json(request): Json<ApplyStateRequest>,
) -> AppResult<Json<DaemonActionResponse>> {
    info!("Applying state: {}", request.state_name);
    // TODO: Implement state application with database lookup
    Ok(Json(DaemonActionResponse::success(format!(
        "State '{}' applied (stub)",
        request.state_name
    ))))
}

#[cfg(test)]
mod tests {
    // Handler tests would require mocking AppState with daemon_manager
    // which is added in Phase 7
}
