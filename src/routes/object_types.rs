use axum::{Router, routing::get, middleware};
use crate::handlers::object_types;
use crate::server::state::AppState;
use crate::util::middleware::auth_middleware;

pub fn add_object_type_routes(router: Router<AppState>) -> Router<AppState> {
    let api_prefix = "/api/v3";
    let protected = Router::new()
        .route(&format!("{}/object-types", api_prefix), get(object_types::list_object_types).post(object_types::create_object_type))
        .route(&format!("{}/object-types/{{id}}", api_prefix), get(object_types::get_object_type).patch(object_types::update_object_type).delete(object_types::delete_object_type))
        .layer(middleware::from_fn(auth_middleware));
    router.merge(protected)
}
