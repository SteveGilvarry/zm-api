use axum::{
    routing::{get, post, delete},
    Router,
};
use tower_http::services::ServeDir;

use crate::{
    handlers::mse,
    server::state::AppState,
};

/// Add MSE (Media Source Extensions) routes
/// 
/// These routes provide:
/// - HTTP endpoints for fMP4 initialization and media segments
/// - WebSocket endpoints for live streaming
/// - Stream management and statistics
/// - Optional DASH/HLS manifest endpoints (future)
pub fn add_mse_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .nest("/api/v3/mse", mse_routes())
        .nest_service("/static", ServeDir::new("static"))
}

fn mse_routes() -> Router<AppState> {
    Router::new()
        // Stream management
        .route("/streams", get(mse::get_streams))
        .route("/streams/{camera_id}", post(mse::create_stream))
        .route("/streams/{camera_id}", get(mse::get_stream_info))
        .route("/streams/{camera_id}", delete(mse::delete_stream))
        
        // Segments
        .route("/streams/{camera_id}/init.mp4", get(mse::get_init_segment))
        .route("/streams/{camera_id}/segments/latest.mp4", get(mse::get_latest_segment))
        .route("/streams/{camera_id}/segments/{sequence}", get(mse::get_segment))
        .route("/streams/{camera_id}/segments/from/{start_sequence}", get(mse::get_segments_from))
        
        // Test endpoints
        .route("/streams/{camera_id}/test/pop", get(mse::test_pop_segment))
        
        // WebSocket live streaming
        .route("/streams/{camera_id}/live", get(mse::websocket_handler))
        
        // Statistics
        .route("/streams/{camera_id}/stats", get(mse::get_stream_stats))
        .route("/stats", get(mse::get_all_stats))
}
