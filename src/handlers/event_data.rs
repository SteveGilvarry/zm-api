use crate::dto::request::event_data::{CreateEventDataRequest, UpdateEventDataRequest};
use crate::dto::response::event_data::PaginatedEventDataResponse;
use crate::dto::response::EventDataResponse;
use crate::dto::PaginationParams;
use crate::error::AppResult;
use crate::server::state::AppState;
use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct EventDataQuery {
    event_id: Option<u64>,
    #[serde(flatten)]
    pagination: PaginationParams,
}

/// List all event data with pagination.
#[utoipa::path(
    get,
    path = "/api/v3/event-data",
    params(
        ("event_id" = Option<u64>, Query, description = "Filter by event ID"),
        ("page" = Option<u64>, Query, description = "Page number (1-indexed)", example = 1),
        ("page_size" = Option<u64>, Query, description = "Items per page (max 1000)", example = 25)
    ),
    responses((status = 200, description = "Paginated list of event data", body = PaginatedEventDataResponse)),
    tag = "Event Data",
    security(("jwt" = []))
)]
pub async fn list_event_data(
    Query(params): Query<EventDataQuery>,
    State(state): State<AppState>,
) -> AppResult<Json<PaginatedEventDataResponse>> {
    let result =
        crate::service::event_data::list_paginated(&state, &params.pagination, params.event_id)
            .await?;
    Ok(Json(PaginatedEventDataResponse::from(result)))
}

/// Get event data by id.
#[utoipa::path(
    get,
    path = "/api/v3/event-data/{id}",
    params(("id" = u64, Path, description = "Event Data ID")),
    responses((status = 200, description = "Event data detail", body = EventDataResponse)),
    tag = "Event Data",
    security(("jwt" = []))
)]
pub async fn get_event_data(
    Path(id): Path<u64>,
    State(state): State<AppState>,
) -> AppResult<Json<EventDataResponse>> {
    let item = crate::service::event_data::get_by_id(&state, id).await?;
    Ok(Json(item))
}

/// Create new event data.
#[utoipa::path(
    post,
    path = "/api/v3/event-data",
    request_body = CreateEventDataRequest,
    responses((status = 201, description = "Created event data", body = EventDataResponse)),
    tag = "Event Data",
    security(("jwt" = []))
)]
pub async fn create_event_data(
    State(state): State<AppState>,
    Json(req): Json<CreateEventDataRequest>,
) -> AppResult<(axum::http::StatusCode, Json<EventDataResponse>)> {
    let item = crate::service::event_data::create(&state, req).await?;
    Ok((axum::http::StatusCode::CREATED, Json(item)))
}

/// Update event data.
#[utoipa::path(
    patch,
    path = "/api/v3/event-data/{id}",
    params(("id" = u64, Path, description = "Event Data ID")),
    request_body = UpdateEventDataRequest,
    responses((status = 200, description = "Updated event data", body = EventDataResponse)),
    tag = "Event Data",
    security(("jwt" = []))
)]
pub async fn update_event_data(
    Path(id): Path<u64>,
    State(state): State<AppState>,
    Json(req): Json<UpdateEventDataRequest>,
) -> AppResult<Json<EventDataResponse>> {
    let item = crate::service::event_data::update(&state, id, req).await?;
    Ok(Json(item))
}

/// Delete event data by id.
#[utoipa::path(
    delete,
    path = "/api/v3/event-data/{id}",
    params(("id" = u64, Path, description = "Event Data ID")),
    responses((status = 204, description = "Deleted event data")),
    tag = "Event Data",
    security(("jwt" = []))
)]
pub async fn delete_event_data(
    Path(id): Path<u64>,
    State(state): State<AppState>,
) -> AppResult<axum::http::StatusCode> {
    crate::service::event_data::delete(&state, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
