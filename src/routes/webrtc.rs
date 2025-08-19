use axum::{
    Router,
    routing::get,
};
use tower_http::services::ServeDir;
use crate::handlers::webrtc::{
    websocket_handler,
    get_stats,
    health_check,
    get_camera_streams,
    get_service_status,
    get_monitor_info,
    get_available_streams,
};
use crate::server::state::AppState;

/// Add WebRTC routes to the router
/// 
/// This module sets up the WebRTC endpoints for the new centralized service:
/// - WebSocket /ws/webrtc - WebSocket endpoint for signaling
/// - GET /api/webrtc/stats - Get comprehensive WebRTC service statistics  
/// - GET /api/webrtc/health - Health check endpoint
/// - GET /api/webrtc/streams - Discover available camera streams
/// - GET /api/webrtc/status - Get detailed service status
/// - GET /api/webrtc/monitors/{id} - Get monitor information (legacy)
/// - GET /api/webrtc/available-streams - Get all available streams
/// - Serve static files from /static for testing
pub fn add_webrtc_routes() -> Router<AppState> {
    Router::new()
        .route("/ws/webrtc", get(websocket_handler))
        .route("/api/webrtc/stats", get(get_stats))
        .route("/api/webrtc/health", get(health_check))
        .route("/api/webrtc/streams", get(get_camera_streams))
        .route("/api/webrtc/status", get(get_service_status))
        .route("/api/webrtc/monitors/{id}", get(get_monitor_info))
        .route("/api/webrtc/available-streams", get(get_available_streams))
        .nest_service("/static", ServeDir::new("static"))
}
