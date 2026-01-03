use crate::handlers::states;
use crate::server::state::AppState;
use crate::util::middleware::auth_middleware;
use axum::{middleware, routing::get, Router};

pub fn add_state_routes(router: Router<AppState>) -> Router<AppState> {
    let api_prefix = "/api/v3";
    let protected = Router::new()
        .route(
            &format!("{}/states", api_prefix),
            get(states::list_states).post(states::create_state),
        )
        .route(
            &format!("{}/states/{{id}}", api_prefix),
            get(states::get_state)
                .patch(states::update_state)
                .delete(states::delete_state),
        )
        .layer(middleware::from_fn(auth_middleware));
    router.merge(protected)
}
