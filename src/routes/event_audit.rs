use crate::handlers::event_audit;
use crate::server::state::AppState;
use crate::util::middleware::auth_middleware;
use axum::{middleware, routing::get, Router};

pub fn add_event_audit_routes(router: Router<AppState>) -> Router<AppState> {
    router.merge(
        Router::new()
            .route("/api/v3/reports/event-audit", get(event_audit::event_audit))
            .layer(middleware::from_fn(auth_middleware)),
    )
}
