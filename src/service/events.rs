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

    let total_pages = total.div_ceil(page_size);

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

    let total_pages = total.div_ceil(page_size);

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

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult};
    use crate::entity::events::Model as EventModel;
    use rust_decimal::Decimal;
    use chrono::DateTime;

    fn mk_event(id: u64, name: &str) -> EventModel {
        use crate::entity::sea_orm_active_enums::{Orientation, Scheme};
        EventModel {
            id,
            monitor_id: 1,
            storage_id: 1,
            secondary_storage_id: None,
            name: name.into(),
            cause: None,
            start_date_time: DateTime::from_timestamp(1_700_000_000, 0).map(|dt| dt.naive_utc()),
            end_date_time: None,
            width: 1920,
            height: 1080,
            length: Decimal::new(0, 0),
            frames: Some(0),
            alarm_frames: Some(0),
            default_video: "v.mp4".into(),
            save_jpe_gs: None,
            tot_score: 0,
            avg_score: None,
            max_score: None,
            archived: 0,
            videoed: 0,
            uploaded: 0,
            emailed: 0,
            messaged: 0,
            executed: 0,
            notes: None,
            state_id: 1,
            orientation: Orientation::Rotate0,
            disk_space: None,
            scheme: Scheme::Deep,
            locked: 0,
            latitude: None,
            longitude: None,
        }
    }

    #[tokio::test]
    async fn test_get_by_id_ok_and_not_found() {
        let db_ok = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<EventModel, _, _>(vec![vec![mk_event(5, "ev")]])
            .into_connection();
        let state_ok = AppState::for_test_with_db(db_ok);
        assert_eq!(get_by_id(&state_ok, 5).await.unwrap().id, 5);

        let empty: Vec<EventModel> = vec![];
        let db_none = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<EventModel, _, _>(vec![empty])
            .into_connection();
        let state_none = AppState::for_test_with_db(db_none);
        assert!(matches!(get_by_id(&state_none, 1).await.err().unwrap(), AppError::NotFoundError(_)));
    }

    #[tokio::test]
    async fn test_create_update_delete_ok() {
        use crate::dto::request::events::{EventCreateRequest, EventUpdateRequest};
        use crate::dto::wrappers::DecimalWrapper;
        use crate::entity::sea_orm_active_enums::Orientation;
        // Create: insert exec result is enough for MockDatabase
        let db_create = MockDatabase::new(DatabaseBackend::MySql)
            .append_exec_results(vec![MockExecResult { last_insert_id: 100, rows_affected: 1 }])
            .append_query_results::<EventModel, _, _>(vec![vec![mk_event(100, "name")]])
            .into_connection();
        let state_create = AppState::for_test_with_db(db_create);
        let req = EventCreateRequest {
            monitor_id: 1,
            storage_id: 1,
            secondary_storage_id: None,
            name: "name".into(),
            cause: None,
            start_date_time: None,
            end_date_time: None,
            width: 1280,
            height: 720,
            length: DecimalWrapper(Decimal::new(0,0)),
            notes: None,
            orientation: Orientation::Rotate0,
        };
        let out = create(&state_create, req).await.unwrap();
        assert_eq!(out.name, "name");

        // Update: first find_by_id returns a model; then exec + query returns updated
        let initial = mk_event(7, "old");
        let mut after = initial.clone();
        after.name = "new".into();
        let db_update = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<EventModel, _, _>(vec![vec![initial]])
            .append_exec_results(vec![MockExecResult { last_insert_id: 0, rows_affected: 1 }])
            .append_query_results::<EventModel, _, _>(vec![vec![after.clone()]])
            .into_connection();
        let state_update = AppState::for_test_with_db(db_update);
        let out_upd = update(&state_update, 7, EventUpdateRequest { name: Some("new".into()), cause: None, notes: None, orientation: None }).await.unwrap();
        assert_eq!(out_upd.name, "new");

        // Delete
        let db_del = MockDatabase::new(DatabaseBackend::MySql)
            .append_exec_results(vec![MockExecResult { last_insert_id: 0, rows_affected: 1 }])
            .into_connection();
        let state_del = AppState::for_test_with_db(db_del);
        assert!(delete(&state_del, 7).await.is_ok());
    }
}
