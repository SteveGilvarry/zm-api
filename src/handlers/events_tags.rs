use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;

use crate::dto::request::events_tags::{CreateEventTagRequest, EventTagQuery};
use crate::dto::response::events_tags::PaginatedEventsTagsResponse;
use crate::dto::response::EventTagResponse;
use crate::dto::PaginationParams;
use crate::error::AppResult;
use crate::server::state::AppState;

/// List all event-tag associations with pagination.
#[utoipa::path(
    get,
    path = "/api/v3/events-tags",
    params(
        ("event_id" = Option<u64>, Query, description = "Filter by event ID"),
        ("tag_id" = Option<u64>, Query, description = "Filter by tag ID"),
        ("page" = Option<u64>, Query, description = "Page number (1-indexed)", example = 1),
        ("page_size" = Option<u64>, Query, description = "Items per page (max 1000)", example = 25)
    ),
    responses((status = 200, description = "Paginated list of event-tag associations", body = PaginatedEventsTagsResponse)),
    tag = "Events Tags",
    security(("jwt" = []))
)]
pub async fn list_events_tags(
    Query(filter): Query<EventTagQuery>,
    Query(params): Query<PaginationParams>,
    State(state): State<AppState>,
) -> AppResult<Json<PaginatedEventsTagsResponse>> {
    let result = crate::service::events_tags::list_paginated(
        &state,
        filter.event_id,
        filter.tag_id,
        &params,
    )
    .await?;
    Ok(Json(PaginatedEventsTagsResponse::from(result)))
}

/// Get an event-tag association by composite ID.
#[utoipa::path(
    get,
    path = "/api/v3/events-tags/{tag_id}/{event_id}",
    params(
        ("tag_id" = u64, Path, description = "Tag ID"),
        ("event_id" = u64, Path, description = "Event ID")
    ),
    responses((status = 200, description = "Event-tag association detail", body = EventTagResponse)),
    tag = "Events Tags",
    security(("jwt" = []))
)]
pub async fn get_event_tag(
    Path((tag_id, event_id)): Path<(u64, u64)>,
    State(state): State<AppState>,
) -> AppResult<Json<EventTagResponse>> {
    let item = crate::service::events_tags::get_by_id(&state, tag_id, event_id).await?;
    Ok(Json(item))
}

/// Create a new event-tag association.
#[utoipa::path(
    post,
    path = "/api/v3/events-tags",
    request_body = CreateEventTagRequest,
    responses((status = 201, description = "Created event-tag association", body = EventTagResponse)),
    tag = "Events Tags",
    security(("jwt" = []))
)]
pub async fn create_event_tag(
    State(state): State<AppState>,
    Json(req): Json<CreateEventTagRequest>,
) -> AppResult<(StatusCode, Json<EventTagResponse>)> {
    let item = crate::service::events_tags::create(&state, req).await?;
    Ok((StatusCode::CREATED, Json(item)))
}

/// Delete an event-tag association by composite ID.
#[utoipa::path(
    delete,
    path = "/api/v3/events-tags/{tag_id}/{event_id}",
    params(
        ("tag_id" = u64, Path, description = "Tag ID"),
        ("event_id" = u64, Path, description = "Event ID")
    ),
    responses((status = 204, description = "Deleted event-tag association")),
    tag = "Events Tags",
    security(("jwt" = []))
)]
pub async fn delete_event_tag(
    Path((tag_id, event_id)): Path<(u64, u64)>,
    State(state): State<AppState>,
) -> AppResult<StatusCode> {
    crate::service::events_tags::delete(&state, tag_id, event_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
