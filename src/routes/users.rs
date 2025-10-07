use axum::{Router, routing::get, middleware};
use crate::handlers::users;
use crate::server::state::AppState;
use crate::util::middleware::auth_middleware;

pub fn add_user_routes(router: Router<AppState>) -> Router<AppState> {
    let api_prefix = "/api/v3";
    let protected = Router::new()
        .route(&format!("{}/users", api_prefix), get(users::list_users).post(users::create_user))
        .route(&format!("{}/users/{{id}}", api_prefix), get(users::get_user).put(users::update_user).delete(users::delete_user))
        .layer(middleware::from_fn(auth_middleware));
    router.merge(protected)
}
