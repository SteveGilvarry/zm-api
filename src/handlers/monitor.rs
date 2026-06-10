use axum::extract::{Path, Query, State};
use axum::Json;
use garde::Validate;
use tracing::{info, warn};

use crate::dto::request::{
    AlarmControlRequest, CreateMonitorRequest, UpdateMonitorRequest, UpdateStateRequest,
};
use crate::dto::response::monitors::PaginatedMonitorsResponse;
use crate::dto::response::MonitorResponse;
use crate::dto::PaginationParams;
use crate::error::AppError;
use crate::error::AppResponseError;
use crate::error::AppResult;
use crate::server::state::AppState;
use crate::service;
use crate::service::monitor_acl::MonitorScope;

/// List all monitors with pagination.
///
/// - Requires a valid JWT.
#[utoipa::path(
    get,
    path = "/api/v3/monitors",
    params(
        ("page" = Option<u64>, Query, description = "Page number (1-indexed)", example = 1),
        ("page_size" = Option<u64>, Query, description = "Items per page (max 1000)", example = 25)
    ),
    responses(
        (status = 200, description = "Paginated list of monitors", body = PaginatedMonitorsResponse),
        (status = 401, description = "Unauthorized - Invalid or missing token", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Monitors"
)]
pub async fn list_monitors(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
    scope: MonitorScope,
) -> AppResult<Json<PaginatedMonitorsResponse>> {
    info!("Listing all monitors with pagination.");
    match service::monitor::list_paginated(&state, &params, &scope).await {
        Ok(result) => Ok(Json(PaginatedMonitorsResponse::from(result))),
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
pub async fn get_monitor(
    State(state): State<AppState>,
    Path(id): Path<u32>,
    scope: MonitorScope,
) -> AppResult<Json<MonitorResponse>> {
    info!("Viewing monitor with ID: {id}.");
    match service::monitor::get_by_id(&state, id, &scope).await {
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
        (status = 403, description = "Caller's monitor access is restricted", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Monitors"
)]
pub async fn create_monitor(
    State(state): State<AppState>,
    scope: MonitorScope,
    Json(req): Json<CreateMonitorRequest>,
) -> AppResult<Json<MonitorResponse>> {
    req.validate().map_err(AppError::InvalidInputError)?;
    // Row-level ACL: creating a monitor is a whole-system operation with no
    // pre-existing monitor row to scope against. A user whose access is
    // restricted to specific monitors must not be able to mint new ones — only
    // unrestricted (admin-equivalent) callers may. Feature-level `Monitors:Edit`
    // is already enforced by the route's `protect` layer (REVIEW_FIXES_PLAN §1.4).
    if scope.is_restricted() {
        return Err(AppError::PermissionDeniedError(
            "creating monitors requires unrestricted monitor access".to_string(),
        ));
    }
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
pub async fn update_monitor(
    State(state): State<AppState>,
    Path(id): Path<u32>,
    scope: MonitorScope,
    Json(req): Json<UpdateMonitorRequest>,
) -> AppResult<Json<MonitorResponse>> {
    req.validate().map_err(AppError::InvalidInputError)?;
    info!("Editing monitor with ID: {id} and request: {req:?}.");
    match service::monitor::update(&state, id, req, &scope).await {
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
pub async fn delete_monitor(
    State(state): State<AppState>,
    Path(id): Path<u32>,
    scope: MonitorScope,
) -> AppResult<Json<()>> {
    info!("Deleting monitor with ID: {id}.");
    match service::monitor::delete(&state, id, &scope).await {
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
    scope: MonitorScope,
    Json(req): Json<UpdateStateRequest>,
) -> AppResult<Json<MonitorResponse>> {
    info!("Updating state of monitor with ID: {id} and request: {req:?}.");
    match service::monitor::update_state(&state, id, req, &scope).await {
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
    scope: MonitorScope,
    Json(req): Json<AlarmControlRequest>,
) -> AppResult<Json<MonitorResponse>> {
    info!("Controlling alarm of monitor with ID: {id} and request: {req:?}.");
    match service::monitor::control_alarm(&state, id, req, &scope).await {
        Ok(monitor) => Ok(Json(monitor)),
        Err(e) => {
            warn!("Failed to control alarm of monitor: {e:?}.");
            Err(e)
        }
    }
}
