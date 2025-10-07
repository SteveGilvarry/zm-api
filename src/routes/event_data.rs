use axum::{Router, routing::get, middleware};
use crate::handlers::event_data;
use crate::server::state::AppState;
use crate::util::middleware::auth_middleware;

pub fn add_event_data_routes(router: Router<AppState>) -> Router<AppState> {
    let api_prefix = "/api/v3";
    let protected = Router::new()
        .route(&format!("{}/event-data", api_prefix), get(event_data::list_event_data).post(event_data::create_event_data))
        .route(&format!("{}/event-data/{{id}}", api_prefix), get(event_data::get_event_data).patch(event_data::update_event_data).delete(event_data::delete_event_data))
        .layer(middleware::from_fn(auth_middleware));
    router.merge(protected)
}
