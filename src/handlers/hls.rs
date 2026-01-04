//! HLS streaming HTTP handlers
//!
//! Provides endpoints for HLS playlist and segment delivery.

use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, info};

use crate::error::{AppError, AppResponseError, AppResult};
use crate::server::state::AppState;
use crate::streaming::hls::{HlsSessionManager, HlsSessionStats};

/// Query parameters for LL-HLS blocking reload
#[derive(Debug, Deserialize)]
pub struct LlHlsQuery {
    /// Media Sequence Number to wait for
    #[serde(rename = "_HLS_msn")]
    pub msn: Option<u64>,
    /// Part index to wait for
    #[serde(rename = "_HLS_part")]
    pub part: Option<u32>,
    /// Skip parameter
    #[serde(rename = "_HLS_skip")]
    pub skip: Option<String>,
}

/// Start HLS streaming for a monitor
#[utoipa::path(
    post,
    path = "/api/v3/hls/{camera_id}/start",
    operation_id = "startHlsStream",
    tag = "HLS Streaming",
    params(
        ("camera_id" = u32, Path, description = "Monitor/Camera ID")
    ),
    responses(
        (status = 200, description = "HLS streaming started", body = HlsStartResponse),
        (status = 400, description = "Bad request", body = AppResponseError),
        (status = 409, description = "Session already exists", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn start_hls_stream(
    State(state): State<AppState>,
    Path(camera_id): Path<u32>,
) -> AppResult<Json<HlsStartResponse>> {
    info!("Starting HLS stream for camera {}", camera_id);

    let hls_manager = state.hls_session_manager.as_ref().ok_or_else(|| {
        AppError::ServiceUnavailableError("HLS streaming not configured".to_string())
    })?;

    hls_manager
        .start_session(camera_id)
        .await
        .map_err(|e| AppError::BadRequestError(format!("Failed to start HLS session: {}", e)))?;

    let base_url = format!("/api/v3/hls/{}", camera_id);

    Ok(Json(HlsStartResponse {
        camera_id,
        master_playlist: format!("{}/master.m3u8", base_url),
        status: "started".to_string(),
    }))
}

/// Response for starting HLS stream
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct HlsStartResponse {
    pub camera_id: u32,
    pub master_playlist: String,
    pub status: String,
}

/// Stop HLS streaming for a monitor
#[utoipa::path(
    delete,
    path = "/api/v3/hls/{camera_id}/stop",
    operation_id = "stopHlsStream",
    tag = "HLS Streaming",
    params(
        ("camera_id" = u32, Path, description = "Monitor/Camera ID")
    ),
    responses(
        (status = 200, description = "HLS streaming stopped"),
        (status = 404, description = "Session not found", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn stop_hls_stream(
    State(state): State<AppState>,
    Path(camera_id): Path<u32>,
) -> AppResult<StatusCode> {
    info!("Stopping HLS stream for camera {}", camera_id);

    let hls_manager = state.hls_session_manager.as_ref().ok_or_else(|| {
        AppError::ServiceUnavailableError("HLS streaming not configured".to_string())
    })?;

    hls_manager.stop_session(camera_id).await.map_err(|e| {
        AppError::NotFoundError(crate::error::Resource {
            resource_type: crate::error::ResourceType::Monitor,
            details: vec![
                ("camera_id".to_string(), camera_id.to_string()),
                ("error".to_string(), e.to_string()),
            ],
        })
    })?;

    Ok(StatusCode::OK)
}

/// Get HLS stream statistics
#[utoipa::path(
    get,
    path = "/api/v3/hls/{camera_id}/stats",
    operation_id = "getHlsStats",
    tag = "HLS Streaming",
    params(
        ("camera_id" = u32, Path, description = "Monitor/Camera ID")
    ),
    responses(
        (status = 200, description = "HLS stream statistics", body = HlsStatsResponse),
        (status = 404, description = "Session not found", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn get_hls_stats(
    State(state): State<AppState>,
    Path(camera_id): Path<u32>,
) -> AppResult<Json<HlsStatsResponse>> {
    let hls_manager = state.hls_session_manager.as_ref().ok_or_else(|| {
        AppError::ServiceUnavailableError("HLS streaming not configured".to_string())
    })?;

    let stats = hls_manager.get_stats(camera_id).await.map_err(|e| {
        AppError::NotFoundError(crate::error::Resource {
            resource_type: crate::error::ResourceType::Monitor,
            details: vec![
                ("camera_id".to_string(), camera_id.to_string()),
                ("error".to_string(), e.to_string()),
            ],
        })
    })?;

    Ok(Json(HlsStatsResponse {
        camera_id: stats.monitor_id,
        uptime_seconds: stats.uptime.as_secs_f64(),
        segment_count: stats.segment_count,
        bytes_written: stats.bytes_written,
        viewer_count: stats.viewer_count,
        current_sequence: stats.current_sequence,
        has_init_segment: stats.has_init_segment,
    }))
}

/// Response for HLS statistics
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct HlsStatsResponse {
    pub camera_id: u32,
    pub uptime_seconds: f64,
    pub segment_count: u64,
    pub bytes_written: u64,
    pub viewer_count: usize,
    pub current_sequence: u64,
    pub has_init_segment: bool,
}

/// Get master playlist (ABR)
#[utoipa::path(
    get,
    path = "/api/v3/hls/{camera_id}/master.m3u8",
    operation_id = "getHlsMasterPlaylist",
    tag = "HLS Streaming",
    params(
        ("camera_id" = u32, Path, description = "Monitor/Camera ID")
    ),
    responses(
        (status = 200, description = "Master playlist", content_type = "application/vnd.apple.mpegurl"),
        (status = 404, description = "Session not found", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    )
)]
pub async fn get_master_playlist(
    State(state): State<AppState>,
    Path(camera_id): Path<u32>,
) -> Result<Response, AppError> {
    debug!("Serving master playlist for camera {}", camera_id);

    let hls_manager = state.hls_session_manager.as_ref().ok_or_else(|| {
        AppError::ServiceUnavailableError("HLS streaming not configured".to_string())
    })?;

    let playlist = hls_manager
        .get_master_playlist(camera_id)
        .await
        .map_err(|e| {
            AppError::NotFoundError(crate::error::Resource {
                resource_type: crate::error::ResourceType::Monitor,
                details: vec![("camera_id".to_string(), camera_id.to_string())],
            })
        })?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/vnd.apple.mpegurl")
        .header(header::CACHE_CONTROL, "no-cache, no-store, must-revalidate")
        .body(Body::from(playlist))
        .unwrap())
}

/// Get media playlist (variant)
#[utoipa::path(
    get,
    path = "/api/v3/hls/{camera_id}/live.m3u8",
    operation_id = "getHlsMediaPlaylist",
    tag = "HLS Streaming",
    params(
        ("camera_id" = u32, Path, description = "Monitor/Camera ID"),
    ),
    responses(
        (status = 200, description = "Media playlist", content_type = "application/vnd.apple.mpegurl"),
        (status = 404, description = "Session not found", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    )
)]
pub async fn get_media_playlist(
    State(state): State<AppState>,
    Path(camera_id): Path<u32>,
    Query(ll_hls): Query<LlHlsQuery>,
) -> Result<Response, AppError> {
    debug!("Serving media playlist for camera {}", camera_id);

    let hls_manager = state.hls_session_manager.as_ref().ok_or_else(|| {
        AppError::ServiceUnavailableError("HLS streaming not configured".to_string())
    })?;

    // Handle LL-HLS blocking reload
    if let Some(msn) = ll_hls.msn {
        let timeout = Duration::from_secs(5);
        if let Err(_) = hls_manager.wait_for_segment(camera_id, msn, timeout).await {
            // Timeout - return current playlist anyway
            debug!("LL-HLS wait timeout for camera {} msn {}", camera_id, msn);
        }
    }

    let playlist = hls_manager.get_playlist(camera_id).await.map_err(|e| {
        AppError::NotFoundError(crate::error::Resource {
            resource_type: crate::error::ResourceType::Monitor,
            details: vec![("camera_id".to_string(), camera_id.to_string())],
        })
    })?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/vnd.apple.mpegurl")
        .header(header::CACHE_CONTROL, "no-cache, no-store, must-revalidate")
        .body(Body::from(playlist))
        .unwrap())
}

/// Get initialization segment
#[utoipa::path(
    get,
    path = "/api/v3/hls/{camera_id}/init.mp4",
    operation_id = "getHlsInitSegment",
    tag = "HLS Streaming",
    params(
        ("camera_id" = u32, Path, description = "Monitor/Camera ID")
    ),
    responses(
        (status = 200, description = "Init segment", content_type = "video/mp4"),
        (status = 404, description = "Init segment not ready", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    )
)]
pub async fn get_init_segment(
    State(state): State<AppState>,
    Path(camera_id): Path<u32>,
) -> Result<Response, AppError> {
    debug!("Serving init segment for camera {}", camera_id);

    let hls_manager = state.hls_session_manager.as_ref().ok_or_else(|| {
        AppError::ServiceUnavailableError("HLS streaming not configured".to_string())
    })?;

    let data = hls_manager.get_init_segment(camera_id).await.map_err(|e| {
        AppError::NotFoundError(crate::error::Resource {
            resource_type: crate::error::ResourceType::Monitor,
            details: vec![
                ("camera_id".to_string(), camera_id.to_string()),
                ("segment".to_string(), "init.mp4".to_string()),
            ],
        })
    })?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "video/mp4")
        .header(header::CACHE_CONTROL, "max-age=31536000") // Init segment is immutable
        .body(Body::from(data))
        .unwrap())
}

/// Segment path parameters
#[derive(Debug, Deserialize)]
pub struct SegmentPath {
    pub camera_id: u32,
    pub segment: String,
}

/// Get media segment
#[utoipa::path(
    get,
    path = "/api/v3/hls/{camera_id}/{segment}",
    operation_id = "getHlsSegment",
    tag = "HLS Streaming",
    params(
        ("camera_id" = u32, Path, description = "Monitor/Camera ID"),
        ("segment" = String, Path, description = "Segment filename (e.g., segment_00001.m4s)")
    ),
    responses(
        (status = 200, description = "Media segment", content_type = "video/iso.segment"),
        (status = 404, description = "Segment not found", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    )
)]
pub async fn get_segment(
    State(state): State<AppState>,
    Path(path): Path<SegmentPath>,
) -> Result<Response, AppError> {
    debug!(
        "Serving segment {} for camera {}",
        path.segment, path.camera_id
    );

    // Parse sequence number from segment name
    // Format: segment_00001.m4s or segment_00001.0.m4s (partial)
    let sequence = parse_segment_sequence(&path.segment).ok_or_else(|| {
        AppError::BadRequestError(format!("Invalid segment name: {}", path.segment))
    })?;

    let hls_manager = state.hls_session_manager.as_ref().ok_or_else(|| {
        AppError::ServiceUnavailableError("HLS streaming not configured".to_string())
    })?;

    let data = hls_manager
        .get_segment(path.camera_id, sequence)
        .await
        .map_err(|e| {
            AppError::NotFoundError(crate::error::Resource {
                resource_type: crate::error::ResourceType::Monitor,
                details: vec![
                    ("camera_id".to_string(), path.camera_id.to_string()),
                    ("segment".to_string(), path.segment.clone()),
                ],
            })
        })?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "video/iso.segment")
        .header(header::CACHE_CONTROL, "max-age=31536000") // Segments are immutable
        .body(Body::from(data))
        .unwrap())
}

/// Parse sequence number from segment filename
fn parse_segment_sequence(filename: &str) -> Option<u64> {
    // segment_00001.m4s -> 1
    // segment_00001.0.m4s -> 1 (partial)
    let name = filename.strip_suffix(".m4s")?;
    let parts: Vec<&str> = name.split('_').collect();

    if parts.len() >= 2 && parts[0] == "segment" {
        // Handle partial segments: segment_00001.0 -> take first part
        let seq_part = parts[1].split('.').next()?;
        seq_part.parse().ok()
    } else {
        None
    }
}

/// List active HLS sessions
#[utoipa::path(
    get,
    path = "/api/v3/hls/sessions",
    operation_id = "listHlsSessions",
    tag = "HLS Streaming",
    responses(
        (status = 200, description = "List of active HLS sessions", body = Vec<u32>),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn list_sessions(State(state): State<AppState>) -> AppResult<Json<Vec<u32>>> {
    let hls_manager = state.hls_session_manager.as_ref().ok_or_else(|| {
        AppError::ServiceUnavailableError("HLS streaming not configured".to_string())
    })?;

    let sessions = hls_manager.list_sessions().await;
    Ok(Json(sessions))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_segment_sequence() {
        assert_eq!(parse_segment_sequence("segment_00001.m4s"), Some(1));
        assert_eq!(parse_segment_sequence("segment_00100.m4s"), Some(100));
        assert_eq!(parse_segment_sequence("segment_00001.0.m4s"), Some(1));
        assert_eq!(parse_segment_sequence("segment_00001.5.m4s"), Some(1));
        assert_eq!(parse_segment_sequence("invalid.m4s"), None);
        assert_eq!(parse_segment_sequence("segment_abc.m4s"), None);
    }
}
