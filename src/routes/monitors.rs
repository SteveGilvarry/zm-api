use crate::handlers::{monitor, monitor_pipeline};
use crate::server::state::AppState;
use crate::util::middleware::auth_middleware;
use axum::{
    middleware,
    routing::{get, patch, post},
    Router,
};
use tracing::info;

pub fn add_monitor_routes(router: Router<AppState>) -> Router<AppState> {
    info!("Registering routes for monitors...");

    let api_prefix = "/api/v3";

    // Create a router with all monitor endpoints and apply auth middleware to all of them
    let protected_routes = Router::new()
        .route(
            &format!("{}/monitors", api_prefix),
            get(monitor::list_monitors).post(monitor::create_monitor),
        )
        .route(
            &format!("{}/monitors/{{id}}", api_prefix),
            get(monitor::get_monitor)
                .patch(monitor::update_monitor)
                .delete(monitor::delete_monitor),
        )
        .route(
            &format!("{}/monitors/{{id}}/pipeline", api_prefix),
            get(monitor_pipeline::get_monitor_pipeline)
                .put(monitor_pipeline::put_monitor_pipeline)
                .delete(monitor_pipeline::delete_monitor_pipeline),
        )
        .route(
            &format!("{}/monitors/{{id}}/zmnext", api_prefix),
            post(monitor_pipeline::enable_monitor_zmnext)
                .delete(monitor_pipeline::disable_monitor_zmnext),
        )
        .route(
            &format!("{}/monitors/{{id}}/state", api_prefix),
            patch(monitor::update_state),
        )
        .route(
            &format!("{}/monitors/{{id}}/alarm", api_prefix),
            patch(monitor::alarm_control),
        )
        .layer(middleware::from_fn(auth_middleware));

    router.merge(protected_routes)
}
