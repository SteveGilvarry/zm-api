use crate::dto::request::groups_permissions::{
    CreateGroupPermissionRequest, UpdateGroupPermissionRequest,
};
use crate::dto::response::GroupPermissionResponse;
use crate::error::AppResult;
use crate::server::state::AppState;
use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GroupPermissionQuery {
    group_id: Option<u32>,
    user_id: Option<u32>,
}

/// List all group permissions.
#[utoipa::path(
    get,
    path = "/api/v3/groups-permissions",
    params(
        ("group_id" = Option<u32>, Query, description = "Filter by group ID"),
        ("user_id" = Option<u32>, Query, description = "Filter by user ID")
    ),
    responses((status = 200, description = "List group permissions", body = [GroupPermissionResponse])),
    tag = "Groups Permissions",
    security(("jwt" = []))
)]
pub async fn list_groups_permissions(
    Query(params): Query<GroupPermissionQuery>,
    State(state): State<AppState>,
) -> AppResult<Json<Vec<GroupPermissionResponse>>> {
    let items =
        crate::service::groups_permissions::list_all(&state, params.group_id, params.user_id)
            .await?;
    Ok(Json(items))
}

/// Get a group permission by id.
#[utoipa::path(
    get,
    path = "/api/v3/groups-permissions/{id}",
    params(("id" = u32, Path, description = "Permission ID")),
    responses((status = 200, description = "Group permission detail", body = GroupPermissionResponse)),
    tag = "Groups Permissions",
    security(("jwt" = []))
)]
pub async fn get_group_permission(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<Json<GroupPermissionResponse>> {
    let item = crate::service::groups_permissions::get_by_id(&state, id).await?;
    Ok(Json(item))
}

/// Create a new group permission.
#[utoipa::path(
    post,
    path = "/api/v3/groups-permissions",
    request_body = CreateGroupPermissionRequest,
    responses((status = 201, description = "Created group permission", body = GroupPermissionResponse)),
    tag = "Groups Permissions",
    security(("jwt" = []))
)]
pub async fn create_group_permission(
    State(state): State<AppState>,
    Json(req): Json<CreateGroupPermissionRequest>,
) -> AppResult<(axum::http::StatusCode, Json<GroupPermissionResponse>)> {
    let item = crate::service::groups_permissions::create(&state, req).await?;
    Ok((axum::http::StatusCode::CREATED, Json(item)))
}

/// Update a group permission.
#[utoipa::path(
    patch,
    path = "/api/v3/groups-permissions/{id}",
    params(("id" = u32, Path, description = "Permission ID")),
    request_body = UpdateGroupPermissionRequest,
    responses((status = 200, description = "Updated group permission", body = GroupPermissionResponse)),
    tag = "Groups Permissions",
    security(("jwt" = []))
)]
pub async fn update_group_permission(
    Path(id): Path<u32>,
    State(state): State<AppState>,
    Json(req): Json<UpdateGroupPermissionRequest>,
) -> AppResult<Json<GroupPermissionResponse>> {
    let item = crate::service::groups_permissions::update(&state, id, req).await?;
    Ok(Json(item))
}

/// Delete a group permission by id.
#[utoipa::path(
    delete,
    path = "/api/v3/groups-permissions/{id}",
    params(("id" = u32, Path, description = "Permission ID")),
    responses((status = 204, description = "Deleted group permission")),
    tag = "Groups Permissions",
    security(("jwt" = []))
)]
pub async fn delete_group_permission(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<axum::http::StatusCode> {
    crate::service::groups_permissions::delete(&state, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
