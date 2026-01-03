use crate::handlers::groups_permissions;
use crate::server::state::AppState;
use crate::util::middleware::auth_middleware;
use axum::{middleware, routing::get, Router};

pub fn add_group_permission_routes(router: Router<AppState>) -> Router<AppState> {
    let api_prefix = "/api/v3";
    let protected = Router::new()
        .route(
            &format!("{}/groups-permissions", api_prefix),
            get(groups_permissions::list_groups_permissions)
                .post(groups_permissions::create_group_permission),
        )
        .route(
            &format!("{}/groups-permissions/{{id}}", api_prefix),
            get(groups_permissions::get_group_permission)
                .patch(groups_permissions::update_group_permission)
                .delete(groups_permissions::delete_group_permission),
        )
        .layer(middleware::from_fn(auth_middleware));
    router.merge(protected)
}
