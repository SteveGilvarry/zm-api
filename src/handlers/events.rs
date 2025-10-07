use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
// use chrono::Utc; // not needed in this module
use garde::Validate;
use tracing::{info, instrument};

use crate::{
    dto::{
        request::events::{EventCreateRequest, EventQueryParams, EventUpdateRequest},
        response::events::{EventCountsResponse, EventResponse, PaginatedEventsResponse},
    },
    error::{AppError, AppResult, AppResponseError},
    server::state::AppState,
    service,
};


/// Get a paginated list of events
#[utoipa::path(
    get,
    path = "/api/v3/events",
    operation_id = "listEvents",
    tag = "Events",
    params(
        ("page" = Option<u64>, Query, description = "Page number (1-indexed)", example = 1),
        ("page_size" = Option<u64>, Query, description = "Number of items per page", example = 20),
        ("monitor_id" = Option<u32>, Query, description = "Filter by monitor ID", example = 1),
        ("start_time" = Option<String>, Query, description = "Filter by start time (ISO8601)", example = "2025-04-28T00:00:00Z"),
        ("end_time" = Option<String>, Query, description = "Filter by end time (ISO8601)", example = "2025-04-29T23:59:59Z")
    ),
    responses(
        (status = 200, description = "List of events", body = PaginatedEventsResponse),
        (status = 400, description = "Bad request", body = AppResponseError),
        (status = 401, description = "Unauthorized", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(
        ("jwt" = [])
    )
)]
#[instrument(skip(state))]
pub async fn list_events(
    State(state): State<AppState>,
    Query(params): Query<EventQueryParams>,
) -> AppResult<Json<PaginatedEventsResponse>> {
    info!("Listing events with params: {:?}", params);
    
    if let Some(monitor_id) = params.monitor_id {
        let events = service::events::list_by_monitor(
            &state,
            monitor_id,
            params.page,
            params.page_size,
            params.start_time.map(|dt| dt.0),
            params.end_time.map(|dt| dt.0),
        ).await?;
        
        Ok(Json(events))
    } else {
        let events = service::events::list_all(&state, params.page, params.page_size).await?;
        
        Ok(Json(events))
    }
}

/// Get a specific event by ID
#[utoipa::path(
    get,
    path = "/api/v3/events/{id}",
    operation_id = "getEvent",
    tag = "Events",
    params(
        ("id" = u32, Path, description = "Event ID")
    ),
    responses(
        (status = 200, description = "Event details", body = EventResponse),
        (status = 404, description = "Event not found", body = AppResponseError),
        (status = 401, description = "Unauthorized", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(
        ("jwt" = [])
    )
)]
#[instrument(skip(state))]
pub async fn get_event(
    State(state): State<AppState>,
    Path(id): Path<u32>,
) -> AppResult<Json<EventResponse>> {
    let event = service::events::get_by_id(&state, id).await?;
    
    Ok(Json(event))
}

/// Create a new event
#[utoipa::path(
    post,
    path = "/api/v3/events",
    operation_id = "createEvent",
    tag = "Events",
    request_body = EventCreateRequest,
    responses(
        (status = 201, description = "Event created", body = EventResponse),
        (status = 400, description = "Invalid request", body = AppResponseError),
        (status = 401, description = "Unauthorized", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(
        ("jwt" = [])
    )
)]
#[instrument(skip(state, event))]
pub async fn create_event(
    State(state): State<AppState>,
    Json(event): Json<EventCreateRequest>,
) -> AppResult<(StatusCode, Json<EventResponse>)> {
    event.validate().map_err(AppError::InvalidInputError)?;
    
    let new_event = service::events::create(&state, event).await?;
    
    Ok((StatusCode::CREATED, Json(new_event)))
}

/// Update an existing event
#[utoipa::path(
    patch,
    path = "/api/v3/events/{id}",
    operation_id = "updateEvent",
    tag = "Events",
    params(
        ("id" = u32, Path, description = "Event ID")
    ),
    request_body = EventUpdateRequest,
    responses(
        (status = 200, description = "Event updated", body = EventResponse),
        (status = 404, description = "Event not found", body = AppResponseError),
        (status = 400, description = "Invalid request", body = AppResponseError),
        (status = 401, description = "Unauthorized", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(
        ("jwt" = [])
    )
)]
#[instrument(skip(state, event_update))]
pub async fn update_event(
    State(state): State<AppState>,
    Path(id): Path<u32>,
    Json(event_update): Json<EventUpdateRequest>,
) -> AppResult<Json<EventResponse>> {
    event_update.validate().map_err(AppError::InvalidInputError)?;
    
    let updated_event = service::events::update(&state, id, event_update).await?;
    
    Ok(Json(updated_event))
}

/// Delete an event
#[utoipa::path(
    delete,
    path = "/api/v3/events/{id}",
    operation_id = "deleteEvent",
    tag = "Events",
    params(
        ("id" = u32, Path, description = "Event ID")
    ),
    responses(
        (status = 204, description = "Event deleted"),
        (status = 401, description = "Unauthorized", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(
        ("jwt" = [])
    )
)]
#[instrument(skip(state))]
pub async fn delete_event(
    State(state): State<AppState>,
    Path(id): Path<u32>,
) -> AppResult<StatusCode> {
    service::events::delete(&state, id).await?;
    
    Ok(StatusCode::NO_CONTENT)
}

/// Get event counts
#[utoipa::path(
    get,
    path = "/api/v3/events/counts/{hours}",
    operation_id = "getEventCounts",
    tag = "Events",
    params(
        ("hours" = i64, Path, description = "Number of hours to count back from now")
    ),
    responses(
        (status = 200, description = "Event counts", body = EventCountsResponse),
        (status = 400, description = "Bad request", body = AppResponseError),
        (status = 401, description = "Unauthorized", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(
        ("jwt" = [])
    )
)]
#[instrument(skip(state))]
pub async fn get_event_counts(
    State(state): State<AppState>,
    Path(hours): Path<i64>,
) -> AppResult<Json<EventCountsResponse>> {
    let counts = service::events::get_event_counts(&state, hours).await?;
    
    Ok(Json(counts))
}
