use crate::handlers::ai;
use crate::server::state::AppState;
use crate::util::middleware::auth_middleware;
use axum::{middleware, routing::get, Router};

/// AI object-detection registry routes.
///
/// Mirrors the CRUD ZoneMinder's own Options UI offers over these tables, and
/// like that UI the whole group is admin-tier — `routes::mod` wraps it with
/// `Feature::System`.
pub fn add_ai_routes(router: Router<AppState>) -> Router<AppState> {
    let api_prefix = "/api/v3";
    let protected = Router::new()
        .route(
            &format!("{}/ai/datasets", api_prefix),
            get(ai::list_datasets).post(ai::create_dataset),
        )
        .route(
            &format!("{}/ai/datasets/{{id}}", api_prefix),
            get(ai::get_dataset)
                .patch(ai::update_dataset)
                .delete(ai::delete_dataset),
        )
        .route(
            &format!("{}/ai/models", api_prefix),
            get(ai::list_models).post(ai::create_model),
        )
        .route(
            &format!("{}/ai/models/{{id}}", api_prefix),
            get(ai::get_model)
                .patch(ai::update_model)
                .delete(ai::delete_model),
        )
        .route(
            &format!("{}/ai/object-classes", api_prefix),
            get(ai::list_classes).post(ai::create_class),
        )
        .route(
            &format!("{}/ai/object-classes/{{id}}", api_prefix),
            get(ai::get_class)
                .patch(ai::update_class)
                .delete(ai::delete_class),
        )
        .layer(middleware::from_fn(auth_middleware));
    router.merge(protected)
}
