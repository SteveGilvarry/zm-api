//! Live streaming HTTP handlers
//!
//! Provides unified endpoints for live streaming via HLS and WebRTC,
//! both backed by the stream-socket source router.

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
use std::time::{Duration, Instant};
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};

use crate::error::{AppError, AppResponseError, AppResult};
use crate::server::state::AppState;
use crate::streaming::live::audio::AacToOpusTranscoder;
use crate::streaming::live::webrtc::{
    AccessUnitAssembler, AssembledAccessUnit, AudioTrackKind, WebRtcLiveConfig, WebRtcLiveManager,
};
use crate::streaming::live::{CoordinatorError, LiveStreamConfig};
use crate::streaming::source::{AudioCodec, CachedKeyframe, MonitorSource, VideoCodec};

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
}

fn default_true() -> bool {
    true
}

impl Default for StartLiveRequest {
    fn default() -> Self {
        Self {
            enable_hls: true,
            enable_webrtc: false,
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
    /// Most-recent WebRTC startup profile (connect→offer→first-RTP), when a
    /// session has run since boot. Lets you confirm cold-vs-warm empirically.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webrtc_startup: Option<WebRtcStartupView>,
}

/// Server-side WebRTC startup timing surfaced on `/live/{id}/stats`. All `*_ms`
/// are from WS connect and nest: `get_source_ms ≤ offer_ms ≤ connected_ms ≤
/// first_rtp_ms`.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct WebRtcStartupView {
    /// Reader was already hot at connect (no reader spin-up on the offer path).
    pub warm_start: bool,
    /// connect → get_source returned (reader acquire/restart cost).
    pub get_source_ms: Option<u64>,
    /// connect → SDP offer sent.
    pub offer_ms: Option<u64>,
    /// connect → peer Connected (ICE+DTLS). `connected_ms − offer_ms` ≈ the DTLS
    /// handshake on a LAN.
    pub connected_ms: Option<u64>,
    /// connect → first video RTP written.
    pub first_rtp_ms: Option<u64>,
}

/// Protocol status in live stats
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct LiveProtocolStatus {
    pub hls: bool,
    pub webrtc: bool,
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
                        (
                            "reason".to_string(),
                            "stream socket not available".to_string(),
                        ),
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

    let webrtc_startup = state
        .source_router
        .as_ref()
        .and_then(|r| r.webrtc_startup(monitor_id))
        .map(|t| WebRtcStartupView {
            warm_start: t.warm_start,
            get_source_ms: t.get_source_ms,
            offer_ms: t.offer_ms,
            connected_ms: t.connected_ms,
            first_rtp_ms: t.first_rtp_ms,
        });

    Ok(Json(LiveStatsResponse {
        monitor_id: stats.monitor_id,
        status: format!("{:?}", stats.status).to_lowercase(),
        packets_processed: stats.packets_processed,
        errors: stats.errors,
        uptime_seconds: stats.uptime_seconds,
        protocols: LiveProtocolStatus {
            hls: stats.hls_enabled,
            webrtc: stats.webrtc_enabled,
        },
        webrtc_startup,
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
    ),
    security(("jwt" = []))
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
    ),
    security(("jwt" = []))
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
    ),
    security(("jwt" = []))
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
    ),
    security(("jwt" = []))
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
// WebRTC Endpoints (WebSocket signaling)
// ============================================================================

/// WebRTC signaling message types.
///
/// This is the *actual* wire format exchanged over the
/// `/api/v3/live/{monitor_id}/webrtc/ws` WebSocket (text frames, JSON). The
/// discriminator is the `type` field, lowercased: `offer`, `answer`,
/// `icecandidate`, `ready`, `error`, `ping`, `pong`.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
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
        /// Whether the session carries an audio track (G.711 pass-through
        /// or AAC transcoded to Opus) alongside the video track
        has_audio: bool,
    },
    /// Error message
    Error {
        message: String,
    },
    /// Ping/Pong for keepalive
    Ping,
    Pong,
}

/// WebSocket handler for WebRTC signaling.
///
/// Upgrades to a WebSocket and exchanges WebRTC signaling over JSON **text
/// frames**. OpenAPI cannot type WebSocket message bodies, so the message
/// schema is documented here and as the `WebRtcSignalingMessage` component.
///
/// ## Message envelope
///
/// Every frame is a JSON object discriminated by a lowercase `type` field
/// (see the `WebRtcSignalingMessage` schema). Note the field casing:
/// `icecandidate` (one word) with camelCase `sdpMid` / `sdpMLineIndex`.
///
/// ## Typical flow (client-initiated, the common path)
///
/// 1. **Server → client** `{"type":"offer","session_id":"<id>","sdp":"..."}`
///    — the server sends its SDP offer immediately on connect.
/// 2. **Client → server** `{"type":"answer","session_id":"<id>","sdp":"..."}`.
/// 3. **Both directions** `{"type":"icecandidate","session_id":"<id>",`
///    `"candidate":"candidate:...","sdpMid":"0","sdpMLineIndex":0}` — trickled
///    as discovered.
/// 4. **Server → client** `{"type":"ready","session_id":"<id>","monitor_id":N,`
///    `"has_audio":false}` once media is flowing. `has_audio` tells the client
///    whether an audio track accompanies the video.
/// 5. **Errors** arrive as `{"type":"error","message":"..."}`.
/// 6. **Keepalive**: either side may send `{"type":"ping"}`; the peer replies
///    `{"type":"pong"}`.
#[utoipa::path(
    get,
    path = "/api/v3/live/{monitor_id}/webrtc/ws",
    operation_id = "webrtcSignalingStream",
    tag = "Live Streaming",
    params(
        ("monitor_id" = u32, Path, description = "Monitor/Camera ID")
    ),
    responses(
        (status = 101, description = "WebSocket upgraded. Frames are JSON `WebRtcSignalingMessage` \
            objects (text). Server sends `offer`/`ready`/`icecandidate`/`error`/`pong`; client \
            sends `answer`/`icecandidate`/`ping`.", body = WebRtcSignalingMessage),
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
                (
                    "reason".to_string(),
                    "stream socket not available".to_string(),
                ),
            ],
        }));
    }

    let router = Arc::clone(source_router);

    // Honor the operator's configured STUN/TURN servers instead of the
    // hardcoded Google STUN defaults (REVIEW_FIXES_PLAN §2.2).
    let webrtc_config = WebRtcLiveConfig::from_webrtc_config(&state.config.streaming.webrtc);

    Ok(ws.on_upgrade(move |socket| {
        handle_webrtc_websocket(router, monitor_id, webrtc_config, socket)
    }))
}

/// Inject a keyframe at DTLS-connected time so the viewer sees a picture
/// immediately instead of waiting a full GOP for the next natural IDR.
///
/// Crucially this re-reads the source's keyframe cache **live**. On a cold
/// start the cache is empty when signaling begins, but the socket reader almost
/// always populates it during the seconds of ICE/DTLS negotiation. The old code
/// injected the snapshot captured *before* signaling (`pre_signaling`), which on
/// a cold start was `None` — throwing away the freshly-cached keyframe and
/// forcing a GOP-length stall (1–8s depending on the camera). We prefer the live
/// read, fall back to the pre-signaling snapshot, then to nothing (the normal
/// access-unit path then waits for the next keyframe). See REVIEW_FIXES_PLAN §2.1.
async fn inject_startup_keyframe(
    webrtc_manager: &WebRtcLiveManager,
    session_id: &str,
    source: &MonitorSource,
    pre_signaling: &Option<CachedKeyframe>,
    au_assembler: &mut AccessUnitAssembler,
) -> bool {
    let Some(ck) = source.cached_keyframe().or_else(|| pre_signaling.clone()) else {
        return false;
    };
    let au = AssembledAccessUnit {
        data: ck.keyframe_au.clone(),
        timestamp_us: ck.timestamp_us,
        is_keyframe: true,
    };
    if let Err(e) = webrtc_manager.write_access_unit(session_id, &au).await {
        warn!("Failed to inject startup keyframe: {}", e);
        return false;
    }
    au_assembler.clear_needs_keyframe();
    true
}

async fn handle_webrtc_websocket(
    source_router: Arc<crate::streaming::source::SourceRouter>,
    monitor_id: u32,
    webrtc_config: WebRtcLiveConfig,
    socket: WebSocket,
) {
    info!(
        "WebRTC signaling WebSocket connected for monitor {}",
        monitor_id
    );

    // t0 for startup instrumentation: connect → offer → first-RTP, surfaced on
    // /live/{id}/stats to confirm cold-vs-warm on real hardware.
    let connect_at = Instant::now();

    // Create WebRTC manager for this session
    let webrtc_manager = WebRtcLiveManager::new(webrtc_config);

    // Whether the reader was already hot — checked *before* get_source (which
    // may (re)start it), so the offer profile distinguishes a fast warm offer
    // from a cold reader spin-up that `warm_start` (keyframe cache) alone missed.
    let warm_start = source_router.is_reader_hot(monitor_id).await;

    // Get source to determine codec. This may start/restart the reader, so it is
    // timed separately — the suspected cold-offer cost.
    let get_source_at = Instant::now();
    let source = match source_router.get_source(monitor_id).await {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to get source for monitor {}: {}", monitor_id, e);
            return;
        }
    };
    let get_source_ms = get_source_at.elapsed().as_millis() as u64;

    // Try the keyframe cache first (warm path: reader already active).
    // If populated, we skip the cold-path codec-detection grace period entirely.
    let cached: Option<CachedKeyframe> = source.cached_keyframe();

    let (codec, profile_level_id) = if let Some(ref ck) = cached {
        info!(
            "Using cached keyframe for monitor {} (profile-level-id={})",
            monitor_id, ck.profile_level_id
        );
        (ck.codec, Some(ck.profile_level_id.clone()))
    } else {
        // Cold path: no cached keyframe yet (first viewer since the reader
        // started). Determine the codec from the source — the socket HELLO sets
        // it as soon as it parses a packet; give it a brief grace period, then
        // default to H.264.
        //
        // We deliberately do NOT wait for an SPS NAL here. The SDP offer
        // advertises fixed, browser-universal H.264 profiles regardless of the
        // camera's real profile-level-id (see `h264_offer_profile_level_ids`),
        // so the detected value only ever reached a debug log — the previous
        // up-to-5s SPS scan added that much startup latency for nothing.
        // See REVIEW_FIXES_PLAN §2.3.
        debug!(
            "No keyframe cache for monitor {}, detecting codec from source",
            monitor_id
        );

        tokio::time::sleep(Duration::from_millis(100)).await;
        let codec = match source.codec().await {
            VideoCodec::Unknown => VideoCodec::H264,
            known => known,
        };

        (codec, None)
    };

    // Decide the audio track for the offer. G.711 passes through unchanged
    // (a mandatory WebRTC codec); AAC is transcoded to Opus — browsers do
    // not implement AAC over RTP. The transcoder is created up front so a
    // missing libopus downgrades the offer to video-only instead of
    // advertising an audio m-line that could never carry media.
    let mut audio_transcoder: Option<AacToOpusTranscoder> = None;
    let audio_kind = match source.audio_codec() {
        Some(AudioCodec::G711Ulaw) => Some(AudioTrackKind::Pcmu),
        Some(AudioCodec::G711Alaw) => Some(AudioTrackKind::Pcma),
        Some(AudioCodec::Aac) => match AacToOpusTranscoder::new() {
            Ok(t) => {
                audio_transcoder = Some(t);
                Some(AudioTrackKind::Opus)
            }
            Err(e) => {
                warn!(
                    "Monitor {} has AAC audio but transcoding is unavailable ({}); \
                     offering video-only",
                    monitor_id, e
                );
                None
            }
        },
        Some(other) => {
            debug!(
                "Monitor {} audio codec {:?} not supported over WebRTC; video-only",
                monitor_id, other
            );
            None
        }
        None => None,
    };

    // Create WebRTC session with offer and state watch channel
    let (session_id, offer, mut pc_state_rx) = match webrtc_manager
        .create_session(monitor_id, codec, profile_level_id.as_deref(), audio_kind)
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
    // Record the offer profile: warm flag, the get_source (reader-acquire) cost,
    // and connect → offer-sent.
    source_router.record_webrtc_offer(
        monitor_id,
        warm_start,
        get_source_ms,
        connect_at.elapsed().as_millis() as u64,
    );

    // Subscribe to video packets for streaming
    let mut video_rx = source.subscribe_video();

    // Subscribe to audio when the offer carries an audio track
    let mut audio_rx = audio_kind.is_some().then(|| source.subscribe_audio());

    // Watch the stream-socket reader's health. If the reader task exits, the watch
    // sender is dropped and `changed()` returns Err — the broadcast channel
    // would then never yield another packet (and never close, since the
    // MonitorSource keeps the Sender alive), so the session must end rather
    // than hang silently.
    let mut reader_health_rx = source.subscribe_reader_health();

    // Assembles individual NAL units into complete access units (frames)
    // so the H264 payloader assigns one RTP timestamp per picture.
    let mut au_assembler = AccessUnitAssembler::new();

    // Track if DTLS handshake is complete and we can stream media
    let mut streaming_started = false;
    // Track if cached keyframe has been injected
    let mut cache_injected = false;
    // Track if the connect → first-video-RTP timing has been recorded.
    let mut first_rtp_recorded = false;
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
                                WebRtcSignalingMessage::Answer { session_id: sid, sdp }
                                    if sid == session_id => {
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
                                                    source_router.record_webrtc_connected(monitor_id, connect_at.elapsed().as_millis() as u64);
                                                    // Inject the freshest keyframe for instant video start
                                                    if !cache_injected {
                                                        cache_injected = inject_startup_keyframe(&webrtc_manager, &session_id, &source, &cached, &mut au_assembler).await;
                                                        if cache_injected {
                                                            info!("Injected startup keyframe for session {} (fast start)", session_id);
                                                        }
                                                    }
                                                    let ready_msg = WebRtcSignalingMessage::Ready {
                                                        session_id: session_id.clone(),
                                                        monitor_id,
                                                        has_audio: audio_kind.is_some(),
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
                                WebRtcSignalingMessage::IceCandidate { session_id: sid, candidate, sdp_mid, sdp_mline_index }
                                    if sid == session_id => {
                                        if let Err(e) = webrtc_manager.add_ice_candidate(&session_id, &candidate, sdp_mid, sdp_mline_index).await {
                                            warn!("Failed to add ICE candidate: {}", e);
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
                                source_router.record_webrtc_connected(monitor_id, connect_at.elapsed().as_millis() as u64);
                                // Inject the freshest keyframe for instant video start
                                if !cache_injected {
                                    cache_injected = inject_startup_keyframe(&webrtc_manager, &session_id, &source, &cached, &mut au_assembler).await;
                                    if cache_injected {
                                        info!("Injected startup keyframe for session {} (fast start)", session_id);
                                    }
                                }
                                // Send ready notification now that DTLS is complete
                                let ready_msg = WebRtcSignalingMessage::Ready {
                                    session_id: session_id.clone(),
                                    monitor_id,
                                    has_audio: audio_kind.is_some(),
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
                            } else if !first_rtp_recorded {
                                first_rtp_recorded = true;
                                source_router.record_webrtc_first_rtp(
                                    monitor_id,
                                    connect_at.elapsed().as_millis() as u64,
                                );
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

            // Handle audio packets. The branch pends forever for video-only
            // sessions; packets arriving before DTLS completes are dropped
            // (the receiver must keep draining so the broadcast channel
            // doesn't lag).
            result = async {
                match audio_rx.as_mut() {
                    Some(rx) => rx.recv().await,
                    None => std::future::pending().await,
                }
            } => {
                match result {
                    Ok(packet) if streaming_started => match packet.codec {
                        AudioCodec::G711Alaw | AudioCodec::G711Ulaw => {
                            // G.711: 8 kHz, one byte per sample — pass through.
                            let duration = Duration::from_micros(
                                packet.data.len() as u64 * 1_000_000 / 8_000,
                            );
                            if let Err(e) = webrtc_manager
                                .write_audio_sample(&session_id, &packet.data, duration)
                                .await
                            {
                                debug!("Failed to write audio sample: {}", e);
                            }
                        }
                        AudioCodec::Aac => {
                            // ADTS AAC → Opus. Sub-millisecond per frame, so
                            // inline in the async loop is fine.
                            if let Some(transcoder) = audio_transcoder.as_mut() {
                                match transcoder.transcode(&packet.data) {
                                    Ok(frames) => {
                                        for frame in frames {
                                            if let Err(e) = webrtc_manager
                                                .write_audio_sample(
                                                    &session_id,
                                                    &frame.data,
                                                    frame.duration,
                                                )
                                                .await
                                            {
                                                debug!("Failed to write Opus frame: {}", e);
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        debug!(
                                            "AAC→Opus transcode error for session {}: {}",
                                            session_id, e
                                        );
                                    }
                                }
                            }
                        }
                        _ => {}
                    },
                    Ok(_) => {} // pre-DTLS audio: drained and dropped
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        debug!("WebRTC session {} audio lagged {} packets", session_id, n);
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        info!("Audio source closed for WebRTC session {}", session_id);
                        audio_rx = None;
                    }
                }
            }

            // Reader health: detect a dead socket reader so the session ends
            // instead of waiting forever on a broadcast channel that never
            // closes. A `Reconnecting` transition is transient (the reader
            // retries on its own) and is only logged; `Err` means the reader
            // task is gone for good.
            result = reader_health_rx.changed() => {
                match result {
                    Ok(()) => {
                        let health = *reader_health_rx.borrow_and_update();
                        debug!(
                            "WebRTC session {} reader health for monitor {}: {:?}",
                            session_id, monitor_id, health
                        );
                    }
                    Err(_) => {
                        error!(
                            "Reader task exited for monitor {}, ending WebRTC session {}",
                            monitor_id, session_id
                        );
                        let error_msg = WebRtcSignalingMessage::Error {
                            message: "Video source stopped".to_string(),
                        };
                        if let Ok(json) = serde_json::to_string(&error_msg) {
                            let _ = ws_sender.send(Message::Text(json.into())).await;
                        }
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
    ),
    security(("jwt" = []))
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

    /// Plain numeric sequences with zero-padding are parsed to their integer value.
    #[test]
    fn parse_segment_sequence_valid_numbers() {
        assert_eq!(parse_segment_sequence("segment_0.m4s"), Some(0));
        assert_eq!(parse_segment_sequence("segment_00000.m4s"), Some(0));
        assert_eq!(parse_segment_sequence("segment_42.m4s"), Some(42));
        assert_eq!(parse_segment_sequence("segment_999999.m4s"), Some(999_999));
        // Maximum u64 still parses.
        assert_eq!(
            parse_segment_sequence("segment_18446744073709551615.m4s"),
            Some(u64::MAX)
        );
    }

    /// The leading `.`-delimited token before the suffix is what gets parsed,
    /// so a `segment_00001.0.m4s` style name yields the first component.
    #[test]
    fn parse_segment_sequence_accepts_dotted_part_suffix() {
        assert_eq!(parse_segment_sequence("segment_7.0.m4s"), Some(7));
        assert_eq!(parse_segment_sequence("segment_12.999.m4s"), Some(12));
    }

    /// Extra underscore-delimited tokens are allowed as long as `parts[0]` is
    /// `segment` and `parts[1]` is numeric (`parts.len() >= 2`).
    #[test]
    fn parse_segment_sequence_accepts_extra_underscore_tokens() {
        assert_eq!(parse_segment_sequence("segment_5_extra.m4s"), Some(5));
        assert_eq!(parse_segment_sequence("segment_5_a_b_c.m4s"), Some(5));
    }

    /// Inputs that do not match the `segment_<num>.m4s` shape return `None`
    /// rather than panicking.
    #[test]
    fn parse_segment_sequence_rejects_garbage() {
        // Wrong / missing suffix.
        assert_eq!(parse_segment_sequence("segment_00001"), None);
        assert_eq!(parse_segment_sequence("segment_00001.ts"), None);
        assert_eq!(parse_segment_sequence("segment_00001.mp4"), None);
        // Wrong prefix.
        assert_eq!(parse_segment_sequence("seg_00001.m4s"), None);
        assert_eq!(parse_segment_sequence("init_00001.m4s"), None);
        assert_eq!(parse_segment_sequence("Segment_00001.m4s"), None);
        // No underscore -> only one part.
        assert_eq!(parse_segment_sequence("segment.m4s"), None);
        // Non-numeric sequence component.
        assert_eq!(parse_segment_sequence("segment_abc.m4s"), None);
        assert_eq!(parse_segment_sequence("segment_-1.m4s"), None);
        assert_eq!(parse_segment_sequence("segment_1a.m4s"), None);
        // Empty sequence component.
        assert_eq!(parse_segment_sequence("segment_.m4s"), None);
        // Overflows u64 -> parse fails gracefully.
        assert_eq!(
            parse_segment_sequence("segment_99999999999999999999999.m4s"),
            None
        );
        // Empty / suffix-only input.
        assert_eq!(parse_segment_sequence(""), None);
        assert_eq!(parse_segment_sequence(".m4s"), None);
        assert_eq!(parse_segment_sequence("..m4s"), None);
    }

    /// `default_true` underpins the `enable_hls` serde default.
    #[test]
    fn default_true_is_true() {
        assert!(default_true());
    }

    #[test]
    fn test_start_live_request_default() {
        let request = StartLiveRequest::default();
        assert!(request.enable_hls);
        assert!(!request.enable_webrtc);
    }

    /// An empty JSON object applies the serde defaults: HLS on, WebRTC off.
    #[test]
    fn start_live_request_deserializes_empty_object() {
        let request: StartLiveRequest = serde_json::from_str("{}").expect("deserialize");
        assert!(request.enable_hls, "enable_hls defaults to true");
        assert!(!request.enable_webrtc);
    }

    /// Explicit fields override the defaults.
    #[test]
    fn start_live_request_deserializes_explicit_fields() {
        let request: StartLiveRequest =
            serde_json::from_str(r#"{"enable_hls": false, "enable_webrtc": true}"#)
                .expect("deserialize");
        assert!(!request.enable_hls);
        assert!(request.enable_webrtc);
    }

    /// The WebRTC signaling enum round-trips, including the renamed ICE fields.
    #[test]
    fn webrtc_signaling_message_round_trips() {
        let ice = WebRtcSignalingMessage::IceCandidate {
            session_id: "sess-1".to_string(),
            candidate: "candidate:0 1 UDP".to_string(),
            sdp_mid: Some("0".to_string()),
            sdp_mline_index: Some(0),
        };
        let json = serde_json::to_string(&ice).expect("serialize");
        assert!(json.contains("\"sdpMid\""));
        assert!(json.contains("\"sdpMLineIndex\""));
        match serde_json::from_str::<WebRtcSignalingMessage>(&json).expect("deserialize") {
            WebRtcSignalingMessage::IceCandidate {
                session_id,
                sdp_mline_index,
                ..
            } => {
                assert_eq!(session_id, "sess-1");
                assert_eq!(sdp_mline_index, Some(0));
            }
            other => panic!("expected IceCandidate, got {other:?}"),
        }
    }
}
