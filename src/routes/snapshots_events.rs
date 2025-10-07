use axum::{Router, routing::get, middleware};
use crate::handlers::snapshots_events;
use crate::server::state::AppState;
use crate::util::middleware::auth_middleware;

pub fn add_snapshot_event_routes(router: Router<AppState>) -> Router<AppState> {
    let api_prefix = "/api/v3";
    let protected = Router::new()
        .route(&format!("{}/snapshots-events", api_prefix), get(snapshots_events::list_snapshot_events).post(snapshots_events::create_snapshot_event))
        .route(&format!("{}/snapshots-events/{{id}}", api_prefix), get(snapshots_events::get_snapshot_event).delete(snapshots_events::delete_snapshot_event))
        .layer(middleware::from_fn(auth_middleware));
    router.merge(protected)
}
