use axum::{
    extract::{ws::WebSocketUpgrade, Query, State, Path},
    response::Response,
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use serde_json::Value;
use tracing::info;

use crate::server::state::AppState;
use crate::webrtc_ffi::CameraStream;

/// Query parameters for WebSocket connection
#[derive(Debug, Deserialize, ToSchema)]
pub struct WebSocketQuery {
    pub client_id: Option<String>,
    pub stream_id: Option<String>,
}

/// WebRTC stats response
#[derive(Debug, Serialize)]
pub struct StatsResponse {
    pub stats: std::collections::HashMap<String, Value>,
}

/// Camera streams response
#[derive(Debug, Serialize, ToSchema)]
pub struct StreamsResponse {
    pub streams: Vec<CameraStream>,
}

/// Service status response
#[derive(Debug, Serialize)]
pub struct ServiceStatusResponse {
    pub status: String,
    pub discovered_streams: i32,
    pub active_streams: i32,
    pub connected_clients: i32,
    pub service_info: String,
}

#[utoipa::path(
    get,
    path = "/api/v3/webrtc/ws",
    params(
        ("client_id" = Option<String>, Query, description = "Optional client identifier"),
        ("stream_id" = Option<String>, Query, description = "Optional stream identifier")
    ),
    responses(
        (status = 200, description = "WebSocket upgrade for signaling"),
        (status = 401, description = "Unauthorized", body = crate::error::AppResponseError)
    ),
    security(("jwt" = [])),
    tag = "Streaming"
)]
/// WebSocket upgrade handler for WebRTC signaling
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    Query(_params): Query<WebSocketQuery>,
    State(_state): State<AppState>,
) -> Response {
    info!("WebSocket upgrade request for /api/v3/webrtc/ws");
    ws.on_upgrade(move |_socket| async move {
        // No-op handler; signaling is handled via REST endpoints
        info!("WebSocket connected (no-op handler)");
    })
}

#[utoipa::path(
    get,
    path = "/api/v3/webrtc/stats",
    responses(
        (status = 200, description = "WebRTC service statistics", body = serde_json::Value),
        (status = 401, description = "Unauthorized", body = crate::error::AppResponseError)
    ),
    security(("jwt" = [])),
    tag = "Streaming"
)]
/// Get WebRTC service statistics including camera streams
pub async fn get_stats(
    State(state): State<AppState>,
) -> Json<Value> {
    let connected_clients = state.webrtc_client.sessions().session_count().await as i32;
    let healthy = state.webrtc_client.test_connection().await.is_ok();
    Json(serde_json::json!({
        "webrtc": {
            "connected_clients": connected_clients,
            "active_monitors": 0,
            "total_streams": 0,
            "service_status": if healthy { "connected" } else { "unavailable" }
        },
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

#[utoipa::path(
    get,
    path = "/api/v3/webrtc/streams",
    responses(
        (status = 200, description = "Available camera streams", body = StreamsResponse),
        (status = 401, description = "Unauthorized", body = crate::error::AppResponseError)
    ),
    security(("jwt" = [])),
    tag = "Streaming"
)]
/// Discover and return available camera streams
pub async fn get_camera_streams(
    State(state): State<AppState>,
) -> Json<StreamsResponse> {
    let _ = state; // discovery not implemented in current client
    Json(StreamsResponse { streams: Vec::new() })
}

#[utoipa::path(
    get,
    path = "/api/v3/webrtc/status",
    responses(
        (status = 200, description = "Detailed service status", body = serde_json::Value),
        (status = 401, description = "Unauthorized", body = crate::error::AppResponseError)
    ),
    security(("jwt" = [])),
    tag = "Streaming"
)]
/// Get detailed service status
pub async fn get_service_status(
    State(state): State<AppState>,
) -> Json<ServiceStatusResponse> {
    let healthy = state.webrtc_client.test_connection().await.is_ok();
    Json(ServiceStatusResponse {
        status: if healthy { "connected".to_string() } else { "unavailable".to_string() },
        discovered_streams: 0,
        active_streams: 0,
        connected_clients: state.webrtc_client.sessions().session_count().await as i32,
        service_info: "WebRTC signaling via plugin".to_string(),
    })
}

#[utoipa::path(
    get,
    path = "/api/v3/webrtc/monitors/{id}",
    params(("id" = i32, Path, description = "Monitor ID")),
    responses(
        (status = 200, description = "Monitor info", body = serde_json::Value),
        (status = 401, description = "Unauthorized", body = crate::error::AppResponseError),
        (status = 404, description = "Monitor not found", body = crate::error::AppResponseError)
    ),
    security(("jwt" = [])),
    tag = "Streaming"
)]
/// Get monitor information (legacy compatibility)
pub async fn get_monitor_info(
    Path(monitor_id): Path<i32>,
    State(state): State<AppState>,
) -> Json<Value> {
    let _ = state;
    Json(serde_json::json!({
        "error": "Monitor info not available",
        "monitor_id": monitor_id
    }))
}

#[utoipa::path(
    get,
    path = "/api/v3/webrtc/health",
    responses(
        (status = 200, description = "Service health", body = serde_json::Value)
    ),
    tag = "Streaming"
)]
/// Health check endpoint
pub async fn health_check() -> Json<Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "webrtc",
        "version": "3.0.0",
        "description": "WebRTC service with dynamic FFI and centralized stream discovery",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

#[utoipa::path(
    get,
    path = "/api/v3/webrtc/available-streams",
    responses(
        (status = 200, description = "Available streams", body = serde_json::Value),
        (status = 401, description = "Unauthorized", body = crate::error::AppResponseError)
    ),
    security(("jwt" = [])),
    tag = "Streaming"
)]
/// Get available streams (combines monitor info with camera streams)
pub async fn get_available_streams(
    State(state): State<AppState>,
) -> Json<Value> {
    let _ = state;
    Json(serde_json::json!({
        "streams": [],
        "count": 0
    }))
}
