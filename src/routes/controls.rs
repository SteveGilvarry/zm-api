use crate::handlers::controls;
use crate::server::state::AppState;
use crate::util::middleware::auth_middleware;
use axum::{middleware, routing::get, Router};

pub fn add_control_routes(router: Router<AppState>) -> Router<AppState> {
    let api_prefix = "/api/v3";
    let protected = Router::new()
        .route(
            &format!("{}/controls", api_prefix),
            get(controls::list_controls).post(controls::create_control),
        )
        .route(
            &format!("{}/controls/{{id}}", api_prefix),
            get(controls::get_control)
                .patch(controls::update_control)
                .delete(controls::delete_control),
        )
        .layer(middleware::from_fn(auth_middleware));
    router.merge(protected)
}
