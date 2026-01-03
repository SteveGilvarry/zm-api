use crate::handlers::{streaming, webrtc_signaling};
use crate::server::state::AppState;
use crate::util::middleware::auth_middleware;
#[allow(unused_imports)]
use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};
use tracing::info;

pub fn add_streaming_routes(router: Router<AppState>) -> Router<AppState> {
    info!("Registering routes for streaming...");

    let api_prefix = "/api/v3";

    // Create a router with all streaming endpoints and apply auth middleware to all of them
    let streaming_routes = Router::new()
        // Existing streaming endpoints (go2rtc integration)
        .route(
            &format!("{}/streams/{{id}}", api_prefix),
            put(streaming::register_stream)
                .get(streaming::get_stream)
                .delete(streaming::delete_stream),
        )
        // WebRTC signaling endpoints
        .route(
            &format!(
                "{}/streaming/webrtc/{{camera_id}}/{{viewer_id}}/offer",
                api_prefix
            ),
            get(webrtc_signaling::get_webrtc_offer),
        )
        .route(
            &format!(
                "{}/streaming/webrtc/{{camera_id}}/{{viewer_id}}/answer",
                api_prefix
            ),
            post(webrtc_signaling::send_webrtc_answer),
        )
        .route(
            &format!(
                "{}/streaming/webrtc/{{camera_id}}/{{viewer_id}}/candidate",
                api_prefix
            ),
            post(webrtc_signaling::send_webrtc_candidate),
        )
        .route(
            &format!("{}/streaming/webrtc/{{viewer_id}}", api_prefix),
            delete(webrtc_signaling::drop_webrtc_viewer),
        )
        // WebRTC management endpoints
        .route(
            &format!("{}/streaming/sessions", api_prefix),
            get(webrtc_signaling::get_webrtc_sessions),
        )
        .route(
            &format!("{}/streaming/health", api_prefix),
            get(webrtc_signaling::test_webrtc_connection),
        )
        .layer(middleware::from_fn(auth_middleware));

    router.merge(streaming_routes)
}
