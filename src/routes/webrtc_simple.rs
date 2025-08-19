use axum::{
    routing::{get},
    Router,
};
use std::sync::Arc;
use tower_http::services::ServeDir;

use crate::handlers::webrtc;
use crate::webrtc_signaling::WebRTCSignalingService;

pub fn add_webrtc_routes(signaling_service: Arc<WebRTCSignalingService>) -> Router {
    Router::new()
        .route("/ws/webrtc", get(webrtc::websocket_handler))
        .route("/api/webrtc/stats", get(webrtc::get_stats))
        .route("/api/webrtc/health", get(webrtc::health_check))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(signaling_service)
}
