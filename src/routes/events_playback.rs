//! Event playback routes
//!
//! Routes for playing back recorded events via HLS or direct video access.
//!
//! These routes use `media_auth_middleware` which accepts authentication via:
//! - `Authorization: Bearer <token>` header (standard)
//! - `?token=<token>` query parameter (for HTML5 media elements)

use axum::{routing::get, Router};

use crate::handlers::events_playback;
use crate::server::state::AppState;
use crate::util::middleware::media_auth_middleware;

/// Add event playback routes to the router
pub fn add_events_playback_routes(router: Router<AppState>) -> Router<AppState> {
    router
        // HLS streaming endpoints
        .route(
            "/api/v3/events/{id}/stream/playlist.m3u8",
            get(events_playback::get_event_playlist)
                .route_layer(axum::middleware::from_fn(media_auth_middleware)),
        )
        .route(
            "/api/v3/events/{id}/stream/video.mp4",
            get(events_playback::get_event_stream_video)
                .route_layer(axum::middleware::from_fn(media_auth_middleware)),
        )
        // Direct video access
        .route(
            "/api/v3/events/{id}/video",
            get(events_playback::get_event_video)
                .route_layer(axum::middleware::from_fn(media_auth_middleware)),
        )
        // Thumbnail
        .route(
            "/api/v3/events/{id}/thumbnail",
            get(events_playback::get_event_thumbnail)
                .route_layer(axum::middleware::from_fn(media_auth_middleware)),
        )
}
