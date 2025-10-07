use axum::{Router, routing::get, middleware};
use crate::handlers::configs;
use crate::server::state::AppState;
use crate::util::middleware::auth_middleware;

pub fn add_config_routes(router: Router<AppState>) -> Router<AppState> {
    let api_prefix = "/api/v3";

    let protected = Router::new()
        .route(&format!("{}/configs", api_prefix), get(configs::list_configs))
        .route(&format!("{}/configs/{{name}}", api_prefix), get(configs::get_config).put(configs::update_config))
        .layer(middleware::from_fn(auth_middleware));

    router.merge(protected)
}
