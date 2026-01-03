use crate::handlers::server_stats;
use crate::server::state::AppState;
use crate::util::middleware::auth_middleware;
use axum::{middleware, routing::get, Router};

pub fn add_server_stat_routes(router: Router<AppState>) -> Router<AppState> {
    let api_prefix = "/api/v3";
    let protected = Router::new()
        .route(
            &format!("{}/server-stats", api_prefix),
            get(server_stats::list_server_stats).post(server_stats::create_server_stat),
        )
        .route(
            &format!("{}/server-stats/{{id}}", api_prefix),
            get(server_stats::get_server_stat).delete(server_stats::delete_server_stat),
        )
        .layer(middleware::from_fn(auth_middleware));
    router.merge(protected)
}
