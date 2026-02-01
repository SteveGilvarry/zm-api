use sea_orm::*;
use tracing::instrument;

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

/// Find event summary by monitor ID
#[instrument(skip(state))]
pub async fn find_by_monitor_id(
    state: &AppState,
    monitor_id: u32,
) -> Result<Option<event_summaries::Model>, DbErr> {
    EventSummaries::find_by_id(monitor_id).one(state.db()).await
}
