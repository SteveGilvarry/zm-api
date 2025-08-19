use axum::{
    routing::{delete, get, patch, post}, // Ensure all needed methods are imported
    Router,
};

use crate::{handlers, server::state::AppState};

/// Create events router using JWT middleware
pub fn add_event_routes(router: Router<AppState>) -> Router<AppState> {
    router.nest(
        "/api/v3/events",
        routes(),
    )
}

pub fn routes() -> Router<AppState> {
    Router::new()
        // Use fully qualified handler paths
        .route("/", get(handlers::events::index).post(handlers::events::create))
        .route(
            "/{id}",
            // Use fully qualified handler paths
            get(handlers::events::get)
                .patch(handlers::events::update)
                .delete(handlers::events::delete),
        )
        // Use fully qualified handler path
        .route("/counts/{hours}", get(handlers::events::counts))
}