use axum::{middleware, routing::get, Router};

use crate::{handlers::go2rtc_proxy, server::state::AppState, util::middleware::auth_middleware};

/// Add go2rtc proxy routes
///
/// These routes provide secure WebSocket proxying to go2rtc with JWT authentication.
/// go2rtc should be configured to bind ONLY to localhost:1984 for security.
///
/// Endpoints:
/// - GET /api/v3/go2rtc/{monitor_id}/ws - Proxy WebSocket connection for a monitor
/// - GET /api/v3/go2rtc/{monitor_id}/ws/{stream_type} - Proxy with explicit stream type
///
/// Security features:
/// - JWT authentication required on all endpoints
/// - go2rtc URLs are always localhost (prevents URL injection)
/// - All connections are logged for audit
/// - Proper cleanup when either side disconnects
pub fn add_go2rtc_proxy_routes(router: Router<AppState>) -> Router<AppState> {
    router.nest("/api/v3/go2rtc", go2rtc_proxy_routes())
}

fn go2rtc_proxy_routes() -> Router<AppState> {
    Router::new()
        // WebSocket proxy endpoints
        .route("/{monitor_id}/ws", get(go2rtc_proxy::go2rtc_ws_proxy))
        .route(
            "/{monitor_id}/ws/{stream_type}",
            get(go2rtc_proxy::go2rtc_typed_ws_proxy),
        )
        // Apply JWT auth to all go2rtc proxy routes
        .layer(middleware::from_fn(auth_middleware))
}
