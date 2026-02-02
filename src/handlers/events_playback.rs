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
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tracing::debug;

use crate::error::{AppError, AppResponseError, AppResult};
use crate::server::state::AppState;
use crate::service;

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

/// Get the video file path for an event
async fn get_event_video_path(state: &AppState, event_id: u64) -> AppResult<PathBuf> {
    // Get event from database
    let event = service::events::get_by_id(state, event_id as u32).await?;

    // Get the events directory from config
    let events_dir = state.config.streaming.zoneminder.events_dir.clone();

    // Construct path based on ZoneMinder event storage structure
    // Format: {events_dir}/{monitor_id}/{YYMM}/{DD}/{event_id}/
    let monitor_id = event.monitor_id;

    // The video file is typically named {event_id}-video.mp4
    let video_filename = format!("{}-video.mp4", event_id);

    // Try the standard path first
    let video_path = PathBuf::from(&events_dir)
        .join(monitor_id.to_string())
        .join(&video_filename);

    if tokio::fs::metadata(&video_path).await.is_ok() {
        return Ok(video_path);
    }

    // Try alternative structures with date-based directories
    // ZoneMinder can use different storage schemes
    Err(AppError::NotFoundError(crate::error::Resource {
        resource_type: crate::error::ResourceType::Event,
        details: vec![
            ("event_id".to_string(), event_id.to_string()),
            ("reason".to_string(), "Video file not found".to_string()),
        ],
    }))
}

/// Parse HTTP Range header
fn parse_range_header(range_header: Option<&str>, file_size: u64) -> Option<(u64, u64)> {
    let range_str = range_header?;

    if !range_str.starts_with("bytes=") {
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

    // Get event from database
    let event = service::events::get_by_id(&state, path.id as u32).await?;

    // Get the events directory from config
    let events_dir = &state.config.streaming.zoneminder.events_dir;

    // Try to find thumbnail image
    // ZoneMinder creates snapshot images during events
    let snapshot_path = PathBuf::from(events_dir)
        .join(event.monitor_id.to_string())
        .join(format!("{}-snapshot.jpg", path.id));

    if let Ok(data) = tokio::fs::read(&snapshot_path).await {
        return Ok(Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "image/jpeg")
            .header(header::CACHE_CONTROL, "max-age=86400")
            .body(Body::from(data))
            .unwrap());
    }

    // If no snapshot, try the first frame (analyse image)
    let analyse_path = PathBuf::from(events_dir)
        .join(event.monitor_id.to_string())
        .join(format!("{}-00001-analyse.jpg", path.id));

    if let Ok(data) = tokio::fs::read(&analyse_path).await {
        return Ok(Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "image/jpeg")
            .header(header::CACHE_CONTROL, "max-age=86400")
            .body(Body::from(data))
            .unwrap());
    }

    Err(AppError::NotFoundError(crate::error::Resource {
        resource_type: crate::error::ResourceType::Event,
        details: vec![
            ("event_id".to_string(), path.id.to_string()),
            ("reason".to_string(), "Thumbnail not found".to_string()),
        ],
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_range_header() {
        // Normal range
        assert_eq!(
            parse_range_header(Some("bytes=0-499"), 1000),
            Some((0, 499))
        );
        assert_eq!(
            parse_range_header(Some("bytes=500-999"), 1000),
            Some((500, 999))
        );

        // Open-ended range
        assert_eq!(
            parse_range_header(Some("bytes=500-"), 1000),
            Some((500, 999))
        );

        // Suffix range
        assert_eq!(
            parse_range_header(Some("bytes=-500"), 1000),
            Some((500, 999))
        );

        // Invalid ranges
        assert_eq!(parse_range_header(Some("bytes=500-400"), 1000), None); // start > end
        assert_eq!(parse_range_header(Some("bytes=0-2000"), 1000), None); // end > file_size
        assert_eq!(parse_range_header(Some("invalid"), 1000), None);
        assert_eq!(parse_range_header(None, 1000), None);
    }
}
