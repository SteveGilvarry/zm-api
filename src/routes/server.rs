use axum::{Router, routing::{get, post}, middleware::from_fn_with_state};
use crate::handlers::server;
use crate::util::middleware::auth_middleware;
use crate::server::state::AppState;
use tracing::info;

pub fn add_server_routes(router: Router<AppState>) -> Router<AppState> {
    info!("Registering routes for server...");
    let api_prefix = "/api/v3";
    let state = router.clone();
    let server_routes = Router::new()
        .route(
            &format!("{}/server/health_check", api_prefix),
            get(server::health_check)
        )
        .route(
            &format!("{}/host/getVersion", api_prefix),
            get(server::get_version)
        )
        .route(
            &format!("{}/states/change/{{action}}", api_prefix),
            post(server::change_state)
                .layer(from_fn_with_state(state, auth_middleware))
        );
    router.merge(server_routes)
}
