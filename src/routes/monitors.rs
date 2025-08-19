use axum::{
    Router,
    routing::{get, patch},
    middleware,
};
use crate::handlers::monitor;
use crate::server::state::AppState;
use crate::util::middleware::auth_middleware;
use tracing::info;

pub fn add_monitor_routes(router: Router<AppState>) -> Router<AppState> {
    info!("Registering routes for monitors...");
    
    let api_prefix = "/api/v3";
    
    // Create a router with all monitor endpoints and apply auth middleware to all of them
    let protected_routes = Router::new()
        .route(
            &format!("{}/monitors", api_prefix),
            get(monitor::index).post(monitor::create)
        )
        .route(
            &format!("{}/monitors/{{id}}", api_prefix),
            get(monitor::view).patch(monitor::edit).delete(monitor::delete)
        )
        .route(
            &format!("{}/monitors/{{id}}/state", api_prefix),
            patch(monitor::update_state)
        )
        .route(
            &format!("{}/monitors/{{id}}/alarm", api_prefix),
            patch(monitor::alarm_control)
        )
        .layer(middleware::from_fn(auth_middleware));
        
    router.merge(protected_routes)
}