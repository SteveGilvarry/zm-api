use crate::handlers;
use crate::server::state::AppState;
use crate::util::middleware::auth_middleware;
use axum::middleware::from_fn_with_state;
use axum::routing::{delete, get, patch, post};
use axum::Router;

pub fn add_frames_routes(router: Router<AppState>) -> Router<AppState> {
    let state = router.clone();
    router.nest(
        "/frames",
        Router::new()
            .route("/", get(handlers::frames::list_frames))
            .route("/", post(handlers::frames::create_frame))
            .route("/{id}", get(handlers::frames::get_frame))
            .route("/{id}", patch(handlers::frames::update_frame))
            .route("/{id}", delete(handlers::frames::delete_frame))
            .layer(from_fn_with_state(state, auth_middleware)),
    )
}
