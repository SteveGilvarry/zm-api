use axum::{middleware, routing::get, Router};

use crate::handlers::events_tags;
use crate::server::state::AppState;
use crate::util::middleware::auth_middleware;

pub fn add_event_tag_routes(router: Router<AppState>) -> Router<AppState> {
    let api_prefix = "/api/v3";
    let protected = Router::new()
        .route(
            &format!("{}/events-tags", api_prefix),
            get(events_tags::list_events_tags).post(events_tags::create_event_tag),
        )
        .route(
            &format!("{}/events-tags/{{tag_id}}/{{event_id}}", api_prefix),
            get(events_tags::get_event_tag).delete(events_tags::delete_event_tag),
        )
        .layer(middleware::from_fn(auth_middleware));
    router.merge(protected)
}
