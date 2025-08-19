use axum::{
    extract::{ws::WebSocketUpgrade, Query, State, Path},
    response::Response,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{info, error};

use crate::server::state::AppState;
use crate::webrtc_ffi::CameraStream;

/// Query parameters for WebSocket connection
#[derive(Debug, Deserialize)]
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
#[derive(Debug, Serialize)]
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

/// WebSocket upgrade handler for WebRTC signaling
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<WebSocketQuery>,
    State(state): State<AppState>,
) -> Response {
    info!("WebSocket upgrade request from client: {:?}, stream: {:?}", 
          params.client_id, params.stream_id);
    
    let signaling_service = match &state.webrtc_signaling {
        Some(service) => service.clone(),
        None => {
            error!("WebRTC signaling service not available");
            return ws.on_upgrade(move |_socket| async move {
                error!("WebRTC signaling service not initialized");
            });
        }
    };
    
    ws.on_upgrade(move |socket| async move {
        if let Err(e) = signaling_service.handle_connection(socket).await {
            error!("WebSocket connection error: {}", e);
        }
    })
}

/// Get WebRTC service statistics including camera streams
pub async fn get_stats(
    State(state): State<AppState>,
) -> Json<Value> {
    let webrtc_stats = match &state.webrtc_service {
        Some(service) => service.get_stats().await,
        None => {
            return Json(serde_json::json!({
                "error": "WebRTC service not available",
                "webrtc_service": null,
                "signaling_service": null,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }));
        }
    };
    
    let signaling_stats = match &state.webrtc_signaling {
        Some(service) => {
            let stats = service.get_stats().await;
            serde_json::to_value(stats).unwrap_or(serde_json::json!({"error": "Failed to serialize stats"}))
        }
        None => serde_json::json!({"error": "Signaling service not available"})
    };

    Json(serde_json::json!({
        "webrtc_service": {
            "connected_clients": webrtc_stats.connected_clients,
            "active_monitors": webrtc_stats.active_monitors,
            "total_streams": webrtc_stats.total_streams,
            "service_status": webrtc_stats.service_status
        },
        "signaling_service": signaling_stats,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// Discover and return available camera streams
pub async fn get_camera_streams(
    State(state): State<AppState>,
) -> Json<StreamsResponse> {
    match &state.webrtc_service {
        Some(service) => {
            match service.discover_camera_streams().await {
                Ok(streams) => {
                    info!("Returning {} discovered camera streams", streams.len());
                    Json(StreamsResponse { streams })
                }
                Err(e) => {
                    error!("Failed to discover camera streams: {}", e);
                    Json(StreamsResponse { streams: Vec::new() })
                }
            }
        }
        None => {
            error!("WebRTC service not available");
            Json(StreamsResponse { streams: Vec::new() })
        }
    }
}

/// Get detailed service status
pub async fn get_service_status(
    State(state): State<AppState>,
) -> Json<ServiceStatusResponse> {
    match &state.webrtc_service {
        Some(service) => {
            let streams = service.discover_camera_streams().await.unwrap_or_default();
            let stats = service.get_stats().await;
            
            Json(ServiceStatusResponse {
                status: "connected".to_string(),
                discovered_streams: streams.len() as i32,
                active_streams: streams.iter().filter(|s| s.is_active).count() as i32,
                connected_clients: stats.connected_clients,
                service_info: "Connected to centralized zm-next WebRTC service".to_string(),
            })
        }
        None => {
            Json(ServiceStatusResponse {
                status: "unavailable".to_string(),
                discovered_streams: 0,
                active_streams: 0,
                connected_clients: 0,
                service_info: "WebRTC service not available".to_string(),
            })
        }
    }
}

/// Get monitor information (legacy compatibility)
pub async fn get_monitor_info(
    Path(monitor_id): Path<i32>,
    State(state): State<AppState>,
) -> Json<Value> {
    match &state.webrtc_service {
        Some(service) => {
            match service.get_monitor_info(monitor_id).await {
                Ok(Some(info)) => Json(serde_json::json!(info)),
                Ok(None) => Json(serde_json::json!({
                    "error": "Monitor not found",
                    "monitor_id": monitor_id
                })),
                Err(e) => {
                    error!("Failed to get monitor info: {}", e);
                    Json(serde_json::json!({
                        "error": "Failed to get monitor info",
                        "message": e.to_string()
                    }))
                }
            }
        }
        None => {
            Json(serde_json::json!({
                "error": "WebRTC service not available",
                "monitor_id": monitor_id
            }))
        }
    }
}

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

/// Get available streams (combines monitor info with camera streams)
pub async fn get_available_streams(
    State(state): State<AppState>,
) -> Json<Value> {
    match &state.webrtc_service {
        Some(service) => {
            match service.get_available_streams().await {
                Ok(streams) => Json(serde_json::json!({
                    "streams": streams,
                    "count": streams.len()
                })),
                Err(e) => {
                    error!("Failed to get available streams: {}", e);
                    Json(serde_json::json!({
                        "error": "Failed to get available streams",
                        "message": e.to_string(),
                        "streams": [],
                        "count": 0
                    }))
                }
            }
        }
        None => {
            Json(serde_json::json!({
                "error": "WebRTC service not available",
                "streams": [],
                "count": 0
            }))
        }
    }
}
