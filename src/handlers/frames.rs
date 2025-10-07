use crate::server::state::AppState;
use crate::dto::request::frames::{CreateFrameRequest, UpdateFrameRequest};
use crate::dto::response::frames::FrameResponse;
use crate::error::AppResult;
use crate::service;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct FrameQueryParams {
    /// Filter frames by event ID
    pub event_id: Option<u64>,
}

/// List frames, optionally filtered by event_id
#[utoipa::path(
    get,
    path = "/api/v3/frames",
    params(
        ("event_id" = Option<u64>, Query, description = "Filter by event")
    ),
    responses(
        (status = 200, description = "List frames", body = Vec<FrameResponse>)
    ),
    tag = "Frames",
    summary = "List frames; optionally filter by event id.",
    description = "- Requires a valid JWT.",
    security(("jwt" = []))
)]
pub async fn list_frames(
    State(state): State<AppState>,
    Query(params): Query<FrameQueryParams>,
) -> AppResult<Json<Vec<FrameResponse>>> {
    let frames = service::frames::list_all(&state, params.event_id).await?;
    Ok(Json(frames))
}

/// Get frame by id
#[utoipa::path(
    get,
    path = "/api/v3/frames/{id}",
    params(
        ("id" = u64, Path, description = "Frame ID")
    ),
    responses(
        (status = 200, description = "Frame detail", body = FrameResponse)
    ),
    tag = "Frames",
    summary = "Get a frame by id.",
    description = "- Requires a valid JWT; responds 404 if not found.",
    security(("jwt" = []))
)]
pub async fn get_frame(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> AppResult<Json<FrameResponse>> {
    let frame = service::frames::get_by_id(&state, id).await?;
    Ok(Json(frame))
}

/// Create a new frame
#[utoipa::path(
    post,
    path = "/api/v3/frames",
    request_body = CreateFrameRequest,
    responses(
        (status = 201, description = "Created frame", body = FrameResponse)
    ),
    tag = "Frames",
    summary = "Create a new frame entry.",
    description = "- Requires a valid JWT.",
    security(("jwt" = []))
)]
pub async fn create_frame(
    State(state): State<AppState>,
    Json(req): Json<CreateFrameRequest>,
) -> AppResult<(StatusCode, Json<FrameResponse>)> {
    let frame = service::frames::create(&state, req).await?;
    Ok((StatusCode::CREATED, Json(frame)))
}

/// Update frame by id
#[utoipa::path(
    patch,
    path = "/api/v3/frames/{id}",
    params(
        ("id" = u64, Path, description = "Frame ID")
    ),
    request_body = UpdateFrameRequest,
    responses(
        (status = 200, description = "Updated frame", body = FrameResponse)
    ),
    tag = "Frames",
    summary = "Update a frame entry.",
    description = "- Partial update.\n- Requires a valid JWT.",
    security(("jwt" = []))
)]
pub async fn update_frame(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Json(req): Json<UpdateFrameRequest>,
) -> AppResult<Json<FrameResponse>> {
    let frame = service::frames::update(&state, id, req).await?;
    Ok(Json(frame))
}

/// Delete frame by id
#[utoipa::path(
    delete,
    path = "/api/v3/frames/{id}",
    params(
        ("id" = u64, Path, description = "Frame ID")
    ),
    responses(
        (status = 204, description = "Deleted frame")
    ),
    tag = "Frames",
    summary = "Delete a frame by id.",
    description = "- Responds 204 on success, 404 if not found.\n- Requires a valid JWT.",
    security(("jwt" = []))
)]
pub async fn delete_frame(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> AppResult<StatusCode> {
    service::frames::delete(&state, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
