use crate::dto::request::user_preferences::{
    CreateUserPreferenceRequest, UpdateUserPreferenceRequest,
};
use crate::dto::response::user_preferences::PaginatedUserPreferencesResponse;
use crate::dto::response::UserPreferenceResponse;
use crate::dto::PaginationParams;
use crate::error::AppResult;
use crate::server::state::AppState;
use axum::{
    extract::{Path, Query, State},
    Json,
};

/// List user preferences with pagination.
///
/// - Requires a valid JWT.
#[utoipa::path(
    get,
    path = "/api/v3/user_preferences",
    params(
        ("page" = Option<u64>, Query, description = "Page number (1-indexed)", example = 1),
        ("page_size" = Option<u64>, Query, description = "Items per page (max 1000)", example = 25)
    ),
    responses((status = 200, description = "Paginated list of user preferences", body = PaginatedUserPreferencesResponse)),
    tag = "User Preferences",
    security(("jwt" = []))
)]
pub async fn list_user_preferences(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> AppResult<Json<PaginatedUserPreferencesResponse>> {
    let result = crate::service::user_preferences::list_paginated(&state, &params).await?;
    Ok(Json(PaginatedUserPreferencesResponse::from(result)))
}

/// Get a user preference by id.
///
/// - Requires a valid JWT; responds 404 if not found.
#[utoipa::path(
    get,
    path = "/api/v3/user_preferences/{id}",
    params(("id" = u32, Path, description = "User Preference ID")),
    responses((status = 200, description = "User preference detail", body = UserPreferenceResponse)),
    tag = "User Preferences",
    security(("jwt" = []))
)]
pub async fn get_user_preference(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<Json<UserPreferenceResponse>> {
    let item = crate::service::user_preferences::get_by_id(&state, id).await?;
    Ok(Json(item))
}

/// Create a new user preference entry.
///
/// - Requires a valid JWT.
#[utoipa::path(
    post,
    path = "/api/v3/user_preferences",
    request_body = CreateUserPreferenceRequest,
    responses((status = 201, description = "Created user preference", body = UserPreferenceResponse)),
    tag = "User Preferences",
    security(("jwt" = []))
)]
pub async fn create_user_preference(
    State(state): State<AppState>,
    Json(req): Json<CreateUserPreferenceRequest>,
) -> AppResult<(axum::http::StatusCode, Json<UserPreferenceResponse>)> {
    let item = crate::service::user_preferences::create(&state, req).await?;
    Ok((axum::http::StatusCode::CREATED, Json(item)))
}

/// Update a user preference entry.
///
/// - Partial update.
/// - Requires a valid JWT.
#[utoipa::path(
    patch,
    path = "/api/v3/user_preferences/{id}",
    params(("id" = u32, Path, description = "User Preference ID")),
    request_body = UpdateUserPreferenceRequest,
    responses((status = 200, description = "Updated user preference", body = UserPreferenceResponse)),
    tag = "User Preferences",
    security(("jwt" = []))
)]
pub async fn update_user_preference(
    Path(id): Path<u32>,
    State(state): State<AppState>,
    Json(req): Json<UpdateUserPreferenceRequest>,
) -> AppResult<Json<UserPreferenceResponse>> {
    let item = crate::service::user_preferences::update(&state, id, req).await?;
    Ok(Json(item))
}

/// Delete a user preference by id.
///
/// - Responds 204 on success, 404 if not found.
/// - Requires a valid JWT.
#[utoipa::path(
    delete,
    path = "/api/v3/user_preferences/{id}",
    params(("id" = u32, Path, description = "User Preference ID")),
    responses((status = 204, description = "Deleted user preference")),
    tag = "User Preferences",
    security(("jwt" = []))
)]
pub async fn delete_user_preference(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<axum::http::StatusCode> {
    crate::service::user_preferences::delete(&state, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
