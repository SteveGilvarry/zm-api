use axum::extract::{Path, Query, State};
use axum::Json;

use crate::dto::request::tags::{CreateTagRequest, TagDetailQuery, UpdateTagRequest};
use crate::dto::request::PaginationParams;
use crate::dto::response::events_tags::TagDetailResponse;
use crate::dto::response::{PaginatedResponse, TagResponse};
use crate::error::AppResult;
use crate::server::state::AppState;

/// List all tags with pagination.
///
/// - Requires a valid JWT.
/// - Returns event_count for each tag.
#[utoipa::path(
    get,
    path = "/api/v3/tags",
    params(
        ("page" = Option<u64>, Query, description = "Page number (1-indexed)", example = 1),
        ("page_size" = Option<u64>, Query, description = "Number of items per page (max 1000)", example = 20)
    ),
    responses((status = 200, description = "Paginated list of tags with event counts", body = PaginatedResponse<TagResponse>)),
    tag = "Tags",
    security(("jwt" = []))
)]
pub async fn list_tags(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> AppResult<Json<PaginatedResponse<TagResponse>>> {
    let items = crate::service::tags::list_all_paginated(&state, &params).await?;
    Ok(Json(items))
}

/// Get a tag by id with paginated events.
///
/// - Requires a valid JWT; responds 404 if not found.
/// - Returns associated events with pagination.
#[utoipa::path(
    get,
    path = "/api/v3/tags/{id}",
    params(
        ("id" = u64, Path, description = "Tag ID"),
        ("page" = Option<u64>, Query, description = "Page number (1-indexed)"),
        ("page_size" = Option<u64>, Query, description = "Items per page (default 20)")
    ),
    responses((status = 200, description = "Tag detail with paginated events", body = TagDetailResponse)),
    tag = "Tags",
    security(("jwt" = []))
)]
pub async fn get_tag(
    Path(id): Path<u64>,
    Query(params): Query<TagDetailQuery>,
    State(state): State<AppState>,
) -> AppResult<Json<TagDetailResponse>> {
    let item =
        crate::service::tags::get_by_id_with_events(&state, id, params.page, params.page_size)
            .await?;
    Ok(Json(item))
}

/// Create a new tag entry.
///
/// - Requires a valid JWT.
/// - Name must be unique.
#[utoipa::path(
    post,
    path = "/api/v3/tags",
    request_body = CreateTagRequest,
    responses((status = 201, description = "Created tag", body = TagResponse)),
    tag = "Tags",
    security(("jwt" = []))
)]
pub async fn create_tag(
    State(state): State<AppState>,
    Json(req): Json<CreateTagRequest>,
) -> AppResult<(axum::http::StatusCode, Json<TagResponse>)> {
    let item = crate::service::tags::create(&state, req).await?;
    Ok((axum::http::StatusCode::CREATED, Json(item)))
}

/// Update a tag entry.
///
/// - Partial update.
/// - Requires a valid JWT.
#[utoipa::path(
    patch,
    path = "/api/v3/tags/{id}",
    params(("id" = u64, Path, description = "Tag ID")),
    request_body = UpdateTagRequest,
    responses((status = 200, description = "Updated tag", body = TagResponse)),
    tag = "Tags",
    security(("jwt" = []))
)]
pub async fn update_tag(
    Path(id): Path<u64>,
    State(state): State<AppState>,
    Json(req): Json<UpdateTagRequest>,
) -> AppResult<Json<TagResponse>> {
    let item = crate::service::tags::update(&state, id, req).await?;
    Ok(Json(item))
}

/// Delete a tag by id.
///
/// - Responds 204 on success, 404 if not found.
/// - Requires a valid JWT.
#[utoipa::path(
    delete,
    path = "/api/v3/tags/{id}",
    params(("id" = u64, Path, description = "Tag ID")),
    responses((status = 204, description = "Deleted tag")),
    tag = "Tags",
    security(("jwt" = []))
)]
pub async fn delete_tag(
    Path(id): Path<u64>,
    State(state): State<AppState>,
) -> AppResult<axum::http::StatusCode> {
    crate::service::tags::delete(&state, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
