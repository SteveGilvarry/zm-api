use std::collections::HashMap;

use sea_orm::sea_query::{Alias, Order, SqliteQueryBuilder};
use sea_orm::*;
use tracing::instrument;

use crate::entity::{events, events_tags, prelude::Events, tags};
use crate::server::state::AppState;

/// Find all events with pagination
#[instrument(skip(state))]
pub async fn find_all(
    state: &AppState,
    page: u64,
    page_size: u64,
) -> Result<(Vec<events::Model>, u64), DbErr> {
    let paginator = Events::find()
        .order_by_desc(events::Column::StartDateTime)
        .paginate(state.db(), page_size);

    let total = paginator.num_items().await?;
    let events = paginator.fetch_page(page).await?;

    Ok((events, total))
}

/// Find events by monitor ID with optional date range filter
#[instrument(skip(state))]
pub async fn find_by_monitor_id(
    state: &AppState,
    monitor_id: u32,
    start_time: Option<sea_orm::prelude::DateTime>,
    end_time: Option<sea_orm::prelude::DateTime>,
    page: u64,
    page_size: u64,
) -> Result<(Vec<events::Model>, u64), DbErr> {
    let mut query = Events::find()
        .filter(events::Column::MonitorId.eq(monitor_id))
        .order_by_desc(events::Column::StartDateTime);

    if let Some(start) = start_time {
        query = query.filter(events::Column::StartDateTime.gte(start));
    }

    if let Some(end) = end_time {
        query = query.filter(events::Column::EndDateTime.lte(end));
    }

    let paginator = query.paginate(state.db(), page_size);
    let total = paginator.num_items().await?;
    let events = paginator.fetch_page(page).await?;

    Ok((events, total))
}

/// Find event by ID
#[instrument(skip(state))]
pub async fn find_by_id(state: &AppState, id: u64) -> Result<Option<events::Model>, DbErr> {
    Events::find_by_id(id).one(state.db()).await
}

/// Create new event
#[instrument(skip(state, event))]
pub async fn create(state: &AppState, event: events::ActiveModel) -> Result<events::Model, DbErr> {
    event.insert(state.db()).await
}

/// Update existing event
#[instrument(skip(state, event))]
pub async fn update(state: &AppState, event: events::ActiveModel) -> Result<events::Model, DbErr> {
    event.update(state.db()).await
}

/// Delete event by ID
#[instrument(skip(state))]
pub async fn delete(state: &AppState, id: u64) -> Result<DeleteResult, DbErr> {
    Events::delete_by_id(id).exec(state.db()).await
}

/// Get event counts grouped by monitor over time period
#[instrument(skip(state))]
pub async fn get_counts_by_monitor(
    state: &AppState,
    hours_back: i64,
) -> Result<Vec<(u32, u64)>, DbErr> {
    use sea_orm::sea_query::{Expr, Query};
    use sea_orm::{FromQueryResult, Statement};

    #[derive(FromQueryResult)]
    struct CountResult {
        monitor_id: u32,
        count: u64,
    }

    // Calculate time boundary
    let now = chrono::Utc::now();
    let time_boundary = now - chrono::Duration::hours(hours_back);
    let time_boundary_str = time_boundary.format("%Y-%m-%d %H:%M:%S").to_string();

    // Raw SQL for better performance on complex aggregation
    let count_alias = Alias::new("count");
    let sql = Query::select()
        .column(events::Column::MonitorId)
        .expr_as(Expr::cust("COUNT(*)"), count_alias)
        .from(events::Entity)
        .and_where(Expr::col(events::Column::StartDateTime).gte(time_boundary_str))
        .group_by_col(events::Column::MonitorId)
        .to_string(SqliteQueryBuilder);

    let stmt = Statement::from_sql_and_values(state.db().get_database_backend(), sql, vec![]);

    let results = CountResult::find_by_statement(stmt).all(state.db()).await?;

    let counts = results
        .into_iter()
        .map(|r| (r.monitor_id, r.count))
        .collect();

    Ok(counts)
}

/// Get event counts by hour for the last n hours
#[instrument(skip(state))]
pub async fn get_counts_by_hour(
    state: &AppState,
    hours_back: i64,
) -> Result<Vec<(chrono::NaiveDateTime, u64)>, DbErr> {
    use sea_orm::sea_query::{Expr, Query};
    use sea_orm::{FromQueryResult, Statement};

    #[derive(FromQueryResult)]
    struct CountResult {
        hour: String,
        count: u64,
    }

    // Calculate time boundary
    let now = chrono::Utc::now();
    let time_boundary = now - chrono::Duration::hours(hours_back);
    let time_boundary_str = time_boundary.format("%Y-%m-%d %H:%M:%S").to_string();

    // Raw SQL for better performance on complex aggregation with grouping by hour
    let hour_alias = Alias::new("hour");
    let count_alias = Alias::new("count");
    let sql = Query::select()
        .expr_as(
            Expr::cust("strftime('%Y-%m-%d %H:00:00', start_date_time)"),
            hour_alias.clone(),
        )
        .expr_as(Expr::cust("COUNT(*)"), count_alias.clone())
        .from(events::Entity)
        .and_where(Expr::col(events::Column::StartDateTime).gte(time_boundary_str))
        .group_by_col(hour_alias.clone())
        .order_by(hour_alias.clone(), Order::Asc)
        .to_string(SqliteQueryBuilder);

    let stmt = Statement::from_sql_and_values(state.db().get_database_backend(), sql, vec![]);

    let results = CountResult::find_by_statement(stmt).all(state.db()).await?;

    let counts = results
        .into_iter()
        .map(|r| {
            let naive_date = chrono::NaiveDateTime::parse_from_str(&r.hour, "%Y-%m-%d %H:%M:%S")
                .unwrap_or_else(|_| {
                    chrono::DateTime::<chrono::Utc>::from_timestamp(0, 0)
                        .unwrap()
                        .naive_utc()
                });
            (naive_date, r.count)
        })
        .collect();

    Ok(counts)
}

/// Find tags for a list of event IDs (batch loading to prevent N+1)
#[instrument(skip(state))]
pub async fn find_tags_for_events(
    state: &AppState,
    event_ids: &[u64],
) -> Result<HashMap<u64, Vec<tags::Model>>, DbErr> {
    if event_ids.is_empty() {
        return Ok(HashMap::new());
    }

    // Fetch all event-tag associations for the given events
    let associations = events_tags::Entity::find()
        .filter(events_tags::Column::EventId.is_in(event_ids.to_vec()))
        .all(state.db())
        .await?;

    if associations.is_empty() {
        return Ok(HashMap::new());
    }

    // Collect unique tag IDs
    let tag_ids: Vec<u64> = associations.iter().map(|a| a.tag_id).collect();

    // Fetch all tags
    let all_tags = tags::Entity::find()
        .filter(tags::Column::Id.is_in(tag_ids))
        .all(state.db())
        .await?;

    // Create a map of tag_id -> tag model
    let tag_map: HashMap<u64, tags::Model> = all_tags.into_iter().map(|t| (t.id, t)).collect();

    // Group tags by event_id
    let mut result: HashMap<u64, Vec<tags::Model>> = HashMap::new();
    for assoc in associations {
        if let Some(tag) = tag_map.get(&assoc.tag_id) {
            result.entry(assoc.event_id).or_default().push(tag.clone());
        }
    }

    Ok(result)
}

/// Find a single event with its tags
#[instrument(skip(state))]
pub async fn find_by_id_with_tags(
    state: &AppState,
    id: u64,
) -> Result<Option<(events::Model, Vec<tags::Model>)>, DbErr> {
    let event = Events::find_by_id(id).one(state.db()).await?;

    match event {
        Some(e) => {
            let tags_map = find_tags_for_events(state, &[id]).await?;
            let tags = tags_map.get(&id).cloned().unwrap_or_default();
            Ok(Some((e, tags)))
        }
        None => Ok(None),
    }
}
