use axum::{http::StatusCode, Router};
use axum_test::TestServer;

use sea_orm::{DatabaseBackend, MockDatabase};

use zm_api::server::state::AppState;

fn auth_header() -> String {
    let token = zm_api::service::token::generate_tokens("tester".to_string())
        .expect("token")
        .access_token;
    format!("Bearer {}", token)
}

#[tokio::test]
async fn filters_get_not_found_maps_404() {
    // Arrange empty result for get_by_id
    let empty: Vec<zm_api::entity::filters::Model> = vec![];
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results::<zm_api::entity::filters::Model, _, _>(vec![empty])
        .into_connection();
    let state = AppState::for_test_with_db(db);
    let app = zm_api::routes::filters::add_filter_routes(Router::new()).with_state(state);
    let server = TestServer::new(app.into_make_service()).unwrap();
    let res = server
        .get("/api/v3/filters/99")
        .add_header("Authorization", auth_header())
        .await;

    res.assert_status(StatusCode::NOT_FOUND);
}
