//! Event playback routes
//!
//! Routes for playing back recorded events via HLS or direct video access.
//!
//! These routes use `media_auth_middleware` which accepts authentication via:
//! - `Authorization: Bearer <token>` header (standard)
//! - `?token=<token>` query parameter (for HTML5 media elements)

use axum::{routing::get, Router};

use crate::handlers::{events_playback, synopsis};
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
        // Range-served media for the in-progress (EVENT) playlist's byte-range
        // segments — the growing incomplete.*.mp4 while recording.
        .route(
            "/api/v3/events/{id}/stream/media.mp4",
            get(events_playback::get_event_stream_media)
                .route_layer(axum::middleware::from_fn(media_auth_middleware)),
        )
        // HLS-VOD init + media segments
        .route(
            "/api/v3/events/{id}/stream/init.mp4",
            get(events_playback::get_event_init)
                .route_layer(axum::middleware::from_fn(media_auth_middleware)),
        )
        .route(
            "/api/v3/events/{id}/stream/segment/{seq}",
            get(events_playback::get_event_segment)
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
        // Codec / dimensions / duration metadata
        .route(
            "/api/v3/events/{id}/info",
            get(events_playback::get_event_info)
                .route_layer(axum::middleware::from_fn(media_auth_middleware)),
        )
        // Motion synopsis — glanceable composite still (P1).
        .route(
            "/api/v3/events/{id}/synopsis/review",
            get(synopsis::get_event_synopsis_review)
                .route_layer(axum::middleware::from_fn(media_auth_middleware)),
        )
        // Motion synopsis — temporal layout preview (P2).
        .route(
            "/api/v3/events/{id}/synopsis/layout",
            get(synopsis::get_event_synopsis_layout)
                .route_layer(axum::middleware::from_fn(media_auth_middleware)),
        )
        // Motion synopsis — request/poll the rendered mp4 (P3).
        .route(
            "/api/v3/events/{id}/synopsis",
            get(synopsis::get_event_synopsis)
                .route_layer(axum::middleware::from_fn(media_auth_middleware)),
        )
        // Motion synopsis — stream the rendered mp4 (P3).
        .route(
            "/api/v3/events/{id}/synopsis/mp4",
            get(synopsis::get_event_synopsis_mp4)
                .route_layer(axum::middleware::from_fn(media_auth_middleware)),
        )
        // Motion synopsis — range/overview montage across events (P4). The static
        // `synopsis` segment sits alongside the `{id}` routes above.
        .route(
            "/api/v3/events/synopsis",
            get(synopsis::get_range_synopsis)
                .route_layer(axum::middleware::from_fn(media_auth_middleware)),
        )
}
