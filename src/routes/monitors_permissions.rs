use crate::handlers::monitors_permissions;
use crate::server::state::AppState;
use crate::util::middleware::auth_middleware;
use axum::{middleware, routing::get, Router};

pub fn add_monitor_permission_routes(router: Router<AppState>) -> Router<AppState> {
    let api_prefix = "/api/v3";
    let protected = Router::new()
        .route(
            &format!("{}/monitors-permissions", api_prefix),
            get(monitors_permissions::list_monitors_permissions)
                .post(monitors_permissions::create_monitor_permission),
        )
        .route(
            &format!("{}/monitors-permissions/{{id}}", api_prefix),
            get(monitors_permissions::get_monitor_permission)
                .patch(monitors_permissions::update_monitor_permission)
                .delete(monitors_permissions::delete_monitor_permission),
        )
        .layer(middleware::from_fn(auth_middleware));
    router.merge(protected)
}
