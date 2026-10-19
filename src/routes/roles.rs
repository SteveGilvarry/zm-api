use crate::handlers::roles;
use crate::server::state::AppState;
use crate::util::middleware::auth_middleware;
use axum::{middleware, routing::get, Router};

pub fn add_role_routes(router: Router<AppState>) -> Router<AppState> {
    let protected = Router::new()
        .route(
            "/api/v3/roles",
            get(roles::list_roles).post(roles::create_role),
        )
        .route(
            "/api/v3/roles/{id}",
            get(roles::get_role)
                .put(roles::update_role)
                .delete(roles::delete_role),
        )
        .layer(middleware::from_fn(auth_middleware));
    router.merge(protected)
}
