//! `GET /api/v3/reports/event-audit` (#36).

use axum::extract::{Query, State};
use axum::Json;
use serde::Deserialize;

use crate::dto::response::event_audit::EventAuditResponse;
use crate::dto::wrappers::DateTimeWrapper;
use crate::error::{AppResponseError, AppResult};
use crate::server::state::AppState;
use crate::service::monitor_acl::MonitorScope;

#[derive(Debug, Deserialize)]
pub struct EventAuditQuery {
    pub start: Option<DateTimeWrapper>,
    pub end: Option<DateTimeWrapper>,
    pub monitor_id: Option<u32>,
}

/// Audit events over a window: per monitor, counts, gaps, and missing or
/// empty media
///
/// - Legacy's "Audit Events Report". Events overlapping `[start, end)` are
///   counted per monitor; each one's media is checked on disk.
/// - Defaults to the hour ending an hour ago, as legacy does. At most 10000
///   events; a wider window is a 400.
#[utoipa::path(
    get,
    path = "/api/v3/reports/event-audit",
    params(
        ("start" = Option<String>, Query, description = "Window start (RFC 3339)"),
        ("end" = Option<String>, Query, description = "Window end (RFC 3339)"),
        ("monitor_id" = Option<u32>, Query, description = "Only this monitor")
    ),
    responses(
        (status = 200, description = "Per-monitor audit", body = EventAuditResponse),
        (status = 400, description = "Bad window, or too many events", body = AppResponseError)
    ),
    tag = "Events",
    security(("jwt" = []))
)]
pub async fn event_audit(
    State(state): State<AppState>,
    Query(q): Query<EventAuditQuery>,
    scope: MonitorScope,
) -> AppResult<Json<EventAuditResponse>> {
    let (default_start, default_end) = crate::service::event_audit::default_window();
    let start = q.start.map(|d| d.0.naive_utc()).unwrap_or(default_start);
    let end = q.end.map(|d| d.0.naive_utc()).unwrap_or(default_end);
    Ok(Json(
        crate::service::event_audit::event_audit(&state, start, end, q.monitor_id, &scope).await?,
    ))
}
