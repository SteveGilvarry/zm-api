use crate::dto::request::sessions::{CreateSessionRequest, UpdateSessionRequest};
use crate::dto::response::SessionResponse;
use crate::error::AppResult;
use crate::server::state::AppState;
use axum::{
    extract::{Path, State},
    Json,
};

/// List all sessions.
///
/// - Requires a valid JWT.
#[utoipa::path(
    get,
    path = "/api/v3/sessions",
    responses((status = 200, description = "List sessions", body = [SessionResponse])),
    tag = "Sessions",
    security(("jwt" = []))
)]
pub async fn list_sessions(State(state): State<AppState>) -> AppResult<Json<Vec<SessionResponse>>> {
    let items = crate::service::sessions::list_all(&state).await?;
    Ok(Json(items))
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
