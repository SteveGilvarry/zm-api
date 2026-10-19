//! `/api/v3/roles`: ZoneMinder 1.39 roles (#143). System-gated, like users.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use garde::Validate;

use crate::dto::request::roles::{CreateRoleRequest, UpdateRoleRequest};
use crate::dto::response::roles::{PaginatedRolesResponse, RoleResponse};
use crate::dto::PaginationParams;
use crate::error::{AppError, AppResponseError, AppResult};
use crate::server::state::AppState;

/// List roles
#[utoipa::path(
    get,
    path = "/api/v3/roles",
    params(
        ("page" = Option<u64>, Query, description = "Page number (1-indexed)", example = 1),
        ("page_size" = Option<u64>, Query, description = "Items per page (max 1000)", example = 25)
    ),
    responses((status = 200, description = "Paginated list of roles", body = PaginatedRolesResponse)),
    tag = "Users",
    security(("jwt" = []))
)]
pub async fn list_roles(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> AppResult<Json<PaginatedRolesResponse>> {
    Ok(Json(
        crate::service::roles::list(&state, &params).await?.into(),
    ))
}

/// Get a role
#[utoipa::path(
    get,
    path = "/api/v3/roles/{id}",
    params(("id" = u32, Path, description = "Role ID")),
    responses(
        (status = 200, description = "Role", body = RoleResponse),
        (status = 404, description = "Role not found", body = AppResponseError)
    ),
    tag = "Users",
    security(("jwt" = []))
)]
pub async fn get_role(
    State(state): State<AppState>,
    Path(id): Path<u32>,
) -> AppResult<Json<RoleResponse>> {
    Ok(Json(crate::service::roles::get(&state, id).await?))
}

/// Create a role
#[utoipa::path(
    post,
    path = "/api/v3/roles",
    request_body = CreateRoleRequest,
    responses((status = 201, description = "Created role", body = RoleResponse)),
    tag = "Users",
    security(("jwt" = []))
)]
pub async fn create_role(
    State(state): State<AppState>,
    Json(req): Json<CreateRoleRequest>,
) -> AppResult<(StatusCode, Json<RoleResponse>)> {
    req.validate().map_err(AppError::InvalidInputError)?;
    Ok((
        StatusCode::CREATED,
        Json(crate::service::roles::create(&state, req).await?),
    ))
}

/// Update a role
#[utoipa::path(
    put,
    path = "/api/v3/roles/{id}",
    params(("id" = u32, Path, description = "Role ID")),
    request_body = UpdateRoleRequest,
    responses(
        (status = 200, description = "Updated role", body = RoleResponse),
        (status = 404, description = "Role not found", body = AppResponseError)
    ),
    tag = "Users",
    security(("jwt" = []))
)]
pub async fn update_role(
    State(state): State<AppState>,
    Path(id): Path<u32>,
    Json(req): Json<UpdateRoleRequest>,
) -> AppResult<Json<RoleResponse>> {
    req.validate().map_err(AppError::InvalidInputError)?;
    Ok(Json(crate::service::roles::update(&state, id, req).await?))
}

/// Delete a role. Its users keep their own permission levels.
#[utoipa::path(
    delete,
    path = "/api/v3/roles/{id}",
    params(("id" = u32, Path, description = "Role ID")),
    responses(
        (status = 204, description = "Deleted"),
        (status = 404, description = "Role not found", body = AppResponseError)
    ),
    tag = "Users",
    security(("jwt" = []))
)]
pub async fn delete_role(
    State(state): State<AppState>,
    Path(id): Path<u32>,
) -> AppResult<StatusCode> {
    crate::service::roles::delete(&state, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
