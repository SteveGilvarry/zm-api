//! Event playback HTTP handlers
//!
//! Provides endpoints for playing back recorded events via HLS or direct video access.
//! Supports byte-range requests for efficient seeking in fragmented MP4 files.

use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, HeaderMap, StatusCode},
    response::Response,
};
use chrono::{Datelike, Timelike};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tracing::{debug, warn};

use crate::entity::events::Model as EventModel;
use crate::entity::sea_orm_active_enums::Scheme;
use crate::error::{AppError, AppResponseError, AppResult, Resource, ResourceType};
use crate::repo;
use crate::server::state::AppState;

// ============================================================================
// DTOs
// ============================================================================

/// Path parameters for event playback
#[derive(Debug, Deserialize)]
pub struct EventPlaybackPath {
    pub id: u64,
}

/// Response for event video metadata
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct EventVideoInfo {
    pub event_id: u64,
    pub duration_seconds: f64,
    pub video_codec: String,
    pub width: u32,
    pub height: u32,
    pub file_size: u64,
    pub is_fragmented: bool,
}

// ============================================================================
// Helpers
// ============================================================================

/// True iff `name` is safe to join onto the event directory: a single,
/// non-traversing filename. `Events.default_video` is DB-controlled, and a
/// malformed value like `"../../../etc/passwd"` would otherwise escape the
/// event directory via `PathBuf::join`.
fn is_safe_event_filename(name: &str) -> bool {
    !name.is_empty()
        && name != "."
        && name != ".."
        && !name.contains('/')
        && !name.contains('\\')
        && !name.contains('\0')
}

/// Pick the filename to look for under the event directory. Falls back to the
/// canonical `{event_id}-video.mp4` name if `default_video` is empty or fails
/// the safety check above.
fn select_video_filename(event_id: u64, default_video: &str) -> String {
    if default_video.is_empty() {
        return format!("{}-video.mp4", event_id);
    }
    if is_safe_event_filename(default_video) {
        return default_video.to_string();
    }
    warn!(
        "Refusing unsafe default_video for event {}: {:?}; using default name",
        event_id, default_video
    );
    format!("{}-video.mp4", event_id)
}

/// Get the video file path for an event
///
/// ZoneMinder supports multiple storage schemes:
/// - **Deep**: `{storage_path}/{MonitorId}/{YY/MM/DD/HH/MM/SS}/{video_file}`
/// - **Medium**: `{storage_path}/{MonitorId}/{YYYY-MM-DD}/{EventId}/{video_file}`
/// - **Shallow**: `{storage_path}/{MonitorId}/{EventId}/{video_file}`
async fn get_event_video_path(state: &AppState, event_id: u64) -> AppResult<PathBuf> {
    // Get event from database (using repo to get raw entity)
    let event = get_event_entity(state, event_id).await?;

    // Get the video filename from the event (e.g., "467-video.h264.mp4"),
    // rejecting traversal attempts in the DB-supplied default_video field.
    let video_filename = select_video_filename(event_id, &event.default_video);

    // Look up storage path from database
    let storage = match event.storage_id {
        Some(storage_id) => repo::storage::find_by_id(state.db(), storage_id).await?,
        None => None,
    };

    let storage_path = match storage {
        Some(s) => s.path,
        None => {
            // Fall back to config if storage not found in DB
            warn!(
                "Storage {} not found in database, using config default",
                event.storage_id.unwrap_or(0)
            );
            state.config.streaming.zoneminder.events_dir.clone()
        }
    };

    // Build event directory path based on scheme
    let event_dir = build_event_directory_path(
        &storage_path,
        event.monitor_id,
        event_id,
        event.start_date_time,
        &event.scheme,
    );

    // Try the computed path
    let video_path = event_dir.join(&video_filename);
    debug!("Looking for video at: {:?}", video_path);

    if tokio::fs::metadata(&video_path).await.is_ok() {
        return Ok(video_path);
    }

    // Try alternative video filenames if the default doesn't exist
    let alternative_names = [
        format!("{}-video.mp4", event_id),
        format!("{}-video.h264.mp4", event_id),
        format!("{}.mp4", event_id),
    ];

    for alt_name in &alternative_names {
        let alt_path = event_dir.join(alt_name);
        if tokio::fs::metadata(&alt_path).await.is_ok() {
            debug!("Found video at alternative path: {:?}", alt_path);
            return Ok(alt_path);
        }
    }

    // Log what we tried for debugging
    warn!(
        "Video not found for event {}. Tried directory: {:?}, filename: {}, alternatives: {:?}",
        event_id, event_dir, video_filename, alternative_names
    );

    Err(AppError::NotFoundError(Resource {
        resource_type: ResourceType::Event,
        details: vec![
            ("event_id".to_string(), event_id.to_string()),
            ("reason".to_string(), "Video file not found".to_string()),
            ("tried_path".to_string(), event_dir.display().to_string()),
        ],
    }))
}

/// Helper to get raw event entity from database
async fn get_event_entity(state: &AppState, event_id: u64) -> AppResult<EventModel> {
    repo::events::find_by_id(state, event_id)
        .await?
        .ok_or_else(|| {
            AppError::NotFoundError(Resource {
                resource_type: ResourceType::Event,
                details: vec![("event_id".to_string(), event_id.to_string())],
            })
        })
}

/// Build the event directory path based on ZoneMinder's storage scheme
fn build_event_directory_path(
    storage_path: &str,
    monitor_id: u32,
    event_id: u64,
    start_time: Option<chrono::NaiveDateTime>,
    scheme: &Scheme,
) -> PathBuf {
    let base = PathBuf::from(storage_path).join(monitor_id.to_string());

    match scheme {
        Scheme::Deep => {
            // Deep: {storage}/{monitor_id}/{YY/MM/DD/HH/MM/SS}/
            if let Some(dt) = start_time {
                base.join(format!("{:02}", dt.year() % 100))
                    .join(format!("{:02}", dt.month()))
                    .join(format!("{:02}", dt.day()))
                    .join(format!("{:02}", dt.hour()))
                    .join(format!("{:02}", dt.minute()))
                    .join(format!("{:02}", dt.second()))
            } else {
                // Fallback to shallow if no start time
                base.join(event_id.to_string())
            }
        }
        Scheme::Medium => {
            // Medium: {storage}/{monitor_id}/{YYYY-MM-DD}/{event_id}/
            if let Some(dt) = start_time {
                base.join(format!(
                    "{:04}-{:02}-{:02}",
                    dt.year(),
                    dt.month(),
                    dt.day()
                ))
                .join(event_id.to_string())
            } else {
                // Fallback to shallow if no start time
                base.join(event_id.to_string())
            }
        }
        Scheme::Shallow => {
            // Shallow: {storage}/{monitor_id}/{event_id}/
            base.join(event_id.to_string())
        }
    }
}

/// Parse HTTP Range header
fn parse_range_header(range_header: Option<&str>, file_size: u64) -> Option<(u64, u64)> {
    let range_str = range_header?;

    if !range_str.starts_with("bytes=") {
        return None;
    }

    // A zero-length file has no satisfiable byte range; bail before the
    // `file_size - 1` arithmetic below, which would otherwise underflow.
    if file_size == 0 {
        return None;
    }

    let range_spec = &range_str[6..];
    let parts: Vec<&str> = range_spec.split('-').collect();

    if parts.len() != 2 {
        return None;
    }

    let (start, end) = if parts[0].is_empty() {
        // Suffix range: -500 means last 500 bytes
        let suffix_len: u64 = parts[1].parse().ok()?;
        let start = file_size.saturating_sub(suffix_len);
        (start, file_size - 1)
    } else {
        let start: u64 = parts[0].parse().ok()?;
        let end = if parts[1].is_empty() {
            file_size - 1
        } else {
            parts[1].parse().ok()?
        };
        (start, end)
    };

    if start <= end && end < file_size {
        Some((start, end))
    } else {
        None
    }
}

// ============================================================================
// Handlers
// ============================================================================

/// Get HLS playlist for event playback
///
/// Generates an HLS playlist with byte-range segments from a fragmented MP4 file.
/// This enables efficient seeking without having separate segment files.
#[utoipa::path(
    get,
    path = "/api/v3/events/{id}/stream/playlist.m3u8",
    operation_id = "getEventPlaylist",
    tag = "Event Playback",
    params(
        ("id" = u64, Path, description = "Event ID")
    ),
    responses(
        (status = 200, description = "HLS playlist", content_type = "application/vnd.apple.mpegurl"),
        (status = 404, description = "Event not found", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn get_event_playlist(
    State(state): State<AppState>,
    Path(path): Path<EventPlaybackPath>,
) -> Result<Response, AppError> {
    debug!("Getting HLS playlist for event {}", path.id);

    let video_path = get_event_video_path(&state, path.id).await?;

    // Get file size
    let metadata = tokio::fs::metadata(&video_path).await.map_err(|_| {
        AppError::NotFoundError(crate::error::Resource {
            resource_type: crate::error::ResourceType::Event,
            details: vec![("event_id".to_string(), path.id.to_string())],
        })
    })?;

    let file_size = metadata.len();

    // For now, generate a simple VOD playlist that points to the video file
    // In a full implementation, we would parse the fMP4 structure to find
    // fragment boundaries for proper byte-range segments
    let playlist = generate_simple_playlist(path.id, file_size);

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/vnd.apple.mpegurl")
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::from(playlist))
        .unwrap())
}

/// Generate a simple HLS playlist for the video
fn generate_simple_playlist(_event_id: u64, _file_size: u64) -> String {
    // Simple playlist that references the video file directly
    // A full implementation would parse fMP4 to find fragment boundaries
    r#"#EXTM3U
#EXT-X-VERSION:6
#EXT-X-TARGETDURATION:10
#EXT-X-PLAYLIST-TYPE:VOD
#EXT-X-INDEPENDENT-SEGMENTS
#EXTINF:10.0,
video.mp4
#EXT-X-ENDLIST
"#
    .to_string()
}

/// Get event video file with Range support
///
/// Serves the event video file with HTTP Range support for efficient seeking.
#[utoipa::path(
    get,
    path = "/api/v3/events/{id}/video",
    operation_id = "getEventVideo",
    tag = "Event Playback",
    params(
        ("id" = u64, Path, description = "Event ID")
    ),
    responses(
        (status = 200, description = "Video file (full or partial)", content_type = "video/mp4"),
        (status = 206, description = "Partial content", content_type = "video/mp4"),
        (status = 404, description = "Event not found", body = AppResponseError),
        (status = 416, description = "Range not satisfiable", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn get_event_video(
    State(state): State<AppState>,
    Path(path): Path<EventPlaybackPath>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    debug!("Getting video for event {}", path.id);

    let video_path = get_event_video_path(&state, path.id).await?;

    // Get file metadata
    let metadata = tokio::fs::metadata(&video_path).await.map_err(|e| {
        AppError::InternalServerError(format!("Failed to read video metadata: {}", e))
    })?;

    let file_size = metadata.len();

    // Check for Range header
    let range_header = headers.get(header::RANGE).and_then(|v| v.to_str().ok());

    if let Some(range_str) = range_header {
        // Handle range request
        if let Some((start, end)) = parse_range_header(Some(range_str), file_size) {
            let length = end - start + 1;

            // Read the requested range
            let mut file = File::open(&video_path).await.map_err(|e| {
                AppError::InternalServerError(format!("Failed to open video: {}", e))
            })?;

            // Seek to start position
            use tokio::io::AsyncSeekExt;
            file.seek(std::io::SeekFrom::Start(start))
                .await
                .map_err(|e| AppError::InternalServerError(format!("Failed to seek: {}", e)))?;

            // Read the range
            let mut buffer = vec![0u8; length as usize];
            file.read_exact(&mut buffer).await.map_err(|e| {
                AppError::InternalServerError(format!("Failed to read video: {}", e))
            })?;

            return Ok(Response::builder()
                .status(StatusCode::PARTIAL_CONTENT)
                .header(header::CONTENT_TYPE, "video/mp4")
                .header(header::ACCEPT_RANGES, "bytes")
                .header(
                    header::CONTENT_RANGE,
                    format!("bytes {}-{}/{}", start, end, file_size),
                )
                .header(header::CONTENT_LENGTH, length.to_string())
                .body(Body::from(buffer))
                .unwrap());
        } else {
            // Invalid range
            return Err(AppError::BadRequestError(
                "Invalid Range header".to_string(),
            ));
        }
    }

    // No range request - return full file
    // For large files, we should stream instead of loading entirely into memory
    let mut file = File::open(&video_path)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to open video: {}", e)))?;

    let mut buffer = Vec::with_capacity(file_size as usize);
    file.read_to_end(&mut buffer)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to read video: {}", e)))?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "video/mp4")
        .header(header::ACCEPT_RANGES, "bytes")
        .header(header::CONTENT_LENGTH, file_size.to_string())
        .body(Body::from(buffer))
        .unwrap())
}

/// Get event video for HLS streaming (same as video endpoint but different path)
#[utoipa::path(
    get,
    path = "/api/v3/events/{id}/stream/video.mp4",
    operation_id = "getEventStreamVideo",
    tag = "Event Playback",
    params(
        ("id" = u64, Path, description = "Event ID")
    ),
    responses(
        (status = 200, description = "Video file (full or partial)", content_type = "video/mp4"),
        (status = 206, description = "Partial content", content_type = "video/mp4"),
        (status = 404, description = "Event not found", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn get_event_stream_video(
    state: State<AppState>,
    path: Path<EventPlaybackPath>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    // Delegate to the main video endpoint
    get_event_video(state, path, headers).await
}

/// Get event thumbnail
#[utoipa::path(
    get,
    path = "/api/v3/events/{id}/thumbnail",
    operation_id = "getEventThumbnail",
    tag = "Event Playback",
    params(
        ("id" = u64, Path, description = "Event ID")
    ),
    responses(
        (status = 200, description = "Thumbnail image", content_type = "image/jpeg"),
        (status = 404, description = "Event or thumbnail not found", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn get_event_thumbnail(
    State(state): State<AppState>,
    Path(path): Path<EventPlaybackPath>,
) -> Result<Response, AppError> {
    debug!("Getting thumbnail for event {}", path.id);

    // Get event from database (using repo to get raw entity)
    let event = get_event_entity(&state, path.id).await?;

    // Look up storage path from database
    let storage = match event.storage_id {
        Some(storage_id) => repo::storage::find_by_id(state.db(), storage_id).await?,
        None => None,
    };

    let storage_path = match storage {
        Some(s) => s.path,
        None => {
            warn!(
                "Storage {} not found in database, using config default",
                event.storage_id.unwrap_or(0)
            );
            state.config.streaming.zoneminder.events_dir.clone()
        }
    };

    // Build event directory path based on scheme
    let event_dir = build_event_directory_path(
        &storage_path,
        event.monitor_id,
        path.id,
        event.start_date_time,
        &event.scheme,
    );

    // Try various thumbnail/snapshot filenames
    let thumbnail_candidates = [
        format!("{}-snapshot.jpg", path.id),
        "snapshot.jpg".to_string(),
        format!("{:05}-capture.jpg", 1), // First capture frame
        format!("{:05}-analyse.jpg", 1), // First analyse frame
        format!("{}-00001-analyse.jpg", path.id),
        format!("{}-00001-capture.jpg", path.id),
    ];

    for filename in &thumbnail_candidates {
        let thumb_path = event_dir.join(filename);
        if let Ok(data) = tokio::fs::read(&thumb_path).await {
            debug!("Found thumbnail at: {:?}", thumb_path);
            return Ok(Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "image/jpeg")
                .header(header::CACHE_CONTROL, "max-age=86400")
                .body(Body::from(data))
                .unwrap());
        }
    }

    // No pre-rendered snapshot on disk — fall back to extracting a frame from
    // the event's MP4. This is the case for in-flight events or recordings
    // where ZoneMinder hasn't materialised a snapshot.jpg yet.
    let video_candidates = [
        select_video_filename(path.id, &event.default_video),
        format!("{}-video.mp4", path.id),
        format!("{}-video.h264.mp4", path.id),
        format!("{}.mp4", path.id),
    ];
    let mut tried_videos: Vec<PathBuf> = Vec::with_capacity(video_candidates.len());

    for name in &video_candidates {
        let video_path = event_dir.join(name);
        if tokio::fs::metadata(&video_path).await.is_err() {
            tried_videos.push(video_path);
            continue;
        }
        debug!("Extracting thumbnail from MP4: {:?}", video_path);
        match crate::streaming::snapshot::extract_mp4_thumbnail(video_path.clone(), 320).await {
            Ok(jpeg) => {
                return Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header(header::CONTENT_TYPE, "image/jpeg")
                    .header(header::CACHE_CONTROL, "max-age=86400")
                    .body(Body::from(jpeg))
                    .unwrap());
            }
            Err(e) => {
                warn!(
                    "Failed to extract thumbnail from {:?} for event {}: {}",
                    video_path, path.id, e
                );
                tried_videos.push(video_path);
                continue;
            }
        }
    }

    warn!(
        "Thumbnail not found for event {}. Tried directory: {:?}",
        path.id, event_dir
    );

    Err(AppError::NotFoundError(Resource {
        resource_type: ResourceType::Event,
        details: vec![
            ("event_id".to_string(), path.id.to_string()),
            ("reason".to_string(), "Thumbnail not found".to_string()),
            ("tried_path".to_string(), event_dir.display().to_string()),
        ],
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    // ------------------------------------------------------------------
    // parse_range_header
    // ------------------------------------------------------------------

    #[test]
    fn parse_range_header_normal_closed_range() {
        assert_eq!(
            parse_range_header(Some("bytes=0-499"), 1000),
            Some((0, 499))
        );
        assert_eq!(
            parse_range_header(Some("bytes=500-999"), 1000),
            Some((500, 999))
        );
    }

    #[test]
    fn parse_range_header_open_ended_range() {
        // `bytes=500-` means from 500 to the end of file.
        assert_eq!(
            parse_range_header(Some("bytes=500-"), 1000),
            Some((500, 999))
        );
        // From the very first byte to the end.
        assert_eq!(parse_range_header(Some("bytes=0-"), 1000), Some((0, 999)));
    }

    #[test]
    fn parse_range_header_suffix_range() {
        // `bytes=-500` means the last 500 bytes.
        assert_eq!(
            parse_range_header(Some("bytes=-500"), 1000),
            Some((500, 999))
        );
        // A suffix larger than the file clamps the start to 0.
        assert_eq!(
            parse_range_header(Some("bytes=-5000"), 1000),
            Some((0, 999))
        );
    }

    #[test]
    fn parse_range_header_single_byte() {
        assert_eq!(parse_range_header(Some("bytes=0-0"), 1000), Some((0, 0)));
        assert_eq!(
            parse_range_header(Some("bytes=999-999"), 1000),
            Some((999, 999))
        );
    }

    #[test]
    fn parse_range_header_rejects_start_after_end() {
        assert_eq!(parse_range_header(Some("bytes=500-400"), 1000), None);
    }

    #[test]
    fn parse_range_header_rejects_end_beyond_file() {
        assert_eq!(parse_range_header(Some("bytes=0-2000"), 1000), None);
        assert_eq!(parse_range_header(Some("bytes=0-1000"), 1000), None);
    }

    #[test]
    fn parse_range_header_rejects_start_beyond_file() {
        assert_eq!(parse_range_header(Some("bytes=2000-3000"), 1000), None);
    }

    #[test]
    fn parse_range_header_rejects_missing_or_garbage() {
        assert_eq!(parse_range_header(None, 1000), None);
        assert_eq!(parse_range_header(Some(""), 1000), None);
        assert_eq!(parse_range_header(Some("invalid"), 1000), None);
        // Missing the `bytes=` unit prefix.
        assert_eq!(parse_range_header(Some("0-499"), 1000), None);
        // Wrong unit.
        assert_eq!(parse_range_header(Some("items=0-499"), 1000), None);
    }

    #[test]
    fn parse_range_header_rejects_malformed_spec() {
        // Too many dashes -> more than two parts.
        assert_eq!(parse_range_header(Some("bytes=0-1-2"), 1000), None);
        // Empty start and empty end.
        assert_eq!(parse_range_header(Some("bytes=-"), 1000), None);
        // Non-numeric start.
        assert_eq!(parse_range_header(Some("bytes=abc-499"), 1000), None);
        // Non-numeric end.
        assert_eq!(parse_range_header(Some("bytes=0-xyz"), 1000), None);
        // Non-numeric suffix.
        assert_eq!(parse_range_header(Some("bytes=-abc"), 1000), None);
    }

    #[test]
    fn parse_range_header_zero_length_file_is_unsatisfiable() {
        // A zero-length file has no valid byte range; every form must yield
        // `None` rather than panicking on the internal `file_size - 1`.
        assert_eq!(parse_range_header(Some("bytes=0-"), 0), None);
        assert_eq!(parse_range_header(Some("bytes=-100"), 0), None);
        assert_eq!(parse_range_header(Some("bytes=0-0"), 0), None);
    }

    // ------------------------------------------------------------------
    // generate_simple_playlist
    // ------------------------------------------------------------------

    #[test]
    fn generate_simple_playlist_is_well_formed() {
        let playlist = generate_simple_playlist(42, 123_456);
        assert!(playlist.starts_with("#EXTM3U"));
        assert!(playlist.contains("#EXT-X-VERSION:6"));
        assert!(playlist.contains("#EXT-X-PLAYLIST-TYPE:VOD"));
        assert!(playlist.contains("#EXT-X-TARGETDURATION:10"));
        assert!(playlist.contains("#EXT-X-INDEPENDENT-SEGMENTS"));
        assert!(playlist.contains("#EXTINF:10.0,"));
        assert!(playlist.contains("video.mp4"));
        assert!(playlist.trim_end().ends_with("#EXT-X-ENDLIST"));
    }

    #[test]
    fn generate_simple_playlist_is_stable_regardless_of_inputs() {
        // The current implementation ignores its arguments; assert that
        // contract so a future change is caught by a failing test.
        assert_eq!(
            generate_simple_playlist(1, 0),
            generate_simple_playlist(9_999, u64::MAX)
        );
    }

    // ------------------------------------------------------------------
    // build_event_directory_path
    // ------------------------------------------------------------------

    fn sample_dt() -> chrono::NaiveDateTime {
        NaiveDate::from_ymd_opt(2026, 3, 9)
            .unwrap()
            .and_hms_opt(7, 4, 5)
            .unwrap()
    }

    #[test]
    fn build_event_directory_path_shallow() {
        let path =
            build_event_directory_path("/events", 7, 467, Some(sample_dt()), &Scheme::Shallow);
        assert_eq!(path, PathBuf::from("/events/7/467"));
    }

    #[test]
    fn build_event_directory_path_shallow_ignores_start_time() {
        // Shallow scheme never uses the timestamp.
        let with =
            build_event_directory_path("/events", 7, 467, Some(sample_dt()), &Scheme::Shallow);
        let without = build_event_directory_path("/events", 7, 467, None, &Scheme::Shallow);
        assert_eq!(with, without);
    }

    #[test]
    fn build_event_directory_path_medium() {
        let path =
            build_event_directory_path("/events", 7, 467, Some(sample_dt()), &Scheme::Medium);
        assert_eq!(path, PathBuf::from("/events/7/2026-03-09/467"));
    }

    #[test]
    fn build_event_directory_path_deep() {
        let path = build_event_directory_path("/events", 7, 467, Some(sample_dt()), &Scheme::Deep);
        // Deep uses YY/MM/DD/HH/MM/SS, all two-digit zero-padded.
        assert_eq!(path, PathBuf::from("/events/7/26/03/09/07/04/05"));
    }

    #[test]
    fn build_event_directory_path_deep_falls_back_to_shallow_without_start_time() {
        let path = build_event_directory_path("/events", 7, 467, None, &Scheme::Deep);
        assert_eq!(path, PathBuf::from("/events/7/467"));
    }

    #[test]
    fn build_event_directory_path_medium_falls_back_to_shallow_without_start_time() {
        let path = build_event_directory_path("/events", 7, 467, None, &Scheme::Medium);
        assert_eq!(path, PathBuf::from("/events/7/467"));
    }

    #[test]
    fn is_safe_event_filename_accepts_normal_video_names() {
        assert!(is_safe_event_filename("467-video.mp4"));
        assert!(is_safe_event_filename("467-video.h264.mp4"));
        assert!(is_safe_event_filename("snapshot.jpg"));
        assert!(is_safe_event_filename("a"));
    }

    #[test]
    fn is_safe_event_filename_rejects_traversal_and_separators() {
        // Path components that would escape the event directory
        assert!(!is_safe_event_filename(""));
        assert!(!is_safe_event_filename("."));
        assert!(!is_safe_event_filename(".."));
        assert!(!is_safe_event_filename("../etc/passwd"));
        assert!(!is_safe_event_filename("../../etc/passwd"));
        assert!(!is_safe_event_filename("/etc/passwd"));
        assert!(!is_safe_event_filename("subdir/file.mp4"));
        assert!(!is_safe_event_filename("foo\\bar"));
        assert!(!is_safe_event_filename("foo\0bar"));
    }

    #[test]
    fn select_video_filename_uses_default_when_empty() {
        assert_eq!(select_video_filename(42, ""), "42-video.mp4");
    }

    #[test]
    fn select_video_filename_passes_through_safe_names() {
        assert_eq!(
            select_video_filename(42, "42-video.h264.mp4"),
            "42-video.h264.mp4"
        );
    }

    #[test]
    fn select_video_filename_falls_back_on_unsafe_input() {
        // Path traversal in default_video must NOT be honored.
        assert_eq!(
            select_video_filename(42, "../../../etc/passwd"),
            "42-video.mp4"
        );
        assert_eq!(
            select_video_filename(42, "subdir/escape.mp4"),
            "42-video.mp4"
        );
    }

    #[test]
    fn build_event_directory_path_year_two_digit_wrap() {
        // Year 2008 -> "08"; month/day single-digit zero-padded.
        let dt = NaiveDate::from_ymd_opt(2008, 1, 2)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let path = build_event_directory_path("/srv", 1, 5, Some(dt), &Scheme::Deep);
        assert_eq!(path, PathBuf::from("/srv/1/08/01/02/00/00/00"));
    }
}
