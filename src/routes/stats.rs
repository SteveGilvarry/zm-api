use crate::handlers::stats;
use crate::server::state::AppState;
use crate::util::middleware::auth_middleware;
use axum::{middleware, routing::get, Router};

pub fn add_stat_routes(router: Router<AppState>) -> Router<AppState> {
    let api_prefix = "/api/v3";
    let protected = Router::new()
        .route(
            &format!("{}/stats", api_prefix),
            get(stats::list_stats).post(stats::create_stat),
        )
        .route(
            &format!("{}/stats/{{id}}", api_prefix),
            get(stats::get_stat)
                .patch(stats::update_stat)
                .delete(stats::delete_stat),
        )
        .layer(middleware::from_fn(auth_middleware));
    router.merge(protected)
}
