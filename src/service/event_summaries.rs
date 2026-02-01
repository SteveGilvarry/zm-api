use tracing::instrument;

use crate::{
    dto::response::event_summaries::EventSummaryResponse,
    dto::{PaginatedResponse, PaginationParams},
    error::{AppError, AppResult, Resource, ResourceType},
    repo::event_summaries as event_summaries_repo,
    server::state::AppState,
};

/// List all event summaries
#[instrument(skip(state))]
pub async fn list_all(state: &AppState) -> AppResult<Vec<EventSummaryResponse>> {
    let summaries = event_summaries_repo::find_all(state).await?;

    Ok(summaries
        .into_iter()
        .map(EventSummaryResponse::from)
        .collect())
}

/// List event summaries with pagination
#[instrument(skip(state))]
pub async fn list_paginated(
    state: &AppState,
    params: &PaginationParams,
) -> AppResult<PaginatedResponse<EventSummaryResponse>> {
    let (items, total) = event_summaries_repo::find_paginated(state, params).await?;
    let responses: Vec<EventSummaryResponse> =
        items.into_iter().map(EventSummaryResponse::from).collect();
    Ok(PaginatedResponse::from_params(responses, total, params))
}

/// Get event summary for a specific monitor
#[instrument(skip(state))]
pub async fn get_by_monitor_id(
    state: &AppState,
    monitor_id: u32,
) -> AppResult<EventSummaryResponse> {
    let summary = event_summaries_repo::find_by_monitor_id(state, monitor_id)
        .await?
        .ok_or_else(|| {
            AppError::NotFoundError(Resource {
                details: vec![("monitor_id".to_string(), monitor_id.to_string())],
                resource_type: ResourceType::EventSummary,
            })
        })?;

    Ok(EventSummaryResponse::from(summary))
}
