use crate::dto::request::frames::{CreateFrameRequest, UpdateFrameRequest};
use crate::dto::response::frames::{FrameResponse, PaginatedFramesResponse};
use crate::dto::PaginationParams;
use crate::error::AppResult;
use crate::server::state::AppState;
use crate::service;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct FrameQueryParams {
    /// Filter frames by event ID
    pub event_id: Option<u64>,
    /// Page number (1-indexed, defaults to 1)
    pub page: Option<u64>,
    /// Number of items per page (defaults to 25, max 1000)
    pub page_size: Option<u64>,
}

/// List frames with pagination, optionally filtered by event_id
#[utoipa::path(
    get,
    path = "/api/v3/frames",
    params(
        ("event_id" = Option<u64>, Query, description = "Filter by event"),
        ("page" = Option<u64>, Query, description = "Page number (1-indexed)", example = 1),
        ("page_size" = Option<u64>, Query, description = "Items per page (max 1000)", example = 25)
    ),
    responses(
        (status = 200, description = "Paginated list of frames", body = PaginatedFramesResponse)
    ),
    tag = "Frames",
    summary = "List frames; optionally filter by event id.",
    description = "- Requires a valid JWT.",
    security(("jwt" = []))
)]
pub async fn list_frames(
    State(state): State<AppState>,
    Query(params): Query<FrameQueryParams>,
) -> AppResult<Json<PaginatedFramesResponse>> {
    let pagination = PaginationParams {
        page: params.page,
        page_size: params.page_size,
    };
    let result = service::frames::list_paginated(&state, params.event_id, &pagination).await?;
    Ok(Json(PaginatedFramesResponse::from(result)))
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
