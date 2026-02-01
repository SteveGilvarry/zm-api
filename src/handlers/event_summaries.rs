use axum::{
    extract::{Path, State},
    Json,
};
use tracing::{info, instrument};

use crate::{
    dto::response::event_summaries::EventSummaryResponse,
    error::{AppResponseError, AppResult},
    server::state::AppState,
    service,
};

/// Get event summaries for all monitors
///
/// Returns pre-calculated event counts and disk space usage for all monitors.
/// These summaries include counts for: total, hour, day, week, month, and archived events.
#[utoipa::path(
    get,
    path = "/api/v3/event-summaries",
    operation_id = "listEventSummaries",
    tag = "Events",
    responses(
        (status = 200, description = "List of event summaries for all monitors", body = Vec<EventSummaryResponse>),
        (status = 401, description = "Unauthorized", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(
        ("jwt" = [])
    )
)]
#[instrument(skip(state))]
pub async fn list_event_summaries(
    State(state): State<AppState>,
) -> AppResult<Json<Vec<EventSummaryResponse>>> {
    info!("Listing event summaries for all monitors");

    let summaries = service::event_summaries::list_all(&state).await?;

    Ok(Json(summaries))
}

/// Get event summary for a specific monitor
///
/// Returns pre-calculated event counts and disk space usage for a specific monitor.
#[utoipa::path(
    get,
    path = "/api/v3/event-summaries/{monitor_id}",
    operation_id = "getEventSummary",
    tag = "Events",
    params(
        ("monitor_id" = u32, Path, description = "Monitor ID")
    ),
    responses(
        (status = 200, description = "Event summary for the monitor", body = EventSummaryResponse),
        (status = 401, description = "Unauthorized", body = AppResponseError),
        (status = 404, description = "Monitor not found", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(
        ("jwt" = [])
    )
)]
#[instrument(skip(state))]
pub async fn get_event_summary(
    State(state): State<AppState>,
    Path(monitor_id): Path<u32>,
) -> AppResult<Json<EventSummaryResponse>> {
    info!("Getting event summary for monitor {}", monitor_id);

    let summary = service::event_summaries::get_by_monitor_id(&state, monitor_id).await?;

    Ok(Json(summary))
}
