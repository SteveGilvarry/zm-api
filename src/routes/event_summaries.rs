use axum::{routing::get, Router};

use crate::{handlers, server::state::AppState};

/// Create event summaries router
pub fn add_event_summaries_routes(router: Router<AppState>) -> Router<AppState> {
    router.nest("/api/v3/event-summaries", routes())
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::event_summaries::list_event_summaries))
        .route(
            "/{monitor_id}",
            get(handlers::event_summaries::get_event_summary),
        )
}
