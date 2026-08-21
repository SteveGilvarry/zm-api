use crate::dto::request::CreateUserRequest;
use crate::dto::response::users::PaginatedUsersResponse;
use crate::dto::response::UserResponse;
use crate::dto::PaginationParams;
use crate::error::AppResult;
use crate::server::state::AppState;
use axum::{
    extract::{Path, Query, State},
    Json,
};

/// List ZoneMinder users with pagination.
///
/// - Requires a valid JWT with appropriate permissions.
#[utoipa::path(
    get,
    path = "/api/v3/users",
    params(
        ("page" = Option<u64>, Query, description = "Page number (1-indexed)", example = 1),
        ("page_size" = Option<u64>, Query, description = "Items per page (max 1000)", example = 25)
    ),
    responses((status = 200, description = "Paginated list of users", body = PaginatedUsersResponse)),
    tag = "Users",
    security(("jwt" = []))
)]
pub async fn list_users(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> AppResult<Json<PaginatedUsersResponse>> {
    let result = crate::service::users::list_paginated(&state, &params).await?;
    Ok(Json(PaginatedUsersResponse::from(result)))
}

/// Get a single user by id.
///
/// - Requires a valid JWT.
#[utoipa::path(
    get,
    path = "/api/v3/users/{id}",
    params(("id" = u32, Path, description = "User ID")),
    responses((status = 200, description = "User detail", body = serde_json::Value)),
    tag = "Users",
    security(("jwt" = []))
)]
pub async fn get_user(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<Json<UserResponse>> {
    let item = crate::service::users::get_by_id(&state, id).await?;
    Ok(Json(item))
}

/// Update user fields.
///
/// - Partial update; only provided fields are changed. Can set the profile
///   fields, a new password (re-hashed), the eight permission levels, and the
///   token-revocation floor (admin revoke-all).
/// - Requires a valid JWT with System (admin) permission.
#[utoipa::path(
    put,
    path = "/api/v3/users/{id}",
    params(("id" = u32, Path, description = "User ID")),
    request_body = crate::dto::request::UpdateUserRequest,
    responses((status = 200, description = "Updated user", body = UserResponse)),
    tag = "Users",
    security(("jwt" = []))
)]
pub async fn update_user(
    Path(id): Path<u32>,
    State(state): State<AppState>,
    Json(req): Json<crate::dto::request::UpdateUserRequest>,
) -> AppResult<Json<UserResponse>> {
    let item = crate::service::users::update(&state, id, req).await?;
    Ok(Json(item))
}

/// Create a new user with initial credentials.
///
/// - Sets sensible permission defaults unless explicitly configured.
/// - Requires a valid JWT with admin permissions.
#[utoipa::path(
    post,
    path = "/api/v3/users",
    request_body = CreateUserRequest,
    responses((status = 201, description = "Created user", body = UserResponse)),
    tag = "Users",
    security(("jwt" = []))
)]
pub async fn create_user(
    State(state): State<AppState>,
    Json(req): Json<CreateUserRequest>,
) -> AppResult<(axum::http::StatusCode, Json<UserResponse>)> {
    let item = crate::service::users::create(&state, req).await?;
    Ok((axum::http::StatusCode::CREATED, Json(item)))
}

/// Delete a user by id.
///
/// - Responds 204 on success, 404 if not found.
/// - Requires a valid JWT with admin permissions.
#[utoipa::path(
    delete,
    path = "/api/v3/users/{id}",
    params(("id" = u32, Path, description = "User ID")),
    responses((status = 204, description = "Deleted user")),
    tag = "Users",
    security(("jwt" = []))
)]
pub async fn delete_user(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<axum::http::StatusCode> {
    crate::service::users::delete(&state, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
