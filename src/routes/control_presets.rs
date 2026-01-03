use crate::handlers::control_presets;
use crate::server::state::AppState;
use crate::util::middleware::auth_middleware;
use axum::{middleware, routing::get, Router};

pub fn add_control_preset_routes(router: Router<AppState>) -> Router<AppState> {
    let api_prefix = "/api/v3";
    let protected = Router::new()
        .route(
            &format!("{}/control_presets", api_prefix),
            get(control_presets::list_control_presets).post(control_presets::create_control_preset),
        )
        .route(
            &format!("{}/control_presets/{{monitor_id}}/{{preset}}", api_prefix),
            get(control_presets::get_control_preset)
                .patch(control_presets::update_control_preset)
                .delete(control_presets::delete_control_preset),
        )
        .layer(middleware::from_fn(auth_middleware));
    router.merge(protected)
}
