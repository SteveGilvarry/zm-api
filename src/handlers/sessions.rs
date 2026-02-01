use crate::dto::request::sessions::{CreateSessionRequest, UpdateSessionRequest};
use crate::dto::response::sessions::PaginatedSessionsResponse;
use crate::dto::response::SessionResponse;
use crate::dto::PaginationParams;
use crate::error::AppResult;
use crate::server::state::AppState;
use axum::{
    extract::{Path, Query, State},
    Json,
};

/// List all sessions with pagination.
///
/// - Requires a valid JWT.
#[utoipa::path(
    get,
    path = "/api/v3/sessions",
    params(
        ("page" = Option<u64>, Query, description = "Page number (1-indexed)", example = 1),
        ("page_size" = Option<u64>, Query, description = "Items per page (max 1000)", example = 25)
    ),
    responses((status = 200, description = "Paginated list of sessions", body = PaginatedSessionsResponse)),
    tag = "Sessions",
    security(("jwt" = []))
)]
pub async fn list_sessions(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> AppResult<Json<PaginatedSessionsResponse>> {
    let result = crate::service::sessions::list_paginated(&state, &params).await?;
    Ok(Json(PaginatedSessionsResponse::from(result)))
}

/// Get a session by id.
///
/// - Requires a valid JWT; responds 404 if not found.
#[utoipa::path(
    get,
    path = "/api/v3/sessions/{id}",
    params(("id" = String, Path, description = "Session ID")),
    responses((status = 200, description = "Session detail", body = SessionResponse)),
    tag = "Sessions",
    security(("jwt" = []))
)]
pub async fn get_session(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> AppResult<Json<SessionResponse>> {
    let item = crate::service::sessions::get_by_id(&state, &id).await?;
    Ok(Json(item))
}

/// Create a new session entry.
///
/// - Requires a valid JWT.
#[utoipa::path(
    post,
    path = "/api/v3/sessions",
    request_body = CreateSessionRequest,
    responses((status = 201, description = "Created session", body = SessionResponse)),
    tag = "Sessions",
    security(("jwt" = []))
)]
pub async fn create_session(
    State(state): State<AppState>,
    Json(req): Json<CreateSessionRequest>,
) -> AppResult<(axum::http::StatusCode, Json<SessionResponse>)> {
    let item = crate::service::sessions::create(&state, req).await?;
    Ok((axum::http::StatusCode::CREATED, Json(item)))
}

/// Update a session entry.
///
/// - Partial update.
/// - Requires a valid JWT.
#[utoipa::path(
    patch,
    path = "/api/v3/sessions/{id}",
    params(("id" = String, Path, description = "Session ID")),
    request_body = UpdateSessionRequest,
    responses((status = 200, description = "Updated session", body = SessionResponse)),
    tag = "Sessions",
    security(("jwt" = []))
)]
pub async fn update_session(
    Path(id): Path<String>,
    State(state): State<AppState>,
    Json(req): Json<UpdateSessionRequest>,
) -> AppResult<Json<SessionResponse>> {
    let item = crate::service::sessions::update(&state, &id, req).await?;
    Ok(Json(item))
}

/// Delete a session by id.
///
/// - Responds 204 on success, 404 if not found.
/// - Requires a valid JWT.
#[utoipa::path(
    delete,
    path = "/api/v3/sessions/{id}",
    params(("id" = String, Path, description = "Session ID")),
    responses((status = 204, description = "Deleted session")),
    tag = "Sessions",
    security(("jwt" = []))
)]
pub async fn delete_session(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> AppResult<axum::http::StatusCode> {
    crate::service::sessions::delete(&state, &id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
