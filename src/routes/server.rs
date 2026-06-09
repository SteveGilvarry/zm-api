use crate::handlers::server;
use crate::server::state::AppState;
use crate::util::authz::{protect, Feature};
use axum::{
    routing::{get, post},
    Router,
};
use tracing::info;

pub fn add_server_routes(router: Router<AppState>) -> Router<AppState> {
    info!("Registering routes for server...");
    let api_prefix = "/api/v3";

    // Public routes — intentionally no auth.
    let public_routes = Router::new()
        .route(
            &format!("{}/server/health_check", api_prefix),
            get(server::health_check),
        )
        .route(
            &format!("{}/host/getVersion", api_prefix),
            get(server::get_version),
        );

    // `/states/change/{action}` invokes `systemctl restart/stop/start
    // zoneminder`, which is an admin-tier operation. Gate it behind the
    // `System` feature, not just authentication.
    let state_change_routes = protect(
        Router::new().route(
            &format!("{}/states/change/{{action}}", api_prefix),
            post(server::change_state),
        ),
        Feature::System,
    );

    router.merge(public_routes).merge(state_change_routes)
}
