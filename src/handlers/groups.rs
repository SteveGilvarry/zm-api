use crate::dto::request::CreateGroupRequest;
use crate::dto::response::GroupResponse;
use crate::error::AppResult;
use crate::server::state::AppState;
use axum::{
    extract::{Path, State},
    Json,
};
use serde::Deserialize;

/// List user groups.
///
/// - Requires a valid JWT.
#[utoipa::path(
    get,
    path = "/api/v3/groups",
    responses((status = 200, description = "List groups", body = serde_json::Value)),
    tag = "Groups",
    security(("jwt" = []))
)]
pub async fn list_groups(State(state): State<AppState>) -> AppResult<Json<Vec<GroupResponse>>> {
    let items = crate::service::groups::list_all(&state).await?;
    Ok(Json(items))
}

/// Get a group by id.
///
/// - Requires a valid JWT.
#[utoipa::path(
    get,
    path = "/api/v3/groups/{id}",
    params(("id" = u32, Path, description = "Group ID")),
    responses((status = 200, description = "Group detail", body = serde_json::Value)),
    tag = "Groups",
    security(("jwt" = []))
)]
pub async fn get_group(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<Json<GroupResponse>> {
    let item = crate::service::groups::get_by_id(&state, id).await?;
    Ok(Json(item))
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateGroupRequest {
    pub name: Option<String>,
}

/// Update a group's basic attributes.
///
/// - Partial update; only provided fields are changed.
#[utoipa::path(
    put,
    path = "/api/v3/groups/{id}",
    params(("id" = u32, Path, description = "Group ID")),
    request_body = UpdateGroupRequest,
    responses((status = 200, description = "Updated group", body = serde_json::Value)),
    tag = "Groups",
    security(("jwt" = []))
)]
pub async fn update_group(
    Path(id): Path<u32>,
    State(state): State<AppState>,
    Json(req): Json<UpdateGroupRequest>,
) -> AppResult<Json<GroupResponse>> {
    let item = crate::service::groups::update(&state, id, req.name).await?;
    Ok(Json(item))
}

/// Create a new group.
///
/// - Optionally specifies `parent_id` to create nested groups.
#[utoipa::path(
    post,
    path = "/api/v3/groups",
    request_body = CreateGroupRequest,
    responses((status = 201, description = "Created group", body = GroupResponse)),
    tag = "Groups",
    security(("jwt" = []))
)]
pub async fn create_group(
    State(state): State<AppState>,
    Json(req): Json<CreateGroupRequest>,
) -> AppResult<(axum::http::StatusCode, Json<GroupResponse>)> {
    let item = crate::service::groups::create(&state, req).await?;
    Ok((axum::http::StatusCode::CREATED, Json(item)))
}

/// Delete a group by id.
///
/// - Responds 204 on success, 404 if not found.
#[utoipa::path(
    delete,
    path = "/api/v3/groups/{id}",
    params(("id" = u32, Path, description = "Group ID")),
    responses((status = 204, description = "Deleted group")),
    tag = "Groups",
    security(("jwt" = []))
)]
pub async fn delete_group(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<axum::http::StatusCode> {
    crate::service::groups::delete(&state, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
