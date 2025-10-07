use axum::{Router, routing::get, middleware};
use crate::handlers::tags;
use crate::server::state::AppState;
use crate::util::middleware::auth_middleware;

pub fn add_tag_routes(router: Router<AppState>) -> Router<AppState> {
    let api_prefix = "/api/v3";
    let protected = Router::new()
        .route(&format!("{}/tags", api_prefix), get(tags::list_tags).post(tags::create_tag))
        .route(&format!("{}/tags/{{id}}", api_prefix), get(tags::get_tag).patch(tags::update_tag).delete(tags::delete_tag))
        .layer(middleware::from_fn(auth_middleware));
    router.merge(protected)
}
