use axum::{http::StatusCode, Router};
use axum_test::TestServer;

use sea_orm::{DatabaseBackend, MockDatabase};

use zm_api::server::state::AppState;

fn mk_filter(id: u32, name: &str) -> zm_api::entity::filters::Model {
    use zm_api::entity::sea_orm_active_enums::EmailFormat;
    zm_api::entity::filters::Model {
        id,
        name: name.into(),
        user_id: Some(1),
        execute_interval: 0,
        query_json: "{}".into(),
        auto_archive: 0,
        auto_unarchive: 0,
        auto_video: 0,
        auto_upload: 0,
        auto_email: 0,
        email_to: None,
        email_subject: None,
        email_body: None,
        email_format: EmailFormat::Summary,
        auto_message: 0,
        auto_execute: 0,
        auto_execute_cmd: None,
        auto_delete: 0,
        auto_move: 0,
        auto_copy: 0,
        auto_copy_to: 0,
        auto_move_to: 0,
        update_disk_space: 0,
        background: 0,
        concurrent: 0,
        lock_rows: 0,
    }
}

fn auth_header() -> String {
    let token = zm_api::service::token::generate_tokens("tester".to_string())
        .expect("token")
        .access_token;
    format!("Bearer {}", token)
}

#[tokio::test]
async fn filters_index_ok() {
    // Arrange DB with two rows
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results::<zm_api::entity::filters::Model, _, _>(vec![vec![
            mk_filter(1, "f1"),
            mk_filter(2, "f2"),
        ]])
        .into_connection();
    let state = AppState::for_test_with_db(db);

    let app = zm_api::routes::filters::add_filter_routes(Router::new()).with_state(state);
    let server = TestServer::new(app.into_make_service()).unwrap();

    // Act
    let res = server
        .get("/api/v3/filters")
        .add_header("Authorization", auth_header())
        .await;

    // Assert
    res.assert_status(StatusCode::OK);
    let items: Vec<zm_api::dto::response::FilterResponse> = res.json();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].name, "f1");
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
