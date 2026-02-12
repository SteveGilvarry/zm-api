//! Live streaming routes
//!
//! Provides unified routes for live streaming via HLS, WebRTC, and MSE.

use axum::{
    routing::{delete, get, post},
    Router,
};

use crate::handlers::live;
use crate::server::state::AppState;
use crate::util::middleware::{auth_middleware, media_auth_middleware};

/// Create live streaming routes
pub fn routes() -> Router<AppState> {
    Router::new()
        // Session control (requires auth)
        .route("/start", post(live::start_live_stream))
        .route("/stop", delete(live::stop_live_stream))
        .route("/stats", get(live::get_live_stats))
        // HLS endpoints (no auth for media delivery)
        .route("/hls/master.m3u8", get(live::get_live_master_playlist))
        .route("/hls/live.m3u8", get(live::get_live_media_playlist))
        .route("/hls/init.mp4", get(live::get_live_init_segment))
        .route("/hls/{segment}", get(live::get_live_segment))
        // MSE endpoints (WebSocket fMP4 streaming)
        .route("/mse/ws", get(live::mse_websocket_handler))
        .route("/mse/init.mp4", get(live::get_mse_init_segment))
        // WebRTC endpoints (WebSocket signaling)
        .route("/webrtc/ws", get(live::webrtc_websocket_handler))
}

/// Add live streaming routes to the router
pub fn add_live_routes(router: Router<AppState>) -> Router<AppState> {
    router
        // Per-monitor live streaming endpoints
        .nest("/api/v3/live/{monitor_id}", routes())
        // Global live streaming endpoints
        .route(
            "/api/v3/live/sessions",
            get(live::list_live_sessions).route_layer(axum::middleware::from_fn(auth_middleware)),
        )
        .route(
            "/api/v3/live/sources",
            get(live::get_live_sources).route_layer(axum::middleware::from_fn(auth_middleware)),
        )
        // Monitor snapshot (supports token query param for <img> tags)
        .route(
            "/api/v3/monitors/{monitor_id}/snapshot",
            get(live::get_monitor_snapshot)
                .route_layer(axum::middleware::from_fn(media_auth_middleware)),
        )
}
