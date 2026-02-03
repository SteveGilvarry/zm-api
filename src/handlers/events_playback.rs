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

/// Get the video file path for an event
///
/// ZoneMinder supports multiple storage schemes:
/// - **Deep**: `{storage_path}/{MonitorId}/{YY/MM/DD/HH/MM/SS}/{video_file}`
/// - **Medium**: `{storage_path}/{MonitorId}/{YYYY-MM-DD}/{EventId}/{video_file}`
/// - **Shallow**: `{storage_path}/{MonitorId}/{EventId}/{video_file}`
async fn get_event_video_path(state: &AppState, event_id: u64) -> AppResult<PathBuf> {
    // Get event from database (using repo to get raw entity)
    let event = get_event_entity(state, event_id).await?;

    // Get the video filename from the event (e.g., "467-video.h264.mp4")
    let video_filename = if event.default_video.is_empty() {
        format!("{}-video.mp4", event_id)
    } else {
        event.default_video.clone()
    };

    // Look up storage path from database
    let storage = repo::storage::find_by_id(state.db(), event.storage_id).await?;

    let storage_path = match storage {
        Some(s) => s.path,
        None => {
            // Fall back to config if storage not found in DB
            warn!(
                "Storage {} not found in database, using config default",
                event.storage_id
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
    let storage = repo::storage::find_by_id(state.db(), event.storage_id).await?;

    let storage_path = match storage {
        Some(s) => s.path,
        None => {
            warn!(
                "Storage {} not found in database, using config default",
                event.storage_id
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
