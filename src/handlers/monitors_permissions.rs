use axum::{extract::{Path, State, Query}, Json};
use serde::Deserialize;
use crate::dto::response::MonitorPermissionResponse;
use crate::dto::request::monitors_permissions::{CreateMonitorPermissionRequest, UpdateMonitorPermissionRequest};
use crate::error::AppResult;
use crate::server::state::AppState;

#[derive(Debug, Deserialize)]
pub struct MonitorPermissionQuery {
    monitor_id: Option<u32>,
    user_id: Option<u32>,
}

/// List all monitor permissions.
#[utoipa::path(
    get,
    path = "/api/v3/monitors-permissions",
    params(
        ("monitor_id" = Option<u32>, Query, description = "Filter by monitor ID"),
        ("user_id" = Option<u32>, Query, description = "Filter by user ID")
    ),
    responses((status = 200, description = "List monitor permissions", body = [MonitorPermissionResponse])),
    tag = "Monitors Permissions",
    security(("jwt" = []))
)]
pub async fn list_monitors_permissions(Query(params): Query<MonitorPermissionQuery>, State(state): State<AppState>) -> AppResult<Json<Vec<MonitorPermissionResponse>>> {
    let items = crate::service::monitors_permissions::list_all(&state, params.monitor_id, params.user_id).await?;
    Ok(Json(items))
}

/// Get a monitor permission by id.
#[utoipa::path(
    get,
    path = "/api/v3/monitors-permissions/{id}",
    params(("id" = u32, Path, description = "Permission ID")),
    responses((status = 200, description = "Monitor permission detail", body = MonitorPermissionResponse)),
    tag = "Monitors Permissions",
    security(("jwt" = []))
)]
pub async fn get_monitor_permission(Path(id): Path<u32>, State(state): State<AppState>) -> AppResult<Json<MonitorPermissionResponse>> {
    let item = crate::service::monitors_permissions::get_by_id(&state, id).await?;
    Ok(Json(item))
}

/// Create a new monitor permission.
#[utoipa::path(
    post,
    path = "/api/v3/monitors-permissions",
    request_body = CreateMonitorPermissionRequest,
    responses((status = 201, description = "Created monitor permission", body = MonitorPermissionResponse)),
    tag = "Monitors Permissions",
    security(("jwt" = []))
)]
pub async fn create_monitor_permission(State(state): State<AppState>, Json(req): Json<CreateMonitorPermissionRequest>) -> AppResult<(axum::http::StatusCode, Json<MonitorPermissionResponse>)> {
    let item = crate::service::monitors_permissions::create(&state, req).await?;
    Ok((axum::http::StatusCode::CREATED, Json(item)))
}

/// Update a monitor permission.
#[utoipa::path(
    patch,
    path = "/api/v3/monitors-permissions/{id}",
    params(("id" = u32, Path, description = "Permission ID")),
    request_body = UpdateMonitorPermissionRequest,
    responses((status = 200, description = "Updated monitor permission", body = MonitorPermissionResponse)),
    tag = "Monitors Permissions",
    security(("jwt" = []))
)]
pub async fn update_monitor_permission(Path(id): Path<u32>, State(state): State<AppState>, Json(req): Json<UpdateMonitorPermissionRequest>) -> AppResult<Json<MonitorPermissionResponse>> {
    let item = crate::service::monitors_permissions::update(&state, id, req).await?;
    Ok(Json(item))
}

/// Delete a monitor permission by id.
#[utoipa::path(
    delete,
    path = "/api/v3/monitors-permissions/{id}",
    params(("id" = u32, Path, description = "Permission ID")),
    responses((status = 204, description = "Deleted monitor permission")),
    tag = "Monitors Permissions",
    security(("jwt" = []))
)]
pub async fn delete_monitor_permission(Path(id): Path<u32>, State(state): State<AppState>) -> AppResult<axum::http::StatusCode> {
    crate::service::monitors_permissions::delete(&state, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
