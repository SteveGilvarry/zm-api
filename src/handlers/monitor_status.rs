use crate::dto::request::monitor_status::UpdateMonitorStatusRequest;
use crate::dto::response::monitor_status::PaginatedMonitorStatusesResponse;
use crate::dto::response::MonitorStatusResponse;
use crate::dto::PaginationParams;
use crate::error::AppResult;
use crate::server::state::AppState;
use axum::{
    extract::{Path, Query, State},
    Json,
};

/// List all monitor statuses with pagination.
#[utoipa::path(
    get,
    path = "/api/v3/monitor-status",
    params(
        ("page" = Option<u64>, Query, description = "Page number (1-indexed)", example = 1),
        ("page_size" = Option<u64>, Query, description = "Items per page (max 1000)", example = 25)
    ),
    responses((status = 200, description = "Paginated list of monitor statuses", body = PaginatedMonitorStatusesResponse)),
    tag = "Monitor Status",
    security(("jwt" = []))
)]
pub async fn list_monitor_statuses(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> AppResult<Json<PaginatedMonitorStatusesResponse>> {
    let result = crate::service::monitor_status::list_paginated(&state, &params).await?;
    Ok(Json(PaginatedMonitorStatusesResponse::from(result)))
}

/// Get monitor status by monitor ID.
#[utoipa::path(
    get,
    path = "/api/v3/monitor-status/{monitor_id}",
    params(("monitor_id" = u32, Path, description = "Monitor ID")),
    responses((status = 200, description = "Monitor status detail", body = MonitorStatusResponse)),
    tag = "Monitor Status",
    security(("jwt" = []))
)]
pub async fn get_monitor_status(
    Path(monitor_id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<Json<MonitorStatusResponse>> {
    let item = crate::service::monitor_status::get_by_monitor_id(&state, monitor_id).await?;
    Ok(Json(item))
}

/// Update monitor status.
#[utoipa::path(
    patch,
    path = "/api/v3/monitor-status/{monitor_id}",
    params(("monitor_id" = u32, Path, description = "Monitor ID")),
    request_body = UpdateMonitorStatusRequest,
    responses((status = 200, description = "Updated monitor status", body = MonitorStatusResponse)),
    tag = "Monitor Status",
    security(("jwt" = []))
)]
pub async fn update_monitor_status(
    Path(monitor_id): Path<u32>,
    State(state): State<AppState>,
    Json(req): Json<UpdateMonitorStatusRequest>,
) -> AppResult<Json<MonitorStatusResponse>> {
    let item = crate::service::monitor_status::update(&state, monitor_id, req).await?;
    Ok(Json(item))
}
