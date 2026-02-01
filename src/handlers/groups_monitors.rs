use crate::dto::request::groups_monitors::CreateGroupMonitorRequest;
use crate::dto::response::groups_monitors::PaginatedGroupMonitorsResponse;
use crate::dto::response::GroupMonitorResponse;
use crate::dto::PaginationParams;
use crate::error::AppResult;
use crate::server::state::AppState;
use axum::{
    extract::{Path, Query, State},
    Json,
};

/// List all group-monitor associations with pagination.
///
/// - Requires a valid JWT.
#[utoipa::path(
    get,
    path = "/api/v3/groups-monitors",
    params(
        ("page" = Option<u64>, Query, description = "Page number (1-indexed)", example = 1),
        ("page_size" = Option<u64>, Query, description = "Items per page (max 1000)", example = 25)
    ),
    responses((status = 200, description = "Paginated list of group-monitor associations", body = PaginatedGroupMonitorsResponse)),
    tag = "Groups Monitors",
    security(("jwt" = []))
)]
pub async fn list_groups_monitors(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> AppResult<Json<PaginatedGroupMonitorsResponse>> {
    let result = crate::service::groups_monitors::list_paginated(&state, &params).await?;
    Ok(Json(PaginatedGroupMonitorsResponse::from(result)))
}

/// Get a group-monitor association by id.
#[utoipa::path(
    get,
    path = "/api/v3/groups-monitors/{id}",
    params(("id" = u32, Path, description = "Association ID")),
    responses((status = 200, description = "Group-monitor association detail", body = GroupMonitorResponse)),
    tag = "Groups Monitors",
    security(("jwt" = []))
)]
pub async fn get_group_monitor(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<Json<GroupMonitorResponse>> {
    let item = crate::service::groups_monitors::get_by_id(&state, id).await?;
    Ok(Json(item))
}

/// Create a new group-monitor association.
#[utoipa::path(
    post,
    path = "/api/v3/groups-monitors",
    request_body = CreateGroupMonitorRequest,
    responses((status = 201, description = "Created group-monitor association", body = GroupMonitorResponse)),
    tag = "Groups Monitors",
    security(("jwt" = []))
)]
pub async fn create_group_monitor(
    State(state): State<AppState>,
    Json(req): Json<CreateGroupMonitorRequest>,
) -> AppResult<(axum::http::StatusCode, Json<GroupMonitorResponse>)> {
    let item = crate::service::groups_monitors::create(&state, req).await?;
    Ok((axum::http::StatusCode::CREATED, Json(item)))
}

/// Delete a group-monitor association by id.
#[utoipa::path(
    delete,
    path = "/api/v3/groups-monitors/{id}",
    params(("id" = u32, Path, description = "Association ID")),
    responses((status = 204, description = "Deleted group-monitor association")),
    tag = "Groups Monitors",
    security(("jwt" = []))
)]
pub async fn delete_group_monitor(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<axum::http::StatusCode> {
    crate::service::groups_monitors::delete(&state, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
