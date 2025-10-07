use axum::{Router, routing::get, middleware};
use crate::handlers::sessions;
use crate::server::state::AppState;
use crate::util::middleware::auth_middleware;

pub fn add_session_routes(router: Router<AppState>) -> Router<AppState> {
    let api_prefix = "/api/v3";
    let protected = Router::new()
        .route(&format!("{}/sessions", api_prefix), get(sessions::list_sessions).post(sessions::create_session))
        .route(&format!("{}/sessions/{{id}}", api_prefix), get(sessions::get_session).patch(sessions::update_session).delete(sessions::delete_session))
        .layer(middleware::from_fn(auth_middleware));
    router.merge(protected)
}
