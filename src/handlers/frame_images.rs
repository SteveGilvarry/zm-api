//! Event frame images (GH #26).
//!
//! Serves the JPEG for one frame of an event, the way ZoneMinder's
//! `view=image&eid=…&fid=…` does. `fid` is a frame number, or one of `alarm`,
//! `objdetect` and `snapshot`. A JPEG ZoneMinder saved to disk is served as is;
//! when there isn't one (monitors that record video only), the frame is
//! decoded from the event's MP4 at that frame's time.

use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode},
    response::Response,
};
use rust_decimal::prelude::ToPrimitive;
use serde::Deserialize;
use std::path::Path as StdPath;
use tracing::{debug, warn};

use crate::entity::events::Model as EventModel;
use crate::error::{AppError, AppResponseError, AppResult, Resource, ResourceType};
use crate::handlers::events_playback::{
    event_is_in_progress, find_incomplete_media, get_event_entity, monitor_orientation,
    select_video_filename,
};
use crate::repo;
use crate::server::state::AppState;
use crate::service::event_storage::{build_event_directory_path, resolve_event_storage_path};
use crate::service::monitor_acl::MonitorScope;

#[derive(Debug, Deserialize)]
pub struct EventFrameImagePath {
    pub id: u64,
    pub fid: String,
}

#[derive(Debug, Deserialize)]
pub struct FrameImagePath {
    pub id: u64,
}

/// Which frame of an event to serve.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FrameSelector {
    Number(u32),
    Alarm,
    ObjDetect,
    Snapshot,
}

impl FrameSelector {
    fn parse(fid: &str) -> Option<Self> {
        match fid {
            "alarm" => Some(Self::Alarm),
            "objdetect" => Some(Self::ObjDetect),
            "snapshot" => Some(Self::Snapshot),
            n => n.parse::<u32>().ok().filter(|n| *n > 0).map(Self::Number),
        }
    }
}

/// JPEG filenames to try, in order, for a frame number. ZoneMinder names
/// them with `ZM_EVENT_IMAGE_DIGITS` (5) digits; some older layouts prefix
/// the event id.
fn numbered_candidates(event_id: u64, n: u32) -> [String; 4] {
    [
        format!("{n:05}-capture.jpg"),
        format!("{event_id}-{n:05}-capture.jpg"),
        format!("{n:05}-analyse.jpg"),
        format!("{event_id}-{n:05}-analyse.jpg"),
    ]
}

/// The named still ZoneMinder writes for a selector, and the frame number to
/// fall back to when that file is missing.
fn named_still(
    event_id: u64,
    sel: FrameSelector,
    event: &EventModel,
) -> (Vec<String>, Option<u32>) {
    let best = event.max_score_frame_id.filter(|n| *n > 0);
    match sel {
        FrameSelector::Number(n) => (Vec::new(), Some(n)),
        FrameSelector::Alarm => (
            vec!["alarm.jpg".into(), format!("{event_id}-alarm.jpg")],
            best,
        ),
        // Only exists when object detection ran; no fallback.
        FrameSelector::ObjDetect => (
            vec!["objdetect.jpg".into(), format!("{event_id}-objdetect.jpg")],
            None,
        ),
        FrameSelector::Snapshot => (
            vec!["snapshot.jpg".into(), format!("{event_id}-snapshot.jpg")],
            best.or(Some(1)),
        ),
    }
}

/// Seconds into the event's video where frame `n` sits: the delta of the
/// nearest `Frames` row at or before it, plus the gap to `n` at the event's
/// average frame rate.
fn frame_seconds(n: u32, nearest: Option<(u32, f64)>, frames: Option<u32>, length: f64) -> f64 {
    let spf = match frames {
        Some(f) if f > 0 && length > 0.0 => length / f64::from(f),
        _ => 0.0,
    };
    match nearest {
        Some((fid, delta)) => delta + f64::from(n.saturating_sub(fid)) * spf,
        None => f64::from(n.saturating_sub(1)) * spf,
    }
}

fn jpeg_response(data: Vec<u8>) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "image/jpeg")
        .header(header::CACHE_CONTROL, "max-age=86400")
        .body(Body::from(data))
        .unwrap()
}

fn frame_not_found(event_id: u64, fid: &str, reason: &str) -> AppError {
    AppError::NotFoundError(Resource {
        resource_type: ResourceType::Event,
        details: vec![
            ("event_id".to_string(), event_id.to_string()),
            ("fid".to_string(), fid.to_string()),
            ("reason".to_string(), reason.to_string()),
        ],
    })
}

async fn read_first(dir: &StdPath, names: &[String]) -> Option<Vec<u8>> {
    for name in names {
        if let Ok(data) = tokio::fs::read(dir.join(name)).await {
            debug!("serving frame image {:?}", dir.join(name));
            return Some(data);
        }
    }
    None
}

async fn serve_frame(state: &AppState, event: EventModel, fid: &str) -> AppResult<Response> {
    let event_id = event.id;
    let sel = FrameSelector::parse(fid).ok_or_else(|| {
        AppError::BadRequestError(format!(
            "fid must be a frame number from 1, `alarm`, `objdetect` or `snapshot`, not `{fid}`"
        ))
    })?;
    if let (FrameSelector::Number(n), Some(total)) = (sel, event.frames) {
        if total > 0 && n > total {
            return Err(frame_not_found(
                event_id,
                fid,
                "past the event's last frame",
            ));
        }
    }

    let storage_path = resolve_event_storage_path(state, &event).await?;
    let dir = build_event_directory_path(
        &storage_path,
        event.monitor_id,
        event_id,
        event.start_date_time,
        &event.scheme,
    );
    let orientation = monitor_orientation(state, event.monitor_id).await;

    let (named, fallback) = named_still(event_id, sel, &event);
    let mut names = named;
    if let Some(n) = fallback {
        names.extend(numbered_candidates(event_id, n));
    }
    if let Some(data) = read_first(&dir, &names).await {
        return Ok(jpeg_response(
            crate::service::image_orientation::orient_jpeg(data, orientation).await,
        ));
    }

    let Some(n) = fallback else {
        return Err(frame_not_found(
            event_id,
            fid,
            "no image saved for this event",
        ));
    };
    let nearest = repo::frames::find_at_or_before(state.db(), event_id, n)
        .await?
        .map(|f| (f.frame_id, f.delta.to_f64().unwrap_or(0.0)));
    let seconds = frame_seconds(
        n,
        nearest,
        event.frames,
        event.length.to_f64().unwrap_or(0.0),
    );

    // While recording, the only video is the growing `incomplete.*.mp4`.
    let mut videos = Vec::with_capacity(4);
    if event_is_in_progress(&event) {
        videos.extend(find_incomplete_media(&dir).await);
    }
    videos.extend(
        [
            select_video_filename(event_id, &event.default_video),
            format!("{event_id}-video.mp4"),
            format!("{event_id}-video.h264.mp4"),
        ]
        .map(|name| dir.join(name)),
    );
    for path in videos {
        if tokio::fs::metadata(&path).await.is_err() {
            continue;
        }
        match crate::streaming::snapshot::extract_mp4_frame_at(path.clone(), seconds, u32::MAX)
            .await
        {
            Ok(jpeg) => {
                return Ok(jpeg_response(
                    crate::service::image_orientation::orient_jpeg(jpeg, orientation).await,
                ))
            }
            Err(e) => warn!("frame {n} of event {event_id} from {path:?}: {e}"),
        }
    }
    Err(frame_not_found(
        event_id,
        fid,
        "no saved image or readable video",
    ))
}

/// Get one frame of an event as a JPEG
#[utoipa::path(
    get,
    path = "/api/v3/events/{id}/frames/{fid}/image",
    operation_id = "getEventFrameImage",
    tag = "Event Playback",
    params(
        ("id" = u64, Path, description = "Event ID"),
        ("fid" = String, Path, description = "Frame number (from 1), or `alarm`, `objdetect` or `snapshot`"),
    ),
    responses(
        (status = 200, description = "Frame image", content_type = "image/jpeg"),
        (status = 400, description = "Invalid fid", body = AppResponseError),
        (status = 404, description = "Event or frame image not found", body = AppResponseError)
    ),
    security(("jwt" = []))
)]
pub async fn get_event_frame_image(
    State(state): State<AppState>,
    Path(path): Path<EventFrameImagePath>,
    scope: MonitorScope,
) -> AppResult<Response> {
    let event = get_event_entity(&state, path.id, &scope).await?;
    serve_frame(&state, event, &path.fid).await
}

/// Get a frame's image by its `Frames` row id
#[utoipa::path(
    get,
    path = "/api/v3/frames/{id}/image",
    operation_id = "getFrameImage",
    tag = "Frames",
    params(("id" = u64, Path, description = "Frame row ID")),
    responses(
        (status = 200, description = "Frame image", content_type = "image/jpeg"),
        (status = 404, description = "Frame or frame image not found", body = AppResponseError)
    ),
    security(("jwt" = []))
)]
pub async fn get_frame_image(
    State(state): State<AppState>,
    Path(path): Path<FrameImagePath>,
    scope: MonitorScope,
) -> AppResult<Response> {
    let not_found = || {
        AppError::NotFoundError(Resource {
            resource_type: ResourceType::Event,
            details: vec![("frame_id".to_string(), path.id.to_string())],
        })
    };
    let frame = repo::frames::find_by_id(state.db(), path.id)
        .await?
        .ok_or_else(not_found)?;
    // A frame in an event the caller can't see is reported as missing.
    let event = get_event_entity(&state, frame.event_id, &scope)
        .await
        .map_err(|_| not_found())?;
    serve_frame(&state, event, &frame.frame_id.to_string()).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fid_accepts_numbers_from_one_and_named_stills() {
        assert_eq!(FrameSelector::parse("42"), Some(FrameSelector::Number(42)));
        assert_eq!(FrameSelector::parse("alarm"), Some(FrameSelector::Alarm));
        assert_eq!(
            FrameSelector::parse("objdetect"),
            Some(FrameSelector::ObjDetect)
        );
        assert_eq!(
            FrameSelector::parse("snapshot"),
            Some(FrameSelector::Snapshot)
        );
        for bad in ["0", "-1", "", "Alarm", "1.5", "../x"] {
            assert_eq!(FrameSelector::parse(bad), None, "{bad}");
        }
    }

    #[test]
    fn numbered_candidates_use_five_digits() {
        assert_eq!(numbered_candidates(7, 42)[0], "00042-capture.jpg");
        assert_eq!(numbered_candidates(7, 42)[1], "7-00042-capture.jpg");
    }

    #[test]
    fn frame_seconds_steps_from_the_nearest_row() {
        // 100 frames over 10s: 0.1s a frame.
        assert!((frame_seconds(1, None, Some(100), 10.0) - 0.0).abs() < 1e-9);
        assert!((frame_seconds(51, None, Some(100), 10.0) - 5.0).abs() < 1e-9);
        assert!((frame_seconds(25, Some((20, 3.0)), Some(100), 10.0) - 3.5).abs() < 1e-9);
        // Exact row, and unknown rate: just the row's delta.
        assert!((frame_seconds(20, Some((20, 3.0)), None, 0.0) - 3.0).abs() < 1e-9);
    }
}
