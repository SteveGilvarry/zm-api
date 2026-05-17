use tracing::instrument;

use crate::{
    dto::response::event_summaries::EventSummaryResponse,
    dto::{PaginatedResponse, PaginationParams},
    error::{AppError, AppResult, Resource, ResourceType},
    repo::event_summaries as event_summaries_repo,
    server::state::AppState,
    service::monitor_acl::MonitorScope,
    util::authz::Level,
};

fn summary_not_found(monitor_id: u32) -> AppError {
    AppError::NotFoundError(Resource {
        details: vec![("monitor_id".to_string(), monitor_id.to_string())],
        resource_type: ResourceType::EventSummary,
    })
}

/// List all event summaries
#[instrument(skip(state, scope))]
pub async fn list_all(
    state: &AppState,
    scope: &MonitorScope,
) -> AppResult<Vec<EventSummaryResponse>> {
    let filter = scope.visible_ids(Level::View);
    let summaries = event_summaries_repo::find_all(state, filter.as_deref()).await?;

    Ok(summaries
        .into_iter()
        .map(EventSummaryResponse::from)
        .collect())
}

/// List event summaries with pagination
#[instrument(skip(state, scope))]
pub async fn list_paginated(
    state: &AppState,
    params: &PaginationParams,
    scope: &MonitorScope,
) -> AppResult<PaginatedResponse<EventSummaryResponse>> {
    let filter = scope.visible_ids(Level::View);
    let (items, total) =
        event_summaries_repo::find_paginated(state, params, filter.as_deref()).await?;
    let responses: Vec<EventSummaryResponse> =
        items.into_iter().map(EventSummaryResponse::from).collect();
    Ok(PaginatedResponse::from_params(responses, total, params))
}

/// Get event summary for a specific monitor
#[instrument(skip(state, scope))]
pub async fn get_by_monitor_id(
    state: &AppState,
    monitor_id: u32,
    scope: &MonitorScope,
) -> AppResult<EventSummaryResponse> {
    if !scope.allows(monitor_id, Level::View) {
        return Err(summary_not_found(monitor_id));
    }
    let summary = event_summaries_repo::find_by_monitor_id(state, monitor_id)
        .await?
        .ok_or_else(|| summary_not_found(monitor_id))?;

    Ok(EventSummaryResponse::from(summary))
}
