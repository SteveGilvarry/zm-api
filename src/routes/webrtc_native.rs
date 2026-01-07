use crate::handlers::webrtc_native;
use crate::server::state::AppState;
use crate::util::middleware::auth_middleware;
use axum::{
    middleware,
    routing::{delete, get, post},
    Router,
};
use tracing::info;

/// Add native WebRTC routes under /api/v3/webrtc/native
///
/// Endpoints provided:
/// - GET  /api/v3/webrtc/native/{camera_id}/signaling  → WebSocket signaling endpoint
/// - POST /api/v3/webrtc/native/{camera_id}/offer      → REST SDP offer (fallback)
/// - POST /api/v3/webrtc/native/{camera_id}/{session_id}/candidate → Add ICE candidate
/// - DELETE /api/v3/webrtc/native/{session_id}         → Close session
/// - GET  /api/v3/webrtc/native/stats                  → Engine statistics
/// - GET  /api/v3/webrtc/native/sessions               → List active sessions
/// - GET  /api/v3/webrtc/native/sessions/{session_id}  → Get session details
/// - GET  /api/v3/webrtc/native/health                 → Health check
pub fn add_native_webrtc_routes(router: Router<AppState>) -> Router<AppState> {
    info!("Registering native WebRTC routes...");

    // Apply auth middleware to all routes except health
    let authenticated_routes = Router::new()
        .route(
            "/{camera_id}/signaling",
            get(webrtc_native::signaling_websocket),
        )
        .route("/{camera_id}/offer", post(webrtc_native::handle_offer))
        .route(
            "/{camera_id}/{session_id}/candidate",
            post(webrtc_native::add_ice_candidate),
        )
        .route("/{session_id}", delete(webrtc_native::close_session))
        .route("/stats", get(webrtc_native::get_stats))
        .route("/sessions", get(webrtc_native::list_sessions))
        .route("/sessions/{session_id}", get(webrtc_native::get_session))
        .layer(middleware::from_fn(auth_middleware));

    let public_routes = Router::new().route("/health", get(webrtc_native::health_check));

    router.nest(
        "/api/v3/webrtc/native",
        authenticated_routes.merge(public_routes),
    )
}
