use axum::{Router, routing::get, middleware};
use crate::handlers::filters;
use crate::server::state::AppState;
use crate::util::middleware::auth_middleware;

pub fn add_filter_routes(router: Router<AppState>) -> Router<AppState> {
    let api_prefix = "/api/v3";
    let protected = Router::new()
        .route(&format!("{}/filters", api_prefix), get(filters::list_filters).post(filters::create_filter))
        .route(&format!("{}/filters/{{id}}", api_prefix), get(filters::get_filter).put(filters::update_filter).delete(filters::delete_filter))
        .layer(middleware::from_fn(auth_middleware));
    router.merge(protected)
}
