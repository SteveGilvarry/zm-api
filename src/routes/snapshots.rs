use axum::{Router, routing::get, middleware};
use crate::handlers::snapshots;
use crate::server::state::AppState;
use crate::util::middleware::auth_middleware;

pub fn add_snapshot_routes(router: Router<AppState>) -> Router<AppState> {
    let api_prefix = "/api/v3";
    let protected = Router::new()
        .route(&format!("{}/snapshots", api_prefix), get(snapshots::list_snapshots).post(snapshots::create_snapshot))
        .route(&format!("{}/snapshots/{{id}}", api_prefix), get(snapshots::get_snapshot).patch(snapshots::update_snapshot).delete(snapshots::delete_snapshot))
        .layer(middleware::from_fn(auth_middleware));
    router.merge(protected)
}
