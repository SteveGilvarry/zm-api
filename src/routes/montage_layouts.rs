use axum::{Router, routing::get, middleware};
use crate::handlers::montage_layouts;
use crate::server::state::AppState;
use crate::util::middleware::auth_middleware;

pub fn add_montage_layout_routes(router: Router<AppState>) -> Router<AppState> {
    let api_prefix = "/api/v3";
    let protected = Router::new()
        .route(&format!("{}/montage_layouts", api_prefix), get(montage_layouts::list_montage_layouts).post(montage_layouts::create_montage_layout))
        .route(&format!("{}/montage_layouts/{{id}}", api_prefix), get(montage_layouts::get_montage_layout).patch(montage_layouts::update_montage_layout).delete(montage_layouts::delete_montage_layout))
        .layer(middleware::from_fn(auth_middleware));
    router.merge(protected)
}
