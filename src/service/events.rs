use chrono::{DateTime, Utc};
use sea_orm::ActiveValue::Set;
use tracing::instrument;

use crate::{
    dto::{
        request::events::{EventCreateRequest, EventUpdateRequest},
        response::events::{EventCountResponse, EventCountsResponse, EventResponse, PaginatedEventsResponse},
    },
    entity::{events},
    error::{AppResult, AppError},
    repo::events as events_repo,
    server::state::AppState,
};

/// Default page size for paginated event listing
const DEFAULT_PAGE_SIZE: u64 = 20;

/// List all events with pagination
#[instrument(skip(state))]
pub async fn list_all(
    state: &AppState,
    page: Option<u64>,
    page_size: Option<u64>,
) -> AppResult<PaginatedEventsResponse> {
    let page = page.unwrap_or(1);
    let page_size = page_size.unwrap_or(DEFAULT_PAGE_SIZE);

    let (events, total) = events_repo::find_all(state, page - 1, page_size).await?;

    let total_pages = (total + page_size - 1) / page_size; // Ceiling division

    let event_responses = events.into_iter().map(EventResponse::from).collect();

    Ok(PaginatedEventsResponse {
        events: event_responses,
        total,
        per_page: page_size,
        current_page: page,
        last_page: total_pages,
    })
}

/// List events by monitor ID with pagination
#[instrument(skip(state))]
pub async fn list_by_monitor(
    state: &AppState,
    monitor_id: u32,
    page: Option<u64>,
    page_size: Option<u64>,
    start_time: Option<DateTime<Utc>>,
    end_time: Option<DateTime<Utc>>,
) -> AppResult<PaginatedEventsResponse> {
    let page = page.unwrap_or(1);
    let page_size = page_size.unwrap_or(DEFAULT_PAGE_SIZE);

    let (events, total) = events_repo::find_by_monitor_id(
        state,
        monitor_id,
        start_time.map(|dt| dt.naive_utc()),
        end_time.map(|dt| dt.naive_utc()),
        page - 1,
        page_size,
    )
    .await?;

    let total_pages = (total + page_size - 1) / page_size; // Ceiling division

    let event_responses = events.into_iter().map(EventResponse::from).collect();

    Ok(PaginatedEventsResponse {
        events: event_responses,
        total,
        per_page: page_size,
        current_page: page,
        last_page: total_pages,
    })
}

/// Get event by ID
#[instrument(skip(state))]
pub async fn get_by_id(state: &AppState, id: u32) -> AppResult<EventResponse> {
    let event = events_repo::find_by_id(state, id as u64).await?
        .ok_or_else(|| AppError::NotFoundError(crate::error::Resource {
            details: vec![("id".to_string(), id.to_string())],
            resource_type: crate::error::ResourceType::Message,
        }))?;
    Ok(EventResponse::from(event))
}

/// Create a new event
#[instrument(skip(state))]
pub async fn create(state: &AppState, event: EventCreateRequest) -> AppResult<EventResponse> {
    // Convert EventCreateRequest to ActiveModel for database insertion
    let active_event = events::ActiveModel {
        monitor_id: Set(event.monitor_id),
        storage_id: Set(event.storage_id),
        secondary_storage_id: Set(event.secondary_storage_id),
        name: Set(event.name),
        cause: Set(event.cause),
        start_date_time: Set(event.start_date_time.map(|dt| dt.0.naive_utc())),
        end_date_time: Set(event.end_date_time.map(|dt| dt.0.naive_utc())),
        width: Set(event.width),
        height: Set(event.height),
        length: Set(event.length.0),
        notes: Set(event.notes),
        orientation: Set(event.orientation),
        ..Default::default()
    };

    let saved_event = events_repo::create(state, active_event).await?;
    Ok(EventResponse::from(saved_event))
}

/// Update an existing event
#[instrument(skip(state))]
pub async fn update(
    state: &AppState,
    id: u32,
    event_update: EventUpdateRequest,
) -> AppResult<EventResponse> {
    let event = events_repo::find_by_id(state, id as u64).await?
        .ok_or_else(|| AppError::NotFoundError(crate::error::Resource {
            details: vec![("id".to_string(), id.to_string())],
            resource_type: crate::error::ResourceType::Message,
        }))?;
    let mut active_event = events::ActiveModel::from(event);

    // Only update fields that are present in the request
    if let Some(name) = event_update.name {
        active_event.name = Set(name);
    }

    if let Some(cause) = event_update.cause {
        active_event.cause = Set(Some(cause));
    }

    if let Some(notes) = event_update.notes {
        active_event.notes = Set(Some(notes));
    }

    if let Some(orientation) = event_update.orientation {
        active_event.orientation = Set(orientation);
    }

    let updated_event = events_repo::update(state, active_event).await?;
    Ok(EventResponse::from(updated_event))
}

/// Delete an event
#[instrument(skip(state))]
pub async fn delete(state: &AppState, id: u32) -> AppResult<()> {
    events_repo::delete(state, id as u64).await?;
    Ok(())
}

/// Get event counts for the last n hours
#[instrument(skip(state))]
pub async fn get_event_counts(state: &AppState, hours: i64) -> AppResult<EventCountsResponse> {
    let counts = events_repo::get_counts_by_hour(state, hours).await?;
    
    let event_counts = counts
        .into_iter()
        .map(|(date, count)| EventCountResponse {
            count,
            date: crate::dto::wrappers::NaiveDateTimeWrapper(date),
        })
        .collect();

    Ok(EventCountsResponse {
        counts: event_counts,
        hours,
    })
}
