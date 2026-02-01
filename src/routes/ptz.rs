//! PTZ control routes

use crate::handlers::ptz;
use crate::server::state::AppState;
use crate::util::middleware::auth_middleware;
use axum::{
    middleware,
    routing::{delete, get, post},
    Router,
};

pub fn add_ptz_routes(router: Router<AppState>) -> Router<AppState> {
    let api_prefix = "/api/v3";

    let protected = Router::new()
        // Status and capabilities
        .route(
            &format!("{}/ptz/monitors/{{id}}/status", api_prefix),
            get(ptz::get_status),
        )
        .route(
            &format!("{}/ptz/monitors/{{id}}/capabilities", api_prefix),
            get(ptz::get_capabilities),
        )
        .route(
            &format!("{}/ptz/protocols", api_prefix),
            get(ptz::list_protocols),
        )
        // Movement
        .route(
            &format!("{}/ptz/monitors/{{id}}/move/up", api_prefix),
            post(ptz::move_up),
        )
        .route(
            &format!("{}/ptz/monitors/{{id}}/move/down", api_prefix),
            post(ptz::move_down),
        )
        .route(
            &format!("{}/ptz/monitors/{{id}}/move/left", api_prefix),
            post(ptz::move_left),
        )
        .route(
            &format!("{}/ptz/monitors/{{id}}/move/right", api_prefix),
            post(ptz::move_right),
        )
        .route(
            &format!("{}/ptz/monitors/{{id}}/move/up-left", api_prefix),
            post(ptz::move_up_left),
        )
        .route(
            &format!("{}/ptz/monitors/{{id}}/move/up-right", api_prefix),
            post(ptz::move_up_right),
        )
        .route(
            &format!("{}/ptz/monitors/{{id}}/move/down-left", api_prefix),
            post(ptz::move_down_left),
        )
        .route(
            &format!("{}/ptz/monitors/{{id}}/move/down-right", api_prefix),
            post(ptz::move_down_right),
        )
        .route(
            &format!("{}/ptz/monitors/{{id}}/move/stop", api_prefix),
            post(ptz::move_stop),
        )
        // Zoom
        .route(
            &format!("{}/ptz/monitors/{{id}}/zoom/in", api_prefix),
            post(ptz::zoom_in),
        )
        .route(
            &format!("{}/ptz/monitors/{{id}}/zoom/out", api_prefix),
            post(ptz::zoom_out),
        )
        .route(
            &format!("{}/ptz/monitors/{{id}}/zoom/stop", api_prefix),
            post(ptz::zoom_stop),
        )
        // Focus
        .route(
            &format!("{}/ptz/monitors/{{id}}/focus/near", api_prefix),
            post(ptz::focus_near),
        )
        .route(
            &format!("{}/ptz/monitors/{{id}}/focus/far", api_prefix),
            post(ptz::focus_far),
        )
        .route(
            &format!("{}/ptz/monitors/{{id}}/focus/auto", api_prefix),
            post(ptz::focus_auto),
        )
        .route(
            &format!("{}/ptz/monitors/{{id}}/focus/stop", api_prefix),
            post(ptz::focus_stop),
        )
        // Presets
        .route(
            &format!(
                "{}/ptz/monitors/{{id}}/presets/{{preset_id}}/goto",
                api_prefix
            ),
            post(ptz::goto_preset),
        )
        .route(
            &format!(
                "{}/ptz/monitors/{{id}}/presets/{{preset_id}}/set",
                api_prefix
            ),
            post(ptz::set_preset),
        )
        .route(
            &format!("{}/ptz/monitors/{{id}}/presets/{{preset_id}}", api_prefix),
            delete(ptz::clear_preset),
        )
        // Home
        .route(
            &format!("{}/ptz/monitors/{{id}}/home", api_prefix),
            post(ptz::goto_home),
        )
        // Absolute/Relative positioning
        .route(
            &format!("{}/ptz/monitors/{{id}}/absolute", api_prefix),
            post(ptz::move_absolute),
        )
        .route(
            &format!("{}/ptz/monitors/{{id}}/relative", api_prefix),
            post(ptz::move_relative),
        )
        .layer(middleware::from_fn(auth_middleware));

    router.merge(protected)
}
