use axum::{Router, routing::get, middleware};
use crate::handlers::manufacturers;
use crate::server::state::AppState;
use crate::util::middleware::auth_middleware;

pub fn add_manufacturer_routes(router: Router<AppState>) -> Router<AppState> {
    let api_prefix = "/api/v3";
    let protected = Router::new()
        .route(&format!("{}/manufacturers", api_prefix), get(manufacturers::list_manufacturers).post(manufacturers::create_manufacturer))
        .route(&format!("{}/manufacturers/{{id}}", api_prefix), get(manufacturers::get_manufacturer).patch(manufacturers::update_manufacturer).delete(manufacturers::delete_manufacturer))
        .layer(middleware::from_fn(auth_middleware));
    router.merge(protected)
}
