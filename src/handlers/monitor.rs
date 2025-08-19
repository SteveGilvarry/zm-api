use axum::extract::{Path, State};
use axum::Json;
use tracing::{info, warn};

use crate::error::AppResult;
use crate::server::state::AppState;
use crate::{dto::*, service};

#[utoipa::path(
    get,
    path = "/api/v3/monitors",
    responses(
        (status = 200, description = "List all monitors", body = Vec<MonitorResponse>),
        (status = 401, description = "Unauthorized - Invalid or missing token", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Monitors"
)]
pub async fn index(State(state): State<AppState>) -> AppResult<Json<Vec<MonitorResponse>>> {
    info!("Listing all monitors.");
    match service::monitor::list_all(&state).await {
        Ok(monitors) => Ok(Json(monitors)),
        Err(e) => {
            warn!("Failed to list monitors: {e:?}.");
            Err(e)
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v3/monitors/{id}",
    params(
        ("id" = u32, Path, description = "Monitor identifier")
    ),
    responses(
        (status = 200, description = "View monitor details", body = MonitorResponse),
        (status = 401, description = "Unauthorized - Invalid or missing token", body = AppResponseError),
        (status = 404, description = "Monitor not found", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Monitors"
)]
pub async fn view(State(state): State<AppState>, Path(id): Path<u32>) -> AppResult<Json<MonitorResponse>> {
    info!("Viewing monitor with ID: {id}.");
    match service::monitor::get_by_id(&state, id).await {
        Ok(monitor) => Ok(Json(monitor)),
        Err(e) => {
            warn!("Failed to view monitor: {e:?}.");
            Err(e)
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/v3/monitors",
    request_body = CreateMonitorRequest,
    responses(
        (status = 200, description = "Monitor created successfully", body = MonitorResponse),
        (status = 400, description = "Invalid request data", body = AppResponseError),
        (status = 401, description = "Unauthorized - Invalid or missing token", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Monitors"
)]
pub async fn create(State(state): State<AppState>, Json(req): Json<CreateMonitorRequest>) -> AppResult<Json<MonitorResponse>> {
    info!("Creating new monitor with request: {req:?}.");
    match service::monitor::create(&state, req).await {
        Ok(monitor) => Ok(Json(monitor)),
        Err(e) => {
            warn!("Failed to create monitor: {e:?}.");
            Err(e)
        }
    }
}

#[utoipa::path(
    patch,
    path = "/api/v3/monitors/{id}",
    params(
        ("id" = u32, Path, description = "Monitor identifier")
    ),
    request_body = UpdateMonitorRequest,
    responses(
        (status = 200, description = "Monitor updated successfully", body = MonitorResponse),
        (status = 400, description = "Invalid request data", body = AppResponseError),
        (status = 401, description = "Unauthorized - Invalid or missing token", body = AppResponseError),
        (status = 404, description = "Monitor not found", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Monitors"
)]
pub async fn edit(
    State(state): State<AppState>,
    Path(id): Path<u32>,
    Json(req): Json<UpdateMonitorRequest>
) -> AppResult<Json<MonitorResponse>> {
    info!("Editing monitor with ID: {id} and request: {req:?}.");
    match service::monitor::update(&state, id, req).await {
        Ok(monitor) => Ok(Json(monitor)),
        Err(e) => {
            warn!("Failed to edit monitor: {e:?}.");
            Err(e)
        }
    }
}

#[utoipa::path(
    delete,
    path = "/api/v3/monitors/{id}",
    params(
        ("id" = u32, Path, description = "Monitor identifier")
    ),
    responses(
        (status = 200, description = "Monitor deleted successfully"),
        (status = 401, description = "Unauthorized - Invalid or missing token", body = AppResponseError),
        (status = 404, description = "Monitor not found", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Monitors"
)]
pub async fn delete(State(state): State<AppState>, Path(id): Path<u32>) -> AppResult<Json<()>> {
    info!("Deleting monitor with ID: {id}.");
    match service::monitor::delete(&state, id).await {
        Ok(_) => Ok(Json(())),
        Err(e) => {
            warn!("Failed to delete monitor: {e:?}.");
            Err(e)
        }
    }
}

#[utoipa::path(
    patch,
    path = "/api/v3/monitors/{id}/state",
    params(
        ("id" = u32, Path, description = "Monitor identifier")
    ),
    request_body = UpdateStateRequest,
    responses(
        (status = 200, description = "Monitor state updated successfully", body = MonitorResponse),
        (status = 400, description = "Invalid request data", body = AppResponseError),
        (status = 401, description = "Unauthorized - Invalid or missing token", body = AppResponseError),
        (status = 404, description = "Monitor not found", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Monitors"
)]
pub async fn update_state(
    State(state): State<AppState>,
    Path(id): Path<u32>,
    Json(req): Json<UpdateStateRequest>
) -> AppResult<Json<MonitorResponse>> {
    info!("Updating state of monitor with ID: {id} and request: {req:?}.");
    match service::monitor::update_state(&state, id, req).await {
        Ok(monitor) => Ok(Json(monitor)),
        Err(e) => {
            warn!("Failed to update state of monitor: {e:?}.");
            Err(e)
        }
    }
}

#[utoipa::path(
    patch,
    path = "/api/v3/monitors/{id}/alarm",
    params(
        ("id" = u32, Path, description = "Monitor identifier")
    ),
    request_body = AlarmControlRequest,
    responses(
        (status = 200, description = "Monitor alarm controlled successfully", body = MonitorResponse),
        (status = 400, description = "Invalid request data", body = AppResponseError),
        (status = 401, description = "Unauthorized - Invalid or missing token", body = AppResponseError),
        (status = 404, description = "Monitor not found", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError) 
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Monitors"
)]
pub async fn alarm_control(
    State(state): State<AppState>,
    Path(id): Path<u32>,
    Json(req): Json<AlarmControlRequest>
) -> AppResult<Json<MonitorResponse>> {
    info!("Controlling alarm of monitor with ID: {id} and request: {req:?}.");
    match service::monitor::control_alarm(&state, id, req).await {
        Ok(monitor) => Ok(Json(monitor)),
        Err(e) => {
            warn!("Failed to control alarm of monitor: {e:?}.");
            Err(e)
        }
    }
}