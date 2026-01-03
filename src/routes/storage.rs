use crate::handlers::storage;
use crate::server::state::AppState;
use crate::util::middleware::auth_middleware;
use axum::{middleware, routing::get, Router};

pub fn add_storage_routes(router: Router<AppState>) -> Router<AppState> {
    let api_prefix = "/api/v3";
    let protected = Router::new()
        .route(
            &format!("{}/storage", api_prefix),
            get(storage::list_storage).post(storage::create_storage),
        )
        .route(
            &format!("{}/storage/{{id}}", api_prefix),
            get(storage::get_storage)
                .patch(storage::update_storage)
                .delete(storage::delete_storage),
        )
        .layer(middleware::from_fn(auth_middleware));
    router.merge(protected)
}
