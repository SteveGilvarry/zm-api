use crate::handlers::user_preferences;
use crate::server::state::AppState;
use crate::util::middleware::auth_middleware;
use axum::{middleware, routing::get, Router};

pub fn add_user_preference_routes(router: Router<AppState>) -> Router<AppState> {
    let api_prefix = "/api/v3";
    let protected = Router::new()
        .route(
            &format!("{}/user_preferences", api_prefix),
            get(user_preferences::list_user_preferences)
                .post(user_preferences::create_user_preference),
        )
        .route(
            &format!("{}/user_preferences/{{id}}", api_prefix),
            get(user_preferences::get_user_preference)
                .patch(user_preferences::update_user_preference)
                .delete(user_preferences::delete_user_preference),
        )
        .layer(middleware::from_fn(auth_middleware));
    router.merge(protected)
}
