//! HLS streaming routes

use axum::{
    routing::{delete, get, post},
    Router,
};

use crate::handlers::hls;
use crate::server::state::AppState;

/// Create HLS streaming routes
pub fn routes() -> Router<AppState> {
    Router::new()
        // Session management
        .route("/sessions", get(hls::list_sessions))
        // Per-camera routes
        .route("/{camera_id}/start", post(hls::start_hls_stream))
        .route("/{camera_id}/stop", delete(hls::stop_hls_stream))
        .route("/{camera_id}/stats", get(hls::get_hls_stats))
        // Playlist routes
        .route("/{camera_id}/master.m3u8", get(hls::get_master_playlist))
        .route("/{camera_id}/live.m3u8", get(hls::get_media_playlist))
        // Segment routes
        .route("/{camera_id}/init.mp4", get(hls::get_init_segment))
        .route("/{camera_id}/{segment}", get(hls::get_segment))
}
