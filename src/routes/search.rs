//! Natural-language / semantic event search routes.
//!
//! JSON endpoints (safe to compress), gated like the rest of the Events feature
//! and additionally bounded by row-level [`MonitorScope`](crate::service::monitor_acl::MonitorScope)
//! ACL inside each handler.

use axum::{middleware, routing::get, Router};

use crate::handlers;
use crate::server::state::AppState;
use crate::util::middleware::auth_middleware;

/// Add the search routes (JWT-protected) to the router.
pub fn add_search_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route("/api/v3/search", get(handlers::search::search_events))
        .route(
            "/api/v3/events/{id}/similar",
            get(handlers::search::similar_events),
        )
        .layer(middleware::from_fn(auth_middleware))
}
