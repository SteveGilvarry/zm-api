use crate::handlers::media_jobs as h;
use crate::server::state::AppState;
use crate::util::middleware::auth_middleware;
use axum::{middleware, routing::get, Router};

/// An event's video files: with the streaming routes, outside compression.
/// Events-gated by the caller (View to read, Edit to generate or delete).
pub fn add_event_video_routes(router: Router<AppState>) -> Router<AppState> {
    let protected = Router::new()
        .route(
            "/api/v3/events/{id}/videos",
            get(h::list_videos).post(h::generate_video),
        )
        .route(
            "/api/v3/events/{id}/videos/{name}",
            get(h::download_video).delete(h::delete_video),
        )
        .layer(middleware::from_fn(auth_middleware));
    router.merge(protected)
}

/// The media job queue (Events-gated by the caller).
pub fn add_job_routes(router: Router<AppState>) -> Router<AppState> {
    let protected = Router::new()
        .route("/api/v3/jobs", get(h::list_jobs))
        .route("/api/v3/jobs/{id}", get(h::get_job).delete(h::delete_job))
        .layer(middleware::from_fn(auth_middleware));
    router.merge(protected)
}
