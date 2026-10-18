use crate::handlers::monitor_actions as h;
use crate::server::state::AppState;
use crate::util::middleware::auth_middleware;
use axum::{
    middleware,
    routing::{get, post},
    Router,
};

/// MonitorActions CRUD and targets (Monitors-gated by the caller).
pub fn add_monitor_action_routes(router: Router<AppState>) -> Router<AppState> {
    let protected = Router::new()
        .route(
            "/api/v3/monitors/{id}/actions",
            get(h::list_actions).post(h::create_action),
        )
        .route(
            "/api/v3/monitors/{id}/actions/targets",
            get(h::list_targets),
        )
        .route(
            "/api/v3/monitors/{id}/actions/{action_id}",
            get(h::get_action)
                .patch(h::update_action)
                .delete(h::delete_action),
        )
        .layer(middleware::from_fn(auth_middleware));
    router.merge(protected)
}

/// Running a Manual action (Control-gated at View by the caller).
pub fn add_monitor_action_run_route(router: Router<AppState>) -> Router<AppState> {
    router.route(
        "/api/v3/monitors/{id}/actions/{action_id}/run",
        post(h::run_action).layer(middleware::from_fn(auth_middleware)),
    )
}
