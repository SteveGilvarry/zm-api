use axum::{Router, routing::get, middleware};
use crate::handlers::servers;
use crate::server::state::AppState;
use crate::util::middleware::auth_middleware;

pub fn add_server_info_routes(router: Router<AppState>) -> Router<AppState> {
    let api_prefix = "/api/v3";
    let protected = Router::new()
        .route(&format!("{}/servers", api_prefix), get(servers::list_servers).post(servers::create_server))
        .route(&format!("{}/servers/{{id}}", api_prefix), get(servers::get_server).patch(servers::update_server).delete(servers::delete_server))
        .layer(middleware::from_fn(auth_middleware));
    router.merge(protected)
}
