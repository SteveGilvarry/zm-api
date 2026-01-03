use crate::handlers::triggers_x10;
use crate::server::state::AppState;
use crate::util::middleware::auth_middleware;
use axum::{middleware, routing::get, Router};

pub fn add_trigger_x10_routes(router: Router<AppState>) -> Router<AppState> {
    let api_prefix = "/api/v3";
    let protected = Router::new()
        .route(
            &format!("{}/triggers_x10", api_prefix),
            get(triggers_x10::list_triggers_x10).post(triggers_x10::create_trigger_x10),
        )
        .route(
            &format!("{}/triggers_x10/{{monitor_id}}", api_prefix),
            get(triggers_x10::get_trigger_x10)
                .patch(triggers_x10::update_trigger_x10)
                .delete(triggers_x10::delete_trigger_x10),
        )
        .layer(middleware::from_fn(auth_middleware));
    router.merge(protected)
}
