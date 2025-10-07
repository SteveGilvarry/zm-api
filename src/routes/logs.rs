use axum::{Router, routing::get, middleware};
use crate::handlers::logs;
use crate::server::state::AppState;
use crate::util::middleware::auth_middleware;

pub fn add_log_routes(router: Router<AppState>) -> Router<AppState> {
    let api_prefix = "/api/v3";
    let protected = Router::new()
        .route(&format!("{}/logs", api_prefix), get(logs::list_logs))
        .route(&format!("{}/logs/{{id}}", api_prefix), get(logs::get_log))
        .layer(middleware::from_fn(auth_middleware));
    router.merge(protected)
}

