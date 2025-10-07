use axum::{extract::{Path, State}, Json};
use serde::Deserialize;
use crate::dto::response::UserResponse;
use crate::dto::request::CreateUserRequest;
use crate::error::AppResult;
use crate::server::state::AppState;

/// List ZoneMinder users.
///
/// - Requires a valid JWT with appropriate permissions.
#[utoipa::path(
    get,
    path = "/api/v3/users",
    responses((status = 200, description = "List users", body = serde_json::Value)),
    tag = "Users",
    security(("jwt" = []))
)]
pub async fn list_users(State(state): State<AppState>) -> AppResult<Json<Vec<UserResponse>>> {
    let items = crate::service::users::list_all(&state).await?;
    Ok(Json(items))
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
pub async fn get_user(Path(id): Path<u32>, State(state): State<AppState>) -> AppResult<Json<UserResponse>> {
    let item = crate::service::users::get_by_id(&state, id).await?;
    Ok(Json(item))
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateUserRequest { pub email: Option<String>, pub enabled: Option<u8> }

/// Update user fields (email/enabled).
///
/// - Partial update; only provided fields are changed.
/// - Requires a valid JWT.
#[utoipa::path(
    put,
    path = "/api/v3/users/{id}",
    params(("id" = u32, Path, description = "User ID")),
    request_body = UpdateUserRequest,
    responses((status = 200, description = "Updated user", body = serde_json::Value)),
    tag = "Users",
    security(("jwt" = []))
)]
pub async fn update_user(Path(id): Path<u32>, State(state): State<AppState>, Json(req): Json<UpdateUserRequest>) -> AppResult<Json<UserResponse>> {
    let item = crate::service::users::update(&state, id, req.email, req.enabled).await?;
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
pub async fn create_user(State(state): State<AppState>, Json(req): Json<CreateUserRequest>) -> AppResult<(axum::http::StatusCode, Json<UserResponse>)> {
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
pub async fn delete_user(Path(id): Path<u32>, State(state): State<AppState>) -> AppResult<axum::http::StatusCode> {
    crate::service::users::delete(&state, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
