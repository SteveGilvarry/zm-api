use axum::{Router, routing::get, middleware};
use crate::handlers::zone_presets;
use crate::server::state::AppState;
use crate::util::middleware::auth_middleware;

pub fn add_zone_preset_routes(router: Router<AppState>) -> Router<AppState> {
    let api_prefix = "/api/v3";
    let protected = Router::new()
        .route(&format!("{}/zone-presets", api_prefix), get(zone_presets::list_zone_presets).post(zone_presets::create_zone_preset))
        .route(&format!("{}/zone-presets/{{id}}", api_prefix), get(zone_presets::get_zone_preset).patch(zone_presets::update_zone_preset).delete(zone_presets::delete_zone_preset))
        .layer(middleware::from_fn(auth_middleware));
    router.merge(protected)
}
