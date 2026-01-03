use crate::handlers::reports;
use crate::server::state::AppState;
use crate::util::middleware::auth_middleware;
use axum::{middleware, routing::get, Router};

pub fn add_report_routes(router: Router<AppState>) -> Router<AppState> {
    let api_prefix = "/api/v3";
    let protected = Router::new()
        .route(
            &format!("{}/reports", api_prefix),
            get(reports::list_reports).post(reports::create_report),
        )
        .route(
            &format!("{}/reports/{{id}}", api_prefix),
            get(reports::get_report)
                .patch(reports::update_report)
                .delete(reports::delete_report),
        )
        .layer(middleware::from_fn(auth_middleware));
    router.merge(protected)
}
