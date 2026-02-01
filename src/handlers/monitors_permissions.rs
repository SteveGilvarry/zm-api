use crate::dto::request::monitors_permissions::{
    CreateMonitorPermissionRequest, UpdateMonitorPermissionRequest,
};
use crate::dto::response::monitors_permissions::PaginatedMonitorPermissionsResponse;
use crate::dto::response::MonitorPermissionResponse;
use crate::dto::PaginationParams;
use crate::error::AppResult;
use crate::server::state::AppState;
use axum::{
    extract::{Path, Query, State},
    Json,
};

/// List all monitor permissions with pagination.
#[utoipa::path(
    get,
    path = "/api/v3/monitors-permissions",
    params(
        ("page" = Option<u64>, Query, description = "Page number (1-indexed)", example = 1),
        ("page_size" = Option<u64>, Query, description = "Items per page (max 1000)", example = 25)
    ),
    responses((status = 200, description = "Paginated list of monitor permissions", body = PaginatedMonitorPermissionsResponse)),
    tag = "Monitors Permissions",
    security(("jwt" = []))
)]
pub async fn list_monitors_permissions(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> AppResult<Json<PaginatedMonitorPermissionsResponse>> {
    let result = crate::service::monitors_permissions::list_paginated(&state, &params).await?;
    Ok(Json(PaginatedMonitorPermissionsResponse::from(result)))
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
pub async fn get_monitor_permission(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<Json<MonitorPermissionResponse>> {
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
pub async fn create_monitor_permission(
    State(state): State<AppState>,
    Json(req): Json<CreateMonitorPermissionRequest>,
) -> AppResult<(axum::http::StatusCode, Json<MonitorPermissionResponse>)> {
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
pub async fn update_monitor_permission(
    Path(id): Path<u32>,
    State(state): State<AppState>,
    Json(req): Json<UpdateMonitorPermissionRequest>,
) -> AppResult<Json<MonitorPermissionResponse>> {
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
pub async fn delete_monitor_permission(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<axum::http::StatusCode> {
    crate::service::monitors_permissions::delete(&state, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
