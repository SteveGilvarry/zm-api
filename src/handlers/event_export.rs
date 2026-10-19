//! `POST /api/v3/events/export` (#30).

use axum::body::Body;
use axum::extract::State;
use axum::http::{header, HeaderValue};
use axum::response::{IntoResponse, Response};
use axum::Json;
use garde::Validate;

use crate::dto::request::event_export::EventExportRequest;
use crate::error::{AppError, AppResponseError, AppResult};
use crate::server::state::AppState;
use crate::service::event_export as svc;
use crate::service::monitor_acl::MonitorScope;

/// Export events as a zip
///
/// Streams a zip, built as it goes, with a `{event_id}/` folder per event
/// holding the parts asked for: `details` (`event.json`), `frames`
/// (`frames.json`), `images`, `video` and `misc` (the event directory's other
/// files). Every id is checked first: one that doesn't exist or isn't
/// visible to the caller is a 404 and nothing is sent.
#[utoipa::path(
    post,
    path = "/api/v3/events/export",
    operation_id = "exportEvents",
    request_body = EventExportRequest,
    responses(
        (status = 200, description = "The zip", content_type = "application/zip", body = Vec<u8>),
        (status = 400, description = "No ids, or more than 500", body = AppResponseError),
        (status = 404, description = "An event doesn't exist or isn't visible", body = AppResponseError)
    ),
    tag = "Events",
    security(("jwt" = []))
)]
pub async fn export_events(
    State(state): State<AppState>,
    scope: MonitorScope,
    Json(req): Json<EventExportRequest>,
) -> AppResult<Response> {
    req.validate().map_err(AppError::InvalidInputError)?;
    let exports = svc::prepare(&state, &scope, &req.ids).await?;
    let (writer, reader) = tokio::io::duplex(64 * 1024);
    tokio::spawn(svc::write(state.clone(), exports, req.parts(), writer));

    let name = format!(
        "attachment; filename=\"zm-events-{}.zip\"",
        chrono::Local::now().format("%Y%m%d-%H%M%S")
    );
    let mut res = Body::from_stream(tokio_util::io::ReaderStream::new(reader)).into_response();
    let h = res.headers_mut();
    h.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/zip"),
    );
    if let Ok(v) = HeaderValue::from_str(&name) {
        h.insert(header::CONTENT_DISPOSITION, v);
    }
    h.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    Ok(res)
}
