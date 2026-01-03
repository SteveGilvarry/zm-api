use axum::{
    extract::{ws::WebSocket, Path, Query, State, WebSocketUpgrade},
    http::{header::CONTENT_TYPE, StatusCode},
    response::Response,
    Json,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::broadcast;
use tracing::{error, info, warn};
use utoipa::ToSchema;

use crate::{error::AppResult, mse_client::MseStats, server::state::AppState};

/// Query parameters for segment requests
#[derive(Debug, Deserialize)]
pub struct SegmentQuery {
    pub sequence: Option<u64>,
    pub from_sequence: Option<u64>,
    pub format: Option<String>, // "fmp4", "mp4", etc.
}

/// Response for stream info
#[derive(Debug, Serialize, ToSchema)]
pub struct StreamInfo {
    pub camera_id: u32,
    pub current_sequence: u64,
    pub segment_count: usize,
    pub has_init_segment: bool,
    pub stats: MseStats,
}

/// Response for available streams
#[derive(Debug, Serialize, ToSchema)]
pub struct StreamsResponse {
    pub streams: Vec<StreamInfo>,
    pub total_count: usize,
}

/// WebSocket message types
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebSocketMessage {
    #[serde(rename = "segment")]
    Segment {
        camera_id: u32,
        sequence: u64,
        timestamp: u64,
        is_init: bool,
        #[serde(with = "base64")]
        data: Vec<u8>,
    },
    #[serde(rename = "stats")]
    Stats { camera_id: u32, stats: MseStats },
    #[serde(rename = "error")]
    Error { message: String },
    #[serde(rename = "ping")]
    Ping,
    #[serde(rename = "pong")]
    Pong,
}

mod base64 {
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

/// Get list of available streams
#[utoipa::path(
    get,
    path = "/api/v3/mse/streams",
    responses(
        (status = 200, description = "List available MSE streams", body = StreamsResponse),
        (status = 401, description = "Unauthorized", body = crate::error::AppResponseError)
    ),
    security(("jwt" = [])),
    tag = "MSE"
)]
pub async fn get_streams(State(state): State<AppState>) -> AppResult<Json<StreamsResponse>> {
    let manager = state.mse_manager();
    let camera_ids = manager.get_active_cameras().await;

    let mut streams = Vec::new();
    for camera_id in &camera_ids {
        if let Some(client) = manager.get_client(*camera_id).await {
            let segment_manager = client.segment_manager();
            let segment_manager = segment_manager.lock().unwrap();

            let stream_info = StreamInfo {
                camera_id: *camera_id,
                current_sequence: segment_manager.current_sequence(),
                segment_count: segment_manager.segment_count(),
                has_init_segment: segment_manager.get_init_segment().is_some(),
                stats: client.get_stats(),
            };
            streams.push(stream_info);
        }
    }

    Ok(Json(StreamsResponse {
        total_count: streams.len(),
        streams,
    }))
}

/// Get stream information for a specific camera
#[utoipa::path(
    get,
    path = "/api/v3/mse/streams/{camera_id}",
    params(("camera_id" = u32, Path, description = "Camera ID")),
    responses(
        (status = 200, description = "MSE stream info", body = StreamInfo),
        (status = 401, description = "Unauthorized", body = crate::error::AppResponseError),
        (status = 404, description = "Stream not found", body = crate::error::AppResponseError)
    ),
    security(("jwt" = [])),
    tag = "MSE"
)]
pub async fn get_stream_info(
    State(state): State<AppState>,
    Path(camera_id): Path<u32>,
) -> AppResult<Json<StreamInfo>> {
    let manager = state.mse_manager();

    let client = manager
        .get_client(camera_id)
        .await
        .ok_or_else(|| anyhow::anyhow!("Stream not found for camera {}", camera_id))?;

    let segment_manager = client.segment_manager();
    let segment_manager = segment_manager.lock().unwrap();

    let stream_info = StreamInfo {
        camera_id,
        current_sequence: segment_manager.current_sequence(),
        segment_count: segment_manager.segment_count(),
        has_init_segment: segment_manager.get_init_segment().is_some(),
        stats: client.get_stats(),
    };

    Ok(Json(stream_info))
}

/// Get initialization segment for a camera
#[utoipa::path(
    get,
    path = "/api/v3/mse/streams/{camera_id}/init.mp4",
    params(("camera_id" = u32, Path, description = "Camera ID")),
    responses(
        (status = 200, description = "fMP4 init segment (video/mp4)"),
        (status = 401, description = "Unauthorized", body = crate::error::AppResponseError),
        (status = 404, description = "No init segment available", body = crate::error::AppResponseError)
    ),
    security(("jwt" = [])),
    tag = "MSE"
)]
pub async fn get_init_segment(
    State(state): State<AppState>,
    Path(camera_id): Path<u32>,
) -> AppResult<Response> {
    let manager = state.mse_manager();

    // Try to get existing client or create new one with default resolution
    let client = match manager.get_client(camera_id).await {
        Some(client) => client,
        None => {
            // Create new client with default resolution (1920x1080)
            // In production, you'd get this from the database or config
            manager
                .get_or_create_client(camera_id, 1920, 1080)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to create client: {}", e))?
        }
    };

    // First try to get init segment directly from the plugin
    if let Ok(Some(init_data)) = client.get_init_segment_from_plugin() {
        info!(
            "Got initialization segment from plugin for camera {}: {} bytes",
            camera_id,
            init_data.len()
        );
        return Ok(Response::builder()
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, "video/mp4")
            .header("Cache-Control", "no-cache")
            .body(init_data.into())
            .unwrap());
    }

    // Fallback to cached segment manager
    let segment_manager = client.segment_manager();
    let segment_manager = segment_manager.lock().unwrap();

    let init_segment = segment_manager.get_init_segment().ok_or_else(|| {
        anyhow::anyhow!(
            "No initialization segment available for camera {}",
            camera_id
        )
    })?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "video/mp4")
        .header("Cache-Control", "no-cache")
        .body(init_segment.data.clone().into())
        .unwrap())
}

/// Get a specific segment
#[utoipa::path(
    get,
    path = "/api/v3/mse/streams/{camera_id}/segments/{sequence}",
    params(
        ("camera_id" = u32, Path, description = "Camera ID"),
        ("sequence" = u64, Path, description = "Segment sequence number")
    ),
    responses(
        (status = 200, description = "fMP4 segment (video/mp4)"),
        (status = 401, description = "Unauthorized", body = crate::error::AppResponseError),
        (status = 404, description = "Segment not found", body = crate::error::AppResponseError)
    ),
    security(("jwt" = [])),
    tag = "MSE"
)]
pub async fn get_segment(
    State(state): State<AppState>,
    Path((camera_id, sequence)): Path<(u32, u64)>,
) -> AppResult<Response> {
    let manager = state.mse_manager();

    let client = manager
        .get_client(camera_id)
        .await
        .ok_or_else(|| anyhow::anyhow!("Stream not found for camera {}", camera_id))?;

    let segment_manager = client.segment_manager();
    let segment_manager = segment_manager.lock().unwrap();

    let segment = segment_manager.get_segment(sequence).ok_or_else(|| {
        anyhow::anyhow!("Segment {} not found for camera {}", sequence, camera_id)
    })?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "video/mp4")
        .header("Cache-Control", "no-cache")
        .body(segment.data.clone().into())
        .unwrap())
}

/// Get the latest segment
#[utoipa::path(
    get,
    path = "/api/v3/mse/streams/{camera_id}/segments/latest.mp4",
    params(("camera_id" = u32, Path, description = "Camera ID")),
    responses(
        (status = 200, description = "Latest fMP4 segment (video/mp4)"),
        (status = 401, description = "Unauthorized", body = crate::error::AppResponseError),
        (status = 404, description = "No segments available", body = crate::error::AppResponseError)
    ),
    security(("jwt" = [])),
    tag = "MSE"
)]
pub async fn get_latest_segment(
    State(state): State<AppState>,
    Path(camera_id): Path<u32>,
) -> AppResult<Response> {
    let manager = state.mse_manager();

    let client = manager
        .get_client(camera_id)
        .await
        .ok_or_else(|| anyhow::anyhow!("Stream not found for camera {}", camera_id))?;

    // First try to get latest segment directly from the plugin
    if let Ok(Some(segment_data)) = client.get_latest_segment_from_plugin() {
        info!(
            "Got latest segment from plugin for camera {}: {} bytes",
            camera_id,
            segment_data.len()
        );
        return Ok(Response::builder()
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, "video/mp4")
            .header("Cache-Control", "no-cache")
            .body(segment_data.into())
            .unwrap());
    }

    // Fallback to cached segment manager
    let segment_manager = client.segment_manager();
    let segment_manager = segment_manager.lock().unwrap();

    let segment = segment_manager
        .get_latest_segment()
        .ok_or_else(|| anyhow::anyhow!("No segments available for camera {}", camera_id))?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "video/mp4")
        .header("Cache-Control", "no-cache")
        .header("X-Segment-Sequence", segment.sequence.to_string())
        .header("X-Segment-Timestamp", segment.timestamp.to_string())
        .body(segment.data.clone().into())
        .unwrap())
}

/// Get multiple segments from a starting sequence
#[utoipa::path(
    get,
    path = "/api/v3/mse/streams/{camera_id}/segments/from/{start_sequence}",
    params(
        ("camera_id" = u32, Path, description = "Camera ID"),
        ("start_sequence" = u64, Path, description = "Starting sequence number")
    ),
    responses(
        (status = 200, description = "Segment index from starting sequence", body = serde_json::Value),
        (status = 401, description = "Unauthorized", body = crate::error::AppResponseError)
    ),
    security(("jwt" = [])),
    tag = "MSE"
)]
pub async fn get_segments_from(
    State(state): State<AppState>,
    Path((camera_id, start_sequence)): Path<(u32, u64)>,
) -> AppResult<Json<serde_json::Value>> {
    let manager = state.mse_manager();

    let client = manager
        .get_client(camera_id)
        .await
        .ok_or_else(|| anyhow::anyhow!("Stream not found for camera {}", camera_id))?;

    let segment_manager = client.segment_manager();
    let segment_manager = segment_manager.lock().unwrap();

    let segments = segment_manager.get_segments_from(start_sequence);

    let segment_info: Vec<_> = segments
        .iter()
        .map(|seg| {
            json!({
                "sequence": seg.sequence,
                "timestamp": seg.timestamp,
                "duration_ms": seg.duration_ms,
                "size": seg.data.len(),
                "is_init": seg.is_init
            })
        })
        .collect();

    Ok(Json(json!({
        "camera_id": camera_id,
        "start_sequence": start_sequence,
        "segments": segment_info,
        "count": segments.len()
    })))
}

/// Create a new stream for a camera
#[utoipa::path(
    post,
    path = "/api/v3/mse/streams/{camera_id}",
    params(("camera_id" = u32, Path, description = "Camera ID")),
    responses(
        (status = 200, description = "Stream created or already exists", body = StreamInfo),
        (status = 401, description = "Unauthorized", body = crate::error::AppResponseError)
    ),
    security(("jwt" = [])),
    tag = "MSE"
)]
pub async fn create_stream(
    State(state): State<AppState>,
    Path(camera_id): Path<u32>,
    Query(query): Query<std::collections::HashMap<String, String>>,
) -> AppResult<Json<StreamInfo>> {
    let manager = state.mse_manager();

    // Parse width and height from query parameters
    let width = query
        .get("width")
        .and_then(|w| w.parse().ok())
        .unwrap_or(1920);
    let height = query
        .get("height")
        .and_then(|h| h.parse().ok())
        .unwrap_or(1080);

    // First check if client already exists
    if let Some(existing_client) = manager.get_client(camera_id).await {
        let segment_manager = existing_client.segment_manager();
        let stream_info = {
            let segment_manager = segment_manager.lock().unwrap();
            StreamInfo {
                camera_id,
                current_sequence: segment_manager.current_sequence(),
                segment_count: segment_manager.segment_count(),
                has_init_segment: segment_manager.get_init_segment().is_some(),
                stats: existing_client.get_stats(),
            }
        };

        info!(
            "Using existing MSE stream for camera {} ({}x{})",
            camera_id, width, height
        );
        return Ok(Json(stream_info));
    }

    // Create new client directly without timeout for now
    let client = manager
        .get_or_create_client(camera_id, width, height)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create stream: {}", e))?;

    let segment_manager = client.segment_manager();

    let stream_info = {
        let segment_manager = segment_manager.lock().unwrap();

        StreamInfo {
            camera_id,
            current_sequence: segment_manager.current_sequence(),
            segment_count: segment_manager.segment_count(),
            has_init_segment: segment_manager.get_init_segment().is_some(),
            stats: client.get_stats(),
        }
    };

    info!(
        "Created new MSE stream for camera {} ({}x{})",
        camera_id, width, height
    );

    Ok(Json(stream_info))
}

/// Remove a stream
#[utoipa::path(
    delete,
    path = "/api/v3/mse/streams/{camera_id}",
    params(("camera_id" = u32, Path, description = "Camera ID")),
    responses(
        (status = 200, description = "Stream removed", body = serde_json::Value),
        (status = 401, description = "Unauthorized", body = crate::error::AppResponseError)
    ),
    security(("jwt" = [])),
    tag = "MSE"
)]
pub async fn delete_stream(
    State(state): State<AppState>,
    Path(camera_id): Path<u32>,
) -> AppResult<Json<serde_json::Value>> {
    let manager = state.mse_manager();

    manager.remove_client(camera_id).await;

    Ok(Json(json!({
        "camera_id": camera_id,
        "status": "removed"
    })))
}

/// WebSocket handler for live streaming
#[utoipa::path(
    get,
    path = "/api/v3/mse/streams/{camera_id}/live",
    params(("camera_id" = u32, Path, description = "Camera ID")),
    responses(
        (status = 200, description = "WebSocket upgraded for live MSE streaming"),
        (status = 401, description = "Unauthorized", body = crate::error::AppResponseError)
    ),
    security(("jwt" = [])),
    tag = "MSE"
)]
pub async fn websocket_handler(
    State(state): State<AppState>,
    Path(camera_id): Path<u32>,
    ws: WebSocketUpgrade,
) -> Response {
    ws.on_upgrade(move |socket| handle_websocket(state, camera_id, socket))
}

async fn handle_websocket(state: AppState, camera_id: u32, socket: WebSocket) {
    let manager = state.mse_manager();

    // Get or create client
    let client = match manager.get_or_create_client(camera_id, 1920, 1080).await {
        Ok(client) => client,
        Err(e) => {
            error!(
                "Failed to create MSE client for camera {}: {}",
                camera_id, e
            );
            return;
        }
    };

    info!("WebSocket client connected for camera {}", camera_id);

    let (mut sender, mut receiver) = socket.split();
    let mut segment_receiver = client.subscribe();

    // Send initial init segment if available
    let init_segment_data = {
        let segment_manager = client.segment_manager();
        let segment_manager = segment_manager.lock().unwrap();

        segment_manager
            .get_init_segment()
            .map(|init_segment| WebSocketMessage::Segment {
                camera_id,
                sequence: init_segment.sequence,
                timestamp: init_segment.timestamp,
                is_init: true,
                data: init_segment.data.clone(),
            })
    };

    if let Some(message) = init_segment_data {
        if let Ok(json) = serde_json::to_string(&message) {
            if let Err(e) = sender
                .send(axum::extract::ws::Message::Text(json.into()))
                .await
            {
                error!("Failed to send init segment: {}", e);
                return;
            }
        }
    }

    // Handle incoming messages and outgoing segments
    loop {
        tokio::select! {
            // Handle incoming WebSocket messages
            msg = receiver.next() => {
                match msg {
                    Some(Ok(axum::extract::ws::Message::Text(text))) => {
                        if let Ok(WebSocketMessage::Ping) = serde_json::from_str::<WebSocketMessage>(&text) {
                            let pong = WebSocketMessage::Pong;
                            if let Ok(json) = serde_json::to_string(&pong) {
                                if sender.send(axum::extract::ws::Message::Text(json.into())).await.is_err() {
                                    break;
                                }
                            }
                        }
                    }
                    Some(Ok(axum::extract::ws::Message::Close(_))) => {
                        info!("WebSocket client disconnected for camera {}", camera_id);
                        break;
                    }
                    Some(Err(e)) => {
                        error!("WebSocket error for camera {}: {}", camera_id, e);
                        break;
                    }
                    None => break,
                    _ => {}
                }
            }

            // Handle new segments
            segment = segment_receiver.recv() => {
                match segment {
                    Ok(segment) => {
                        let message = WebSocketMessage::Segment {
                            camera_id,
                            sequence: segment.sequence,
                            timestamp: segment.timestamp,
                            is_init: segment.is_init,
                            data: segment.data,
                        };

                        if let Ok(json) = serde_json::to_string(&message) {
                            if sender.send(axum::extract::ws::Message::Text(json.into())).await.is_err() {
                                warn!("Failed to send segment to WebSocket client for camera {}", camera_id);
                                break;
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                        warn!("WebSocket client lagged {} segments for camera {}", skipped, camera_id);
                        // Continue receiving
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        info!("Segment broadcast closed for camera {}", camera_id);
                        break;
                    }
                }
            }
        }
    }

    info!("WebSocket handler finished for camera {}", camera_id);
}

/// Get stream statistics
#[utoipa::path(
    get,
    path = "/api/v3/mse/streams/{camera_id}/stats",
    params(("camera_id" = u32, Path, description = "Camera ID")),
    responses(
        (status = 200, description = "Stream statistics", body = MseStats),
        (status = 401, description = "Unauthorized", body = crate::error::AppResponseError),
        (status = 404, description = "Stream not found", body = crate::error::AppResponseError)
    ),
    security(("jwt" = [])),
    tag = "MSE"
)]
pub async fn get_stream_stats(
    State(state): State<AppState>,
    Path(camera_id): Path<u32>,
) -> AppResult<Json<MseStats>> {
    let manager = state.mse_manager();

    let client = manager
        .get_client(camera_id)
        .await
        .ok_or_else(|| anyhow::anyhow!("Stream not found for camera {}", camera_id))?;

    Ok(Json(client.get_stats()))
}

/// Get statistics for all streams
#[utoipa::path(
    get,
    path = "/api/v3/mse/stats",
    responses(
        (status = 200, description = "All streams statistics", body = serde_json::Value),
        (status = 401, description = "Unauthorized", body = crate::error::AppResponseError)
    ),
    security(("jwt" = [])),
    tag = "MSE"
)]
pub async fn get_all_stats(
    State(state): State<AppState>,
) -> AppResult<Json<std::collections::HashMap<u32, MseStats>>> {
    let manager = state.mse_manager();
    let stats = manager.get_all_stats().await;

    Ok(Json(stats))
}

/// Test endpoint to directly test socket communication with MSE plugin
pub async fn test_pop_segment(Path(camera_id): Path<u32>) -> AppResult<Json<serde_json::Value>> {
    use crate::mse_socket_client::MseSocketClient;

    info!("Testing socket communication for camera {}", camera_id);
    let socket_client = MseSocketClient::new();

    // Test try_pop_segment
    let (segment_size, has_data) = match socket_client.try_pop_segment(camera_id) {
        Ok(Some(data)) => {
            info!(
                "Socket try_pop_segment returned {} bytes for camera {}",
                data.len(),
                camera_id
            );
            (data.len(), true)
        }
        Ok(None) => {
            info!(
                "Socket try_pop_segment returned no data for camera {}",
                camera_id
            );
            (0, false)
        }
        Err(e) => {
            warn!(
                "Socket try_pop_segment error for camera {}: {}",
                camera_id, e
            );
            (0, false)
        }
    };

    // Test buffer size
    let buffer_size = match socket_client.get_buffer_size(camera_id) {
        Ok(size) => {
            info!(
                "Socket get_buffer_size returned {} for camera {}",
                size, camera_id
            );
            size
        }
        Err(e) => {
            warn!(
                "Socket get_buffer_size error for camera {}: {}",
                camera_id, e
            );
            0
        }
    };

    // Test stats
    let (stats_buffer_size, total_segments, dropped_segments, bytes_received, frame_count) =
        match socket_client.get_buffer_stats(camera_id) {
            Ok((buffer_size, total, dropped, bytes, frames)) => {
                info!("Socket get_buffer_stats for camera {}: buffer_size={}, total={}, dropped={}, bytes={}, frames={}", 
                      camera_id, buffer_size, total, dropped, bytes, frames);
                (buffer_size, total, dropped, bytes, frames)
            }
            Err(e) => {
                warn!(
                    "Socket get_buffer_stats error for camera {}: {}",
                    camera_id, e
                );
                (0, 0, 0, 0, 0)
            }
        };

    Ok(Json(json!({
        "camera_id": camera_id,
        "communication_method": "socket",
        "pop_segment_size": segment_size,
        "buffer_size": buffer_size,
        "stats_buffer_size": stats_buffer_size,
        "total_segments": total_segments,
        "dropped_segments": dropped_segments,
        "bytes_received": bytes_received,
        "frame_count": frame_count,
        "has_data": has_data
    })))
}
