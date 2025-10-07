use axum::{Router, routing::get, middleware};
use crate::handlers::monitor_presets;
use crate::server::state::AppState;
use crate::util::middleware::auth_middleware;

pub fn add_monitor_preset_routes(router: Router<AppState>) -> Router<AppState> {
    let api_prefix = "/api/v3";
    let protected = Router::new()
        .route(&format!("{}/monitor_presets", api_prefix), get(monitor_presets::list_monitor_presets).post(monitor_presets::create_monitor_preset))
        .route(&format!("{}/monitor_presets/{{id}}", api_prefix), get(monitor_presets::get_monitor_preset).patch(monitor_presets::update_monitor_preset).delete(monitor_presets::delete_monitor_preset))
        .layer(middleware::from_fn(auth_middleware));
    router.merge(protected)
}
