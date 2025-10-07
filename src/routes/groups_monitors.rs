use axum::{Router, routing::get, middleware};
use crate::handlers::groups_monitors;
use crate::server::state::AppState;
use crate::util::middleware::auth_middleware;

pub fn add_group_monitor_routes(router: Router<AppState>) -> Router<AppState> {
    let api_prefix = "/api/v3";
    let protected = Router::new()
        .route(&format!("{}/groups-monitors", api_prefix), get(groups_monitors::list_groups_monitors).post(groups_monitors::create_group_monitor))
        .route(&format!("{}/groups-monitors/{{id}}", api_prefix), get(groups_monitors::get_group_monitor).delete(groups_monitors::delete_group_monitor))
        .layer(middleware::from_fn(auth_middleware));
    router.merge(protected)
}
