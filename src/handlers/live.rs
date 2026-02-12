//! Live streaming HTTP handlers
//!
//! Provides unified endpoints for live streaming via various protocols
//! (HLS, WebRTC, MSE) using the FIFO-based source router.

use axum::{
    body::Body,
    extract::{
        ws::{Message, WebSocket},
        Path, Query, State, WebSocketUpgrade,
    },
    http::{header, StatusCode},
    response::Response,
    Json,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};

use crate::error::{AppError, AppResponseError, AppResult};
use crate::server::state::AppState;
use crate::streaming::live::mse::{MseLiveConfig, MseLiveManager};
use crate::streaming::live::webrtc::{
    extract_profile_level_id, AccessUnitAssembler, WebRtcLiveConfig, WebRtcLiveManager,
};
use crate::streaming::live::{CoordinatorError, LiveStreamConfig};
use crate::streaming::source::VideoCodec;

// ============================================================================
// DTOs
// ============================================================================

/// Request to start a live stream
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct StartLiveRequest {
    /// Enable HLS output
    #[serde(default = "default_true")]
    pub enable_hls: bool,
    /// Enable WebRTC output
    #[serde(default)]
    pub enable_webrtc: bool,
    /// Enable MSE output
    #[serde(default)]
    pub enable_mse: bool,
}

fn default_true() -> bool {
    true
}

impl Default for StartLiveRequest {
    fn default() -> Self {
        Self {
            enable_hls: true,
            enable_webrtc: false,
            enable_mse: false,
        }
    }
}

/// Response for starting a live stream
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct StartLiveResponse {
    pub monitor_id: u32,
    pub status: String,
    pub hls_playlist: Option<String>,
    pub webrtc_signaling: Option<String>,
    pub mse_websocket: Option<String>,
}

/// Response for live stream statistics
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct LiveStatsResponse {
    pub monitor_id: u32,
    pub status: String,
    pub packets_processed: u64,
    pub errors: u64,
    pub uptime_seconds: f64,
    pub protocols: LiveProtocolStatus,
}

/// Protocol status in live stats
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct LiveProtocolStatus {
    pub hls: bool,
    pub webrtc: bool,
    pub mse: bool,
}

/// Query parameters for HLS media playlist (LL-HLS support)
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

// ============================================================================
// Unified Live Stream Control
// ============================================================================

/// Start live streaming for a monitor
#[utoipa::path(
    post,
    path = "/api/v3/live/{monitor_id}/start",
    operation_id = "startLiveStream",
    tag = "Live Streaming",
    params(
        ("monitor_id" = u32, Path, description = "Monitor/Camera ID")
    ),
    request_body(content = StartLiveRequest, description = "Stream configuration"),
    responses(
        (status = 200, description = "Live streaming started", body = StartLiveResponse),
        (status = 400, description = "Bad request", body = AppResponseError),
        (status = 409, description = "Session already exists", body = AppResponseError),
        (status = 503, description = "Service unavailable", body = AppResponseError)
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn start_live_stream(
    State(state): State<AppState>,
    Path(monitor_id): Path<u32>,
    Json(request): Json<StartLiveRequest>,
) -> AppResult<Json<StartLiveResponse>> {
    info!("Starting live stream for monitor {}", monitor_id);

    let coordinator = state.live_coordinator.as_ref().ok_or_else(|| {
        AppError::ServiceUnavailableError("Live streaming not configured".to_string())
    })?;

    let config = LiveStreamConfig {
        enable_hls: request.enable_hls,
        enable_webrtc: request.enable_webrtc,
        enable_mse: request.enable_mse,
    };

    coordinator
        .start_session(monitor_id, config.clone())
        .await
        .map_err(|e| match e {
            CoordinatorError::SessionExists(_) => AppError::ConflictError(format!(
                "Live stream already exists for monitor {}",
                monitor_id
            )),
            CoordinatorError::SourceNotAvailable(_) => {
                AppError::NotFoundError(crate::error::Resource {
                    resource_type: crate::error::ResourceType::Monitor,
                    details: vec![
                        ("monitor_id".to_string(), monitor_id.to_string()),
                        ("reason".to_string(), "FIFO not available".to_string()),
                    ],
                })
            }
            _ => AppError::BadRequestError(format!("Failed to start live stream: {}", e)),
        })?;

    let base_url = format!("/api/v3/live/{}", monitor_id);

    Ok(Json(StartLiveResponse {
        monitor_id,
        status: "started".to_string(),
        hls_playlist: if config.enable_hls {
            Some(format!("{}/hls/live.m3u8", base_url))
        } else {
            None
        },
        webrtc_signaling: if config.enable_webrtc {
            Some(format!("{}/webrtc/ws", base_url))
        } else {
            None
        },
        mse_websocket: if config.enable_mse {
            Some(format!("{}/mse/ws", base_url))
        } else {
            None
        },
    }))
}

/// Stop live streaming for a monitor
///
/// This endpoint is idempotent: stopping a non-active stream returns 204.
#[utoipa::path(
    delete,
    path = "/api/v3/live/{monitor_id}/stop",
    operation_id = "stopLiveStream",
    tag = "Live Streaming",
    params(
        ("monitor_id" = u32, Path, description = "Monitor/Camera ID")
    ),
    responses(
        (status = 204, description = "Live streaming stopped (or was not active)"),
        (status = 503, description = "Service unavailable", body = AppResponseError)
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn stop_live_stream(
    State(state): State<AppState>,
    Path(monitor_id): Path<u32>,
) -> AppResult<StatusCode> {
    info!("Stopping live stream for monitor {}", monitor_id);

    let coordinator = state.live_coordinator.as_ref().ok_or_else(|| {
        AppError::ServiceUnavailableError("Live streaming not configured".to_string())
    })?;

    match coordinator.stop_session(monitor_id).await {
        Ok(()) => {}
        Err(CoordinatorError::SessionNotFound(_)) => {
            debug!(
                "Stop requested for non-active monitor {}, returning 204",
                monitor_id
            );
        }
        Err(e) => {
            return Err(AppError::BadRequestError(format!(
                "Failed to stop live stream: {}",
                e
            )));
        }
    }

    Ok(StatusCode::NO_CONTENT)
}

/// Get live stream statistics
#[utoipa::path(
    get,
    path = "/api/v3/live/{monitor_id}/stats",
    operation_id = "getLiveStats",
    tag = "Live Streaming",
    params(
        ("monitor_id" = u32, Path, description = "Monitor/Camera ID")
    ),
    responses(
        (status = 200, description = "Live stream statistics", body = LiveStatsResponse),
        (status = 404, description = "Session not found", body = AppResponseError),
        (status = 503, description = "Service unavailable", body = AppResponseError)
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn get_live_stats(
    State(state): State<AppState>,
    Path(monitor_id): Path<u32>,
) -> AppResult<Json<LiveStatsResponse>> {
    let coordinator = state.live_coordinator.as_ref().ok_or_else(|| {
        AppError::ServiceUnavailableError("Live streaming not configured".to_string())
    })?;

    let stats = coordinator.get_stats(monitor_id).await.ok_or_else(|| {
        AppError::NotFoundError(crate::error::Resource {
            resource_type: crate::error::ResourceType::Monitor,
            details: vec![("monitor_id".to_string(), monitor_id.to_string())],
        })
    })?;

    Ok(Json(LiveStatsResponse {
        monitor_id: stats.monitor_id,
        status: format!("{:?}", stats.status).to_lowercase(),
        packets_processed: stats.packets_processed,
        errors: stats.errors,
        uptime_seconds: stats.uptime_seconds,
        protocols: LiveProtocolStatus {
            hls: stats.hls_enabled,
            webrtc: stats.webrtc_enabled,
            mse: stats.mse_enabled,
        },
    }))
}

/// List all active live streams
#[utoipa::path(
    get,
    path = "/api/v3/live/sessions",
    operation_id = "listLiveSessions",
    tag = "Live Streaming",
    responses(
        (status = 200, description = "List of active live sessions", body = Vec<u32>),
        (status = 503, description = "Service unavailable", body = AppResponseError)
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn list_live_sessions(State(state): State<AppState>) -> AppResult<Json<Vec<u32>>> {
    let coordinator = state.live_coordinator.as_ref().ok_or_else(|| {
        AppError::ServiceUnavailableError("Live streaming not configured".to_string())
    })?;

    let sessions = coordinator.list_sessions().await;
    Ok(Json(sessions))
}

// ============================================================================
// HLS Endpoints (proxied through /api/v3/live/{monitor_id}/hls/...)
// ============================================================================

/// Get HLS master playlist
#[utoipa::path(
    get,
    path = "/api/v3/live/{monitor_id}/hls/master.m3u8",
    operation_id = "getLiveMasterPlaylist",
    tag = "Live Streaming",
    params(
        ("monitor_id" = u32, Path, description = "Monitor/Camera ID")
    ),
    responses(
        (status = 200, description = "Master playlist", content_type = "application/vnd.apple.mpegurl"),
        (status = 404, description = "Session not found", body = AppResponseError)
    )
)]
pub async fn get_live_master_playlist(
    State(state): State<AppState>,
    Path(monitor_id): Path<u32>,
) -> Result<Response, AppError> {
    debug!("Serving live master playlist for monitor {}", monitor_id);

    let hls_manager = state.hls_session_manager.as_ref().ok_or_else(|| {
        AppError::ServiceUnavailableError("HLS streaming not configured".to_string())
    })?;

    let playlist = hls_manager
        .get_master_playlist(monitor_id)
        .await
        .map_err(|_| {
            AppError::NotFoundError(crate::error::Resource {
                resource_type: crate::error::ResourceType::Monitor,
                details: vec![("monitor_id".to_string(), monitor_id.to_string())],
            })
        })?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/vnd.apple.mpegurl")
        .header(header::CACHE_CONTROL, "no-cache, no-store, must-revalidate")
        .body(Body::from(playlist))
        .unwrap())
}

/// Get HLS media playlist
#[utoipa::path(
    get,
    path = "/api/v3/live/{monitor_id}/hls/live.m3u8",
    operation_id = "getLiveMediaPlaylist",
    tag = "Live Streaming",
    params(
        ("monitor_id" = u32, Path, description = "Monitor/Camera ID")
    ),
    responses(
        (status = 200, description = "Media playlist", content_type = "application/vnd.apple.mpegurl"),
        (status = 404, description = "Session not found", body = AppResponseError)
    )
)]
pub async fn get_live_media_playlist(
    State(state): State<AppState>,
    Path(monitor_id): Path<u32>,
    Query(ll_hls): Query<LlHlsQuery>,
) -> Result<Response, AppError> {
    debug!("Serving live media playlist for monitor {}", monitor_id);

    let hls_manager = state.hls_session_manager.as_ref().ok_or_else(|| {
        AppError::ServiceUnavailableError("HLS streaming not configured".to_string())
    })?;

    // Handle LL-HLS blocking reload
    if let Some(msn) = ll_hls.msn {
        let timeout = Duration::from_secs(5);
        if hls_manager
            .wait_for_segment(monitor_id, msn, timeout)
            .await
            .is_err()
        {
            debug!("LL-HLS wait timeout for monitor {} msn {}", monitor_id, msn);
        }
    }

    let playlist = hls_manager.get_playlist(monitor_id).await.map_err(|_| {
        AppError::NotFoundError(crate::error::Resource {
            resource_type: crate::error::ResourceType::Monitor,
            details: vec![("monitor_id".to_string(), monitor_id.to_string())],
        })
    })?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/vnd.apple.mpegurl")
        .header(header::CACHE_CONTROL, "no-cache, no-store, must-revalidate")
        .body(Body::from(playlist))
        .unwrap())
}

/// Get HLS init segment
#[utoipa::path(
    get,
    path = "/api/v3/live/{monitor_id}/hls/init.mp4",
    operation_id = "getLiveInitSegment",
    tag = "Live Streaming",
    params(
        ("monitor_id" = u32, Path, description = "Monitor/Camera ID")
    ),
    responses(
        (status = 200, description = "Init segment", content_type = "video/mp4"),
        (status = 404, description = "Init segment not ready", body = AppResponseError)
    )
)]
pub async fn get_live_init_segment(
    State(state): State<AppState>,
    Path(monitor_id): Path<u32>,
) -> Result<Response, AppError> {
    debug!("Serving live init segment for monitor {}", monitor_id);

    let hls_manager = state.hls_session_manager.as_ref().ok_or_else(|| {
        AppError::ServiceUnavailableError("HLS streaming not configured".to_string())
    })?;

    let data = hls_manager
        .get_init_segment(monitor_id)
        .await
        .map_err(|_| {
            AppError::NotFoundError(crate::error::Resource {
                resource_type: crate::error::ResourceType::Monitor,
                details: vec![
                    ("monitor_id".to_string(), monitor_id.to_string()),
                    ("segment".to_string(), "init.mp4".to_string()),
                ],
            })
        })?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "video/mp4")
        .header(header::CACHE_CONTROL, "max-age=31536000")
        .body(Body::from(data))
        .unwrap())
}

/// Path parameters for HLS segment
#[derive(Debug, Deserialize)]
pub struct HlsSegmentPath {
    pub monitor_id: u32,
    pub segment: String,
}

/// Get HLS media segment
#[utoipa::path(
    get,
    path = "/api/v3/live/{monitor_id}/hls/{segment}",
    operation_id = "getLiveSegment",
    tag = "Live Streaming",
    params(
        ("monitor_id" = u32, Path, description = "Monitor/Camera ID"),
        ("segment" = String, Path, description = "Segment filename (e.g., segment_00001.m4s)")
    ),
    responses(
        (status = 200, description = "Media segment", content_type = "video/iso.segment"),
        (status = 404, description = "Segment not found", body = AppResponseError)
    )
)]
pub async fn get_live_segment(
    State(state): State<AppState>,
    Path(path): Path<HlsSegmentPath>,
) -> Result<Response, AppError> {
    debug!(
        "Serving live segment {} for monitor {}",
        path.segment, path.monitor_id
    );

    let sequence = parse_segment_sequence(&path.segment).ok_or_else(|| {
        AppError::BadRequestError(format!("Invalid segment name: {}", path.segment))
    })?;

    let hls_manager = state.hls_session_manager.as_ref().ok_or_else(|| {
        AppError::ServiceUnavailableError("HLS streaming not configured".to_string())
    })?;

    let data = hls_manager
        .get_segment(path.monitor_id, sequence)
        .await
        .map_err(|_| {
            AppError::NotFoundError(crate::error::Resource {
                resource_type: crate::error::ResourceType::Monitor,
                details: vec![
                    ("monitor_id".to_string(), path.monitor_id.to_string()),
                    ("segment".to_string(), path.segment.clone()),
                ],
            })
        })?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "video/iso.segment")
        .header(header::CACHE_CONTROL, "max-age=31536000")
        .body(Body::from(data))
        .unwrap())
}

/// Parse sequence number from segment filename
fn parse_segment_sequence(filename: &str) -> Option<u64> {
    let name = filename.strip_suffix(".m4s")?;
    let parts: Vec<&str> = name.split('_').collect();

    if parts.len() >= 2 && parts[0] == "segment" {
        let seq_part = parts[1].split('.').next()?;
        seq_part.parse().ok()
    } else {
        None
    }
}

// ============================================================================
// MSE Endpoints (WebSocket fMP4 streaming)
// ============================================================================

/// WebSocket message types for MSE streaming
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum MseWebSocketMessage {
    /// Init segment (sent first)
    Init {
        monitor_id: u32,
        codec: String,
        #[serde(with = "base64_serde")]
        data: Vec<u8>,
    },
    /// Media segment
    Segment {
        monitor_id: u32,
        sequence: u64,
        duration_ms: u32,
        is_keyframe: bool,
        timestamp_us: u64,
        #[serde(with = "base64_serde")]
        data: Vec<u8>,
    },
    /// Ping message (keepalive)
    Ping,
    /// Pong response
    Pong,
    /// Error message
    Error { message: String },
    /// Info about the stream
    Info {
        monitor_id: u32,
        session_id: String,
        codec: String,
    },
}

mod base64_serde {
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&STANDARD.encode(bytes))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        STANDARD.decode(s).map_err(serde::de::Error::custom)
    }
}

/// MSE session info response
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct MseSessionInfoResponse {
    pub session_id: String,
    pub monitor_id: u32,
    pub websocket_url: String,
    pub init_url: String,
}

/// WebSocket handler for MSE live streaming
#[utoipa::path(
    get,
    path = "/api/v3/live/{monitor_id}/mse/ws",
    operation_id = "mseWebSocketStream",
    tag = "Live Streaming",
    params(
        ("monitor_id" = u32, Path, description = "Monitor/Camera ID")
    ),
    responses(
        (status = 101, description = "WebSocket connection upgraded for MSE streaming"),
        (status = 404, description = "Monitor not found", body = AppResponseError),
        (status = 503, description = "Service unavailable", body = AppResponseError)
    )
)]
pub async fn mse_websocket_handler(
    State(state): State<AppState>,
    Path(monitor_id): Path<u32>,
    ws: WebSocketUpgrade,
) -> Result<Response, AppError> {
    info!(
        "MSE WebSocket connection request for monitor {}",
        monitor_id
    );

    // Check if source router is available
    let source_router = state.source_router.as_ref().ok_or_else(|| {
        AppError::ServiceUnavailableError("Live streaming not configured".to_string())
    })?;

    // Check if the source is available
    if !source_router.is_available(monitor_id) {
        return Err(AppError::NotFoundError(crate::error::Resource {
            resource_type: crate::error::ResourceType::Monitor,
            details: vec![
                ("monitor_id".to_string(), monitor_id.to_string()),
                ("reason".to_string(), "FIFO not available".to_string()),
            ],
        }));
    }

    let router = Arc::clone(source_router);

    Ok(ws.on_upgrade(move |socket| handle_mse_websocket(router, monitor_id, socket)))
}

async fn handle_mse_websocket(
    source_router: Arc<crate::streaming::source::SourceRouter>,
    monitor_id: u32,
    socket: WebSocket,
) {
    info!("MSE WebSocket connected for monitor {}", monitor_id);

    // Create MSE manager for this session
    let mse_manager = MseLiveManager::new(MseLiveConfig::default());

    // Create session
    let session = match mse_manager.create_session(monitor_id) {
        Ok(s) => s,
        Err(e) => {
            error!(
                "Failed to create MSE session for monitor {}: {}",
                monitor_id, e
            );
            return;
        }
    };

    let session_id = session.id.to_string();
    info!(
        "Created MSE session {} for monitor {}",
        session_id, monitor_id
    );

    // Get source and subscribe to video packets
    let source = match source_router.get_source(monitor_id).await {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to get source for monitor {}: {}", monitor_id, e);
            return;
        }
    };

    let mut video_rx = source.subscribe_video();
    let mut segment_rx = session.subscribe();

    let (mut ws_sender, mut ws_receiver) = socket.split();

    // Send session info
    let info_msg = MseWebSocketMessage::Info {
        monitor_id,
        session_id: session_id.clone(),
        codec: source.codec().await.as_str().to_string(),
    };
    if let Ok(json) = serde_json::to_string(&info_msg) {
        if ws_sender.send(Message::Text(json.into())).await.is_err() {
            error!("Failed to send session info");
            return;
        }
    }

    // Track if init segment has been sent
    let mut init_sent = false;

    // Main event loop
    loop {
        tokio::select! {
            // Handle incoming video packets from source
            result = video_rx.recv() => {
                match result {
                    Ok(packet) => {
                        // Process packet through MSE session
                        session.process_packet(&packet).await;
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        warn!("MSE session lagged {} packets for monitor {}", n, monitor_id);
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        info!("Video source closed for monitor {}", monitor_id);
                        break;
                    }
                }
            }

            // Handle outgoing segments to WebSocket
            result = segment_rx.recv() => {
                match result {
                    Ok(segment) => {
                        // Send init segment first if not sent
                        if !init_sent {
                            if let Some(init) = session.get_init_segment().await {
                                let init_msg = MseWebSocketMessage::Init {
                                    monitor_id,
                                    codec: source.codec().await.as_str().to_string(),
                                    data: init.data,
                                };
                                if let Ok(json) = serde_json::to_string(&init_msg) {
                                    if ws_sender.send(Message::Text(json.into())).await.is_err() {
                                        break;
                                    }
                                    init_sent = true;
                                    debug!("Sent init segment for monitor {}", monitor_id);
                                }
                            }
                        }

                        // Send segment
                        let segment_msg = MseWebSocketMessage::Segment {
                            monitor_id,
                            sequence: segment.sequence,
                            duration_ms: segment.duration_ms,
                            is_keyframe: segment.is_keyframe,
                            timestamp_us: segment.timestamp_us,
                            data: segment.data,
                        };
                        if let Ok(json) = serde_json::to_string(&segment_msg) {
                            if ws_sender.send(Message::Text(json.into())).await.is_err() {
                                warn!("Failed to send segment to WebSocket for monitor {}", monitor_id);
                                break;
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        warn!("MSE WebSocket client lagged {} segments for monitor {}", n, monitor_id);
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        info!("Segment channel closed for monitor {}", monitor_id);
                        break;
                    }
                }
            }

            // Handle incoming WebSocket messages
            msg = ws_receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        // Handle ping/pong
                        if let Ok(MseWebSocketMessage::Ping) = serde_json::from_str::<MseWebSocketMessage>(&text) {
                            let pong = MseWebSocketMessage::Pong;
                            if let Ok(json) = serde_json::to_string(&pong) {
                                let _ = ws_sender.send(Message::Text(json.into())).await;
                            }
                        }
                    }
                    Some(Ok(Message::Ping(data))) => {
                        let _ = ws_sender.send(Message::Pong(data)).await;
                    }
                    Some(Ok(Message::Close(_))) => {
                        info!("MSE WebSocket client disconnected for monitor {}", monitor_id);
                        break;
                    }
                    Some(Err(e)) => {
                        error!("MSE WebSocket error for monitor {}: {}", monitor_id, e);
                        break;
                    }
                    None => break,
                    _ => {}
                }
            }
        }
    }

    // Cleanup
    let _ = mse_manager.remove_session(&session_id);
    info!("MSE WebSocket handler finished for monitor {}", monitor_id);
}

/// Get MSE init segment for a monitor
#[utoipa::path(
    get,
    path = "/api/v3/live/{monitor_id}/mse/init.mp4",
    operation_id = "getMseInitSegment",
    tag = "Live Streaming",
    params(
        ("monitor_id" = u32, Path, description = "Monitor/Camera ID")
    ),
    responses(
        (status = 200, description = "MSE init segment", content_type = "video/mp4"),
        (status = 404, description = "Init segment not ready", body = AppResponseError),
        (status = 503, description = "Service unavailable", body = AppResponseError)
    )
)]
pub async fn get_mse_init_segment(
    State(state): State<AppState>,
    Path(monitor_id): Path<u32>,
) -> Result<Response, AppError> {
    debug!("Serving MSE init segment for monitor {}", monitor_id);

    // For now, use the HLS init segment since they're the same format
    let hls_manager = state
        .hls_session_manager
        .as_ref()
        .ok_or_else(|| AppError::ServiceUnavailableError("Streaming not configured".to_string()))?;

    let data = hls_manager
        .get_init_segment(monitor_id)
        .await
        .map_err(|_| {
            AppError::NotFoundError(crate::error::Resource {
                resource_type: crate::error::ResourceType::Monitor,
                details: vec![
                    ("monitor_id".to_string(), monitor_id.to_string()),
                    ("segment".to_string(), "init.mp4".to_string()),
                ],
            })
        })?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "video/mp4")
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::from(data))
        .unwrap())
}

// ============================================================================
// WebRTC Endpoints (WebSocket signaling)
// ============================================================================

/// WebRTC signaling message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum WebRtcSignalingMessage {
    /// Server offer to client
    Offer {
        session_id: String,
        sdp: String,
    },
    /// Client answer to server
    Answer {
        session_id: String,
        sdp: String,
    },
    /// ICE candidate exchange
    IceCandidate {
        session_id: String,
        candidate: String,
        #[serde(rename = "sdpMid")]
        sdp_mid: Option<String>,
        #[serde(rename = "sdpMLineIndex")]
        sdp_mline_index: Option<u16>,
    },
    /// Session ready notification
    Ready {
        session_id: String,
        monitor_id: u32,
    },
    /// Error message
    Error {
        message: String,
    },
    /// Ping/Pong for keepalive
    Ping,
    Pong,
}

/// WebSocket handler for WebRTC signaling
#[utoipa::path(
    get,
    path = "/api/v3/live/{monitor_id}/webrtc/ws",
    operation_id = "webrtcSignalingStream",
    tag = "Live Streaming",
    params(
        ("monitor_id" = u32, Path, description = "Monitor/Camera ID")
    ),
    responses(
        (status = 101, description = "WebSocket connection upgraded for WebRTC signaling"),
        (status = 404, description = "Monitor not found", body = AppResponseError),
        (status = 503, description = "Service unavailable", body = AppResponseError)
    )
)]
pub async fn webrtc_websocket_handler(
    State(state): State<AppState>,
    Path(monitor_id): Path<u32>,
    ws: WebSocketUpgrade,
) -> Result<Response, AppError> {
    info!(
        "WebRTC WebSocket signaling request for monitor {}",
        monitor_id
    );

    // Check if source router is available
    let source_router = state.source_router.as_ref().ok_or_else(|| {
        AppError::ServiceUnavailableError("Live streaming not configured".to_string())
    })?;

    // Check if the source is available
    if !source_router.is_available(monitor_id) {
        return Err(AppError::NotFoundError(crate::error::Resource {
            resource_type: crate::error::ResourceType::Monitor,
            details: vec![
                ("monitor_id".to_string(), monitor_id.to_string()),
                ("reason".to_string(), "FIFO not available".to_string()),
            ],
        }));
    }

    let router = Arc::clone(source_router);

    Ok(ws.on_upgrade(move |socket| handle_webrtc_websocket(router, monitor_id, socket)))
}

async fn handle_webrtc_websocket(
    source_router: Arc<crate::streaming::source::SourceRouter>,
    monitor_id: u32,
    socket: WebSocket,
) {
    info!(
        "WebRTC signaling WebSocket connected for monitor {}",
        monitor_id
    );

    // Create WebRTC manager for this session
    let webrtc_manager = WebRtcLiveManager::new(WebRtcLiveConfig::default());

    // Get source to determine codec
    let source = match source_router.get_source(monitor_id).await {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to get source for monitor {}: {}", monitor_id, e);
            return;
        }
    };

    // Wait a moment for codec detection
    tokio::time::sleep(Duration::from_millis(100)).await;
    let codec = source.codec().await;
    let codec = if codec == VideoCodec::Unknown {
        // Default to H264 if not detected yet
        VideoCodec::H264
    } else {
        codec
    };

    // Detect the actual H264 profile from the first SPS NAL so the SDP
    // offer advertises the correct profile-level-id.  Without this, the
    // browser may refuse to create a decoder (decoderImpl=none).
    let mut profile_level_id: Option<String> = None;
    if codec == VideoCodec::H264 {
        let mut detect_rx = source.subscribe_video();
        let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
        loop {
            tokio::select! {
                result = detect_rx.recv() => {
                    match result {
                        Ok(packet) => {
                            if let Some(plid) = extract_profile_level_id(&packet.data) {
                                info!(
                                    "Detected H264 profile-level-id={} for monitor {}",
                                    plid, monitor_id
                                );
                                profile_level_id = Some(plid);
                                break;
                            }
                        }
                        Err(broadcast::error::RecvError::Lagged(_)) => continue,
                        Err(broadcast::error::RecvError::Closed) => break,
                    }
                }
                _ = tokio::time::sleep_until(deadline) => {
                    warn!(
                        "Timeout waiting for SPS NAL on monitor {}, using default profile",
                        monitor_id
                    );
                    break;
                }
            }
        }
    }

    // Create WebRTC session with offer and state watch channel
    let (session_id, offer, mut pc_state_rx) = match webrtc_manager
        .create_session(monitor_id, codec, profile_level_id.as_deref())
        .await
    {
        Ok(result) => result,
        Err(e) => {
            error!(
                "Failed to create WebRTC session for monitor {}: {}",
                monitor_id, e
            );
            return;
        }
    };

    info!(
        "Created WebRTC session {} for monitor {}",
        session_id, monitor_id
    );

    let (mut ws_sender, mut ws_receiver) = socket.split();

    // Send offer to client
    let offer_msg = WebRtcSignalingMessage::Offer {
        session_id: session_id.clone(),
        sdp: offer.sdp,
    };
    if let Ok(json) = serde_json::to_string(&offer_msg) {
        if ws_sender.send(Message::Text(json.into())).await.is_err() {
            error!("Failed to send offer to client");
            return;
        }
    }

    // Subscribe to video packets for streaming
    let mut video_rx = source.subscribe_video();

    // Assembles individual NAL units into complete access units (frames)
    // so the H264 payloader assigns one RTP timestamp per picture.
    let mut au_assembler = AccessUnitAssembler::new();

    // Track if DTLS handshake is complete and we can stream media
    let mut streaming_started = false;
    // Track if we've set the SDP answer (waiting for DTLS)
    let mut answer_set = false;
    // Connection timeout: 30s after answer is set, if not streaming yet
    let connection_timeout = tokio::time::sleep(Duration::from_secs(30));
    tokio::pin!(connection_timeout);

    // Main event loop
    loop {
        tokio::select! {
            // Handle incoming signaling messages
            msg = ws_receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if let Ok(signaling_msg) = serde_json::from_str::<WebRtcSignalingMessage>(&text) {
                            match signaling_msg {
                                WebRtcSignalingMessage::Answer { session_id: sid, sdp } => {
                                    if sid == session_id {
                                        let answer = match webrtc::peer_connection::sdp::session_description::RTCSessionDescription::answer(sdp) {
                                            Ok(a) => a,
                                            Err(e) => {
                                                error!("Invalid SDP answer from client: {}", e);
                                                let error_msg = WebRtcSignalingMessage::Error {
                                                    message: format!("Invalid SDP answer: {}", e),
                                                };
                                                if let Ok(json) = serde_json::to_string(&error_msg) {
                                                    let _ = ws_sender.send(Message::Text(json.into())).await;
                                                }
                                                continue;
                                            }
                                        };
                                        if let Err(e) = webrtc_manager.set_answer(&session_id, answer).await {
                                            error!("Failed to set answer: {}", e);
                                            let error_msg = WebRtcSignalingMessage::Error {
                                                message: e.to_string(),
                                            };
                                            if let Ok(json) = serde_json::to_string(&error_msg) {
                                                let _ = ws_sender.send(Message::Text(json.into())).await;
                                            }
                                        } else {
                                            answer_set = true;
                                            // Reset timeout to 30s from now
                                            connection_timeout.as_mut().reset(tokio::time::Instant::now() + Duration::from_secs(30));
                                            info!("SDP answer set for session {}, waiting for DTLS completion", session_id);

                                            // Check if peer connection already reached Connected
                                            // before we started watching. This handles the race
                                            // where DTLS completes very quickly (e.g. on LAN).
                                            let current_state = *pc_state_rx.borrow_and_update();
                                            match current_state {
                                                webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState::Connected => {
                                                    streaming_started = true;
                                                    let ready_msg = WebRtcSignalingMessage::Ready {
                                                        session_id: session_id.clone(),
                                                        monitor_id,
                                                    };
                                                    if let Ok(json) = serde_json::to_string(&ready_msg) {
                                                        let _ = ws_sender.send(Message::Text(json.into())).await;
                                                    }
                                                    info!("WebRTC session {} already connected, streaming started", session_id);
                                                }
                                                webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState::Failed => {
                                                    error!("WebRTC peer connection already failed for session {}", session_id);
                                                    let error_msg = WebRtcSignalingMessage::Error {
                                                        message: "Peer connection failed".to_string(),
                                                    };
                                                    if let Ok(json) = serde_json::to_string(&error_msg) {
                                                        let _ = ws_sender.send(Message::Text(json.into())).await;
                                                    }
                                                    break;
                                                }
                                                _ => {
                                                    debug!("WebRTC session {} current state after answer: {:?}", session_id, current_state);
                                                }
                                            }
                                        }
                                    }
                                }
                                WebRtcSignalingMessage::IceCandidate { session_id: sid, candidate, sdp_mid, sdp_mline_index } => {
                                    if sid == session_id {
                                        if let Err(e) = webrtc_manager.add_ice_candidate(&session_id, &candidate, sdp_mid, sdp_mline_index).await {
                                            warn!("Failed to add ICE candidate: {}", e);
                                        }
                                    }
                                }
                                WebRtcSignalingMessage::Ping => {
                                    let pong = WebRtcSignalingMessage::Pong;
                                    if let Ok(json) = serde_json::to_string(&pong) {
                                        let _ = ws_sender.send(Message::Text(json.into())).await;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    Some(Ok(Message::Ping(data))) => {
                        let _ = ws_sender.send(Message::Pong(data)).await;
                    }
                    Some(Ok(Message::Close(_))) => {
                        info!("WebRTC signaling client disconnected for monitor {}", monitor_id);
                        break;
                    }
                    Some(Err(e)) => {
                        error!("WebRTC signaling error for monitor {}: {}", monitor_id, e);
                        break;
                    }
                    None => break,
                    _ => {}
                }
            }

            // Wait for peer connection state changes (DTLS handshake)
            result = pc_state_rx.changed(), if answer_set && !streaming_started => {
                match result {
                    Ok(()) => {
                        let state = *pc_state_rx.borrow_and_update();
                        match state {
                            webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState::Connected => {
                                streaming_started = true;
                                // Send ready notification now that DTLS is complete
                                let ready_msg = WebRtcSignalingMessage::Ready {
                                    session_id: session_id.clone(),
                                    monitor_id,
                                };
                                if let Ok(json) = serde_json::to_string(&ready_msg) {
                                    let _ = ws_sender.send(Message::Text(json.into())).await;
                                }
                                info!("WebRTC session {} connected (DTLS complete), streaming started", session_id);
                            }
                            webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState::Failed => {
                                error!("WebRTC peer connection failed for session {}", session_id);
                                let error_msg = WebRtcSignalingMessage::Error {
                                    message: "Peer connection failed".to_string(),
                                };
                                if let Ok(json) = serde_json::to_string(&error_msg) {
                                    let _ = ws_sender.send(Message::Text(json.into())).await;
                                }
                                break;
                            }
                            webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState::Disconnected
                            | webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState::Closed => {
                                warn!("WebRTC session {} peer connection state: {:?}", session_id, state);
                                let error_msg = WebRtcSignalingMessage::Error {
                                    message: format!("Peer connection {:?}", state),
                                };
                                if let Ok(json) = serde_json::to_string(&error_msg) {
                                    let _ = ws_sender.send(Message::Text(json.into())).await;
                                }
                                break;
                            }
                            _ => {
                                debug!("WebRTC session {} state: {:?}", session_id, state);
                            }
                        }
                    }
                    Err(_) => {
                        error!("WebRTC peer connection state watch dropped for session {}", session_id);
                        let error_msg = WebRtcSignalingMessage::Error {
                            message: "Peer connection closed unexpectedly".to_string(),
                        };
                        if let Ok(json) = serde_json::to_string(&error_msg) {
                            let _ = ws_sender.send(Message::Text(json.into())).await;
                        }
                        break;
                    }
                }
            }

            // Connection timeout: if answer was set but DTLS never completes
            () = &mut connection_timeout, if answer_set && !streaming_started => {
                error!("WebRTC connection timeout for session {} (30s after SDP answer)", session_id);
                let error_msg = WebRtcSignalingMessage::Error {
                    message: "Connection timeout: DTLS handshake did not complete within 30 seconds".to_string(),
                };
                if let Ok(json) = serde_json::to_string(&error_msg) {
                    let _ = ws_sender.send(Message::Text(json.into())).await;
                }
                break;
            }

            // Handle video packets for streaming (only after DTLS connected)
            result = video_rx.recv(), if streaming_started => {
                match result {
                    Ok(packet) => {
                        // Feed each NAL into the assembler; when a complete
                        // access unit (frame) is ready, send it as one sample
                        // so all NALs share a single RTP timestamp.
                        if let Some(au) = au_assembler.push(&packet) {
                            if let Err(e) = webrtc_manager.write_access_unit(&session_id, &au).await {
                                debug!("Failed to write access unit: {}", e);
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        warn!("WebRTC session {} lagged {} packets", session_id, n);
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        info!("Video source closed for WebRTC session {}", session_id);
                        break;
                    }
                }
            }
        }
    }

    // Cleanup
    let _ = webrtc_manager.remove_session(&session_id).await;
    info!(
        "WebRTC signaling handler finished for monitor {}",
        monitor_id
    );
}

// ============================================================================
// Monitor Snapshot
// ============================================================================

/// Get a JPEG snapshot from a live monitor
#[utoipa::path(
    get,
    path = "/api/v3/monitors/{monitor_id}/snapshot",
    operation_id = "getMonitorSnapshot",
    tag = "Live Streaming",
    params(
        ("monitor_id" = u32, Path, description = "Monitor/Camera ID")
    ),
    responses(
        (status = 200, description = "JPEG snapshot image", content_type = "image/jpeg"),
        (status = 404, description = "Monitor not found", body = AppResponseError),
        (status = 503, description = "Service unavailable", body = AppResponseError)
    )
)]
pub async fn get_monitor_snapshot(
    State(state): State<AppState>,
    Path(monitor_id): Path<u32>,
) -> AppResult<Response> {
    let service = state.snapshot_service.as_ref().ok_or_else(|| {
        AppError::ServiceUnavailableError("Snapshot service not configured".to_string())
    })?;

    let jpeg = service.get_snapshot(monitor_id).await.map_err(|e| {
        use crate::streaming::snapshot::SnapshotError;
        match e {
            SnapshotError::SourceNotAvailable(id) | SnapshotError::KeyframeTimeout(id) => {
                AppError::NotFoundError(crate::error::Resource {
                    resource_type: crate::error::ResourceType::Monitor,
                    details: vec![
                        ("monitor_id".to_string(), id.to_string()),
                        ("reason".to_string(), e.to_string()),
                    ],
                })
            }
            _ => AppError::InternalServerError(e.to_string()),
        }
    })?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "image/jpeg")
        .header(header::CACHE_CONTROL, "no-cache, no-store")
        .body(Body::from(jpeg))
        .unwrap())
}

// ============================================================================
// Source Statistics
// ============================================================================

/// Get source router statistics
#[utoipa::path(
    get,
    path = "/api/v3/live/sources",
    operation_id = "getLiveSources",
    tag = "Live Streaming",
    responses(
        (status = 200, description = "Source statistics"),
        (status = 503, description = "Service unavailable", body = AppResponseError)
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn get_live_sources(State(state): State<AppState>) -> AppResult<Json<serde_json::Value>> {
    let coordinator = state.live_coordinator.as_ref().ok_or_else(|| {
        AppError::ServiceUnavailableError("Live streaming not configured".to_string())
    })?;

    let source_router = coordinator.source_router();
    let stats = source_router.stats().await;

    Ok(Json(serde_json::json!({
        "active_sources": source_router.active_source_count(),
        "sources": stats
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_segment_sequence() {
        assert_eq!(parse_segment_sequence("segment_00001.m4s"), Some(1));
        assert_eq!(parse_segment_sequence("segment_00100.m4s"), Some(100));
        assert_eq!(parse_segment_sequence("segment_00001.0.m4s"), Some(1));
        assert_eq!(parse_segment_sequence("invalid.m4s"), None);
    }

    #[test]
    fn test_start_live_request_default() {
        let request = StartLiveRequest::default();
        assert!(request.enable_hls);
        assert!(!request.enable_webrtc);
        assert!(!request.enable_mse);
    }
}
