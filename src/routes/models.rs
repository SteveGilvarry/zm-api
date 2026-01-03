use crate::handlers::models;
use crate::server::state::AppState;
use crate::util::middleware::auth_middleware;
use axum::{middleware, routing::get, Router};

pub fn add_model_routes(router: Router<AppState>) -> Router<AppState> {
    let api_prefix = "/api/v3";
    let protected = Router::new()
        .route(
            &format!("{}/models", api_prefix),
            get(models::list_models).post(models::create_model),
        )
        .route(
            &format!("{}/models/{{id}}", api_prefix),
            get(models::get_model)
                .patch(models::update_model)
                .delete(models::delete_model),
        )
        .layer(middleware::from_fn(auth_middleware));
    router.merge(protected)
}
