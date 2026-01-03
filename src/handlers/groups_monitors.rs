use crate::dto::request::groups_monitors::CreateGroupMonitorRequest;
use crate::dto::response::GroupMonitorResponse;
use crate::error::AppResult;
use crate::server::state::AppState;
use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GroupMonitorQuery {
    group_id: Option<u32>,
    monitor_id: Option<u32>,
}

/// List all group-monitor associations.
#[utoipa::path(
    get,
    path = "/api/v3/groups-monitors",
    params(
        ("group_id" = Option<u32>, Query, description = "Filter by group ID"),
        ("monitor_id" = Option<u32>, Query, description = "Filter by monitor ID")
    ),
    responses((status = 200, description = "List group-monitor associations", body = [GroupMonitorResponse])),
    tag = "Groups Monitors",
    security(("jwt" = []))
)]
pub async fn list_groups_monitors(
    Query(params): Query<GroupMonitorQuery>,
    State(state): State<AppState>,
) -> AppResult<Json<Vec<GroupMonitorResponse>>> {
    let items =
        crate::service::groups_monitors::list_all(&state, params.group_id, params.monitor_id)
            .await?;
    Ok(Json(items))
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
