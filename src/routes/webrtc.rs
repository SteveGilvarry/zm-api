use crate::handlers::webrtc::{
    get_available_streams, get_camera_streams, get_monitor_info, get_service_status, get_stats,
    health_check, websocket_handler,
};
use crate::server::state::AppState;
use crate::util::middleware::auth_middleware;
use axum::{middleware, routing::get, Router};
use tower_http::services::ServeDir;

/// Add WebRTC routes under /api/v3/webrtc
///
/// Endpoints provided (relative to /api/v3/webrtc):
/// - GET /ws                   → WebSocket endpoint for signaling
/// - GET /stats                → Service statistics
/// - GET /health               → Health check
/// - GET /streams              → Discover available camera streams
/// - GET /status               → Detailed service status
/// - GET /monitors/{id}        → Monitor information (legacy)
/// - GET /available-streams    → All available streams
pub fn add_webrtc_routes(router: Router<AppState>) -> Router<AppState> {
    router.nest("/api/v3/webrtc", webrtc_routes())
}

fn webrtc_routes() -> Router<AppState> {
    Router::new()
        .route("/ws", get(websocket_handler))
        .route("/stats", get(get_stats))
        .route("/health", get(health_check))
        .route("/streams", get(get_camera_streams))
        .route("/status", get(get_service_status))
        .route("/monitors/{id}", get(get_monitor_info))
        .route("/available-streams", get(get_available_streams))
        .nest_service("/static", ServeDir::new("static"))
        .layer(middleware::from_fn(auth_middleware))
}
