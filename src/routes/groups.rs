use crate::handlers::groups;
use crate::server::state::AppState;
use crate::util::middleware::auth_middleware;
use axum::{middleware, routing::get, Router};

pub fn add_group_routes(router: Router<AppState>) -> Router<AppState> {
    let api_prefix = "/api/v3";
    let protected = Router::new()
        .route(
            &format!("{}/groups", api_prefix),
            get(groups::list_groups).post(groups::create_group),
        )
        .route(
            &format!("{}/groups/{{id}}", api_prefix),
            get(groups::get_group)
                .put(groups::update_group)
                .delete(groups::delete_group),
        )
        .layer(middleware::from_fn(auth_middleware));
    router.merge(protected)
}
