use crate::handlers::devices;
use crate::server::state::AppState;
use crate::util::middleware::auth_middleware;
use axum::{middleware, routing::get, Router};

pub fn add_device_routes(router: Router<AppState>) -> Router<AppState> {
    let api_prefix = "/api/v3";
    let protected = Router::new()
        .route(
            &format!("{}/devices", api_prefix),
            get(devices::list_devices).post(devices::create_device),
        )
        .route(
            &format!("{}/devices/{{id}}", api_prefix),
            get(devices::get_device)
                .patch(devices::update_device)
                .delete(devices::delete_device),
        )
        .layer(middleware::from_fn(auth_middleware));
    router.merge(protected)
}
