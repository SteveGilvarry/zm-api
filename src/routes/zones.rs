use axum::{Router, routing::get, middleware};
use crate::handlers::zones;
use crate::server::state::AppState;
use crate::util::middleware::auth_middleware;

pub fn add_zone_routes(router: Router<AppState>) -> Router<AppState> {
    let api_prefix = "/api/v3";

    let protected = Router::new()
        .route(&format!("{}/monitors/{{id}}/zones", api_prefix), get(zones::list_by_monitor).post(zones::create))
        .route(&format!("{}/zones/{{id}}", api_prefix), get(zones::get).put(zones::update).delete(zones::delete))
        .layer(middleware::from_fn(auth_middleware));

    router.merge(protected)
}
