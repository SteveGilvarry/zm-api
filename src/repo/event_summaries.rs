use sea_orm::*;
use tracing::instrument;

use crate::dto::PaginationParams;
use crate::entity::{event_summaries, prelude::EventSummaries};
use crate::server::state::AppState;

/// Find all event summaries
#[instrument(skip(state))]
pub async fn find_all(state: &AppState) -> Result<Vec<event_summaries::Model>, DbErr> {
    EventSummaries::find()
        .order_by_asc(event_summaries::Column::MonitorId)
        .all(state.db())
        .await
}

/// Find event summaries with pagination
#[instrument(skip(state))]
pub async fn find_paginated(
    state: &AppState,
    params: &PaginationParams,
) -> Result<(Vec<event_summaries::Model>, u64), DbErr> {
    let paginator = EventSummaries::find()
        .order_by_asc(event_summaries::Column::MonitorId)
        .paginate(state.db(), params.page_size());
    let total = paginator.num_items().await?;
    let items = paginator
        .fetch_page(params.page().saturating_sub(1))
        .await?;
    Ok((items, total))
}

/// Find event summary by monitor ID
#[instrument(skip(state))]
pub async fn find_by_monitor_id(
    state: &AppState,
    monitor_id: u32,
) -> Result<Option<event_summaries::Model>, DbErr> {
    EventSummaries::find_by_id(monitor_id).one(state.db()).await
}
