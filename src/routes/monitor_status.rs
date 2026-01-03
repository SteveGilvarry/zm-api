use crate::handlers::monitor_status;
use crate::server::state::AppState;
use crate::util::middleware::auth_middleware;
use axum::{middleware, routing::get, Router};

pub fn add_monitor_status_routes(router: Router<AppState>) -> Router<AppState> {
    let api_prefix = "/api/v3";
    let protected = Router::new()
        .route(
            &format!("{}/monitor-status", api_prefix),
            get(monitor_status::list_monitor_statuses),
        )
        .route(
            &format!("{}/monitor-status/{{monitor_id}}", api_prefix),
            get(monitor_status::get_monitor_status).patch(monitor_status::update_monitor_status),
        )
        .layer(middleware::from_fn(auth_middleware));
    router.merge(protected)
}
