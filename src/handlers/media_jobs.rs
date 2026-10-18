//! Generated event videos and the media job queue (#187).

use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use garde::Validate;

use crate::dto::request::media_jobs::{GenerateVideoRequest, JobListQuery};
use crate::dto::response::media_jobs::{EventVideoResponse, MediaJobResponse};
use crate::error::{AppError, AppResponseError, AppResult};
use crate::server::state::AppState;
use crate::service::media_jobs as svc;
use crate::service::monitor_acl::MonitorScope;
use crate::util::authz::Level;
use crate::util::claim::UserClaims;

/// Generate a video for an event
///
/// Queues a `video` job, ZoneMinder's Generate Video: from the recording
/// when the event has one, else from its capture JPEGs. Poll
/// `GET /api/v3/jobs/{id}`; when it is `done`, `output` names the file in
/// `GET /api/v3/events/{id}/videos`.
#[utoipa::path(
    post,
    path = "/api/v3/events/{id}/videos",
    operation_id = "generateEventVideo",
    params(("id" = u64, Path, description = "Event ID")),
    request_body = GenerateVideoRequest,
    responses(
        (status = 202, description = "Queued", body = MediaJobResponse),
        (status = 400, description = "Bad options, or a format not in ZM_FFMPEG_FORMATS", body = AppResponseError),
        (status = 404, description = "Event not found", body = AppResponseError)
    ),
    tag = "Events",
    security(("jwt" = []))
)]
pub async fn generate_video(
    State(state): State<AppState>,
    Extension(claims): Extension<UserClaims>,
    scope: MonitorScope,
    Path(id): Path<u64>,
    Json(req): Json<GenerateVideoRequest>,
) -> AppResult<(StatusCode, Json<MediaJobResponse>)> {
    req.validate().map_err(AppError::InvalidInputError)?;
    Ok((
        StatusCode::ACCEPTED,
        Json(svc::enqueue_video(&state, &scope, claims.uid, id, req).await?),
    ))
}

/// List an event's video files
///
/// Generated files and the recording, with the options read back from a
/// generated file's name.
#[utoipa::path(
    get,
    path = "/api/v3/events/{id}/videos",
    operation_id = "listEventVideos",
    params(("id" = u64, Path, description = "Event ID")),
    responses(
        (status = 200, description = "Video files", body = Vec<EventVideoResponse>),
        (status = 404, description = "Event not found", body = AppResponseError)
    ),
    tag = "Events",
    security(("jwt" = []))
)]
pub async fn list_videos(
    State(state): State<AppState>,
    scope: MonitorScope,
    Path(id): Path<u64>,
) -> AppResult<Json<Vec<EventVideoResponse>>> {
    Ok(Json(svc::list_videos(&state, &scope, id).await?))
}

/// Download one of an event's video files
#[utoipa::path(
    get,
    path = "/api/v3/events/{id}/videos/{name}",
    operation_id = "downloadEventVideo",
    params(
        ("id" = u64, Path, description = "Event ID"),
        ("name" = String, Path, description = "File name, from the listing")
    ),
    responses(
        (status = 200, description = "The file", content_type = "application/octet-stream", body = Vec<u8>),
        (status = 404, description = "Not found", body = AppResponseError)
    ),
    tag = "Events",
    security(("jwt" = []))
)]
pub async fn download_video(
    State(state): State<AppState>,
    scope: MonitorScope,
    Path((id, name)): Path<(u64, String)>,
) -> AppResult<Response> {
    let (_, path) = svc::video_path(&state, &scope, id, &name, Level::View).await?;
    let file = tokio::fs::File::open(&path).await?;
    let len = file.metadata().await?.len();
    let mut res = Body::from_stream(tokio_util::io::ReaderStream::new(file)).into_response();
    let h = res.headers_mut();
    let mime = mime_for(&name);
    h.insert(header::CONTENT_TYPE, HeaderValue::from_static(mime));
    h.insert(header::CONTENT_LENGTH, HeaderValue::from(len));
    if let Ok(v) = HeaderValue::from_str(&format!("attachment; filename=\"{name}\"")) {
        h.insert(header::CONTENT_DISPOSITION, v);
    }
    Ok(res)
}

fn mime_for(name: &str) -> &'static str {
    match name
        .rsplit('.')
        .next()
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("mp4") => "video/mp4",
        Some("mkv") => "video/x-matroska",
        Some("webm") => "video/webm",
        Some("avi") => "video/x-msvideo",
        Some("mov") => "video/quicktime",
        Some("mpg" | "mpeg") => "video/mpeg",
        Some("3gp") => "video/3gpp",
        _ => "application/octet-stream",
    }
}

/// Delete a generated video
///
/// The event's recording can't be deleted here (409).
#[utoipa::path(
    delete,
    path = "/api/v3/events/{id}/videos/{name}",
    operation_id = "deleteEventVideo",
    params(
        ("id" = u64, Path, description = "Event ID"),
        ("name" = String, Path, description = "File name, from the listing")
    ),
    responses(
        (status = 204, description = "Deleted"),
        (status = 404, description = "Not found", body = AppResponseError),
        (status = 409, description = "The event's recording", body = AppResponseError)
    ),
    tag = "Events",
    security(("jwt" = []))
)]
pub async fn delete_video(
    State(state): State<AppState>,
    scope: MonitorScope,
    Path((id, name)): Path<(u64, String)>,
) -> AppResult<StatusCode> {
    svc::delete_video(&state, &scope, id, &name).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// List media jobs
///
/// Newest first, at most 500, for the events the caller can see.
#[utoipa::path(
    get,
    path = "/api/v3/jobs",
    operation_id = "listMediaJobs",
    params(JobListQuery),
    responses((status = 200, description = "Jobs", body = Vec<MediaJobResponse>)),
    tag = "Events",
    security(("jwt" = []))
)]
pub async fn list_jobs(
    State(state): State<AppState>,
    scope: MonitorScope,
    Query(q): Query<JobListQuery>,
) -> AppResult<Json<Vec<MediaJobResponse>>> {
    Ok(Json(svc::list_jobs(&state, &scope, q).await?))
}

/// Get a media job
#[utoipa::path(
    get,
    path = "/api/v3/jobs/{id}",
    operation_id = "getMediaJob",
    params(("id" = u32, Path, description = "Job ID")),
    responses(
        (status = 200, description = "Job", body = MediaJobResponse),
        (status = 404, description = "Not found", body = AppResponseError)
    ),
    tag = "Events",
    security(("jwt" = []))
)]
pub async fn get_job(
    State(state): State<AppState>,
    scope: MonitorScope,
    Path(id): Path<u32>,
) -> AppResult<Json<MediaJobResponse>> {
    Ok(Json(svc::get_job(&state, &scope, id).await?))
}

/// Cancel or remove a media job
///
/// Cancels a queued job; removes a finished one's record (its file stays).
/// A running job is a 409.
#[utoipa::path(
    delete,
    path = "/api/v3/jobs/{id}",
    operation_id = "deleteMediaJob",
    params(("id" = u32, Path, description = "Job ID")),
    responses(
        (status = 204, description = "Cancelled or removed"),
        (status = 404, description = "Not found", body = AppResponseError),
        (status = 409, description = "Running", body = AppResponseError)
    ),
    tag = "Events",
    security(("jwt" = []))
)]
pub async fn delete_job(
    State(state): State<AppState>,
    scope: MonitorScope,
    Path(id): Path<u32>,
) -> AppResult<StatusCode> {
    svc::delete_job(&state, &scope, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
