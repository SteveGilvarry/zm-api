use axum::{middleware, routing::get, Router};

use crate::util::middleware::auth_middleware;
use crate::{handlers, server::state::AppState};

/// Create events router using JWT middleware
pub fn add_event_routes(router: Router<AppState>) -> Router<AppState> {
    router.nest("/api/v3/events", routes())
}

pub fn routes() -> Router<AppState> {
    Router::new()
        // Use fully qualified handler paths
        .route(
            "/",
            get(handlers::events::list_events).post(handlers::events::create_event),
        )
        .route(
            "/{id}",
            // Use fully qualified handler paths
            get(handlers::events::get_event)
                .patch(handlers::events::update_event)
                .delete(handlers::events::delete_event),
        )
        // Event counts grouped by hour
        .route("/counts/{hours}", get(handlers::events::get_event_counts))
        // Event counts grouped by monitor (for console view)
        .route(
            "/counts-by-monitor/{hours}",
            get(handlers::events::get_event_counts_by_monitor),
        )
        .layer(middleware::from_fn(auth_middleware))
}
