use axum::http::{Request, StatusCode, header};
use axum::body::{self, Body};
use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult};
use tower::ServiceExt;
use zm_api::entity::devices::Model as DeviceModel;
use zm_api::entity::sea_orm_active_enums::DeviceType;
use zm_api::dto::response::DeviceResponse;

fn sample_device() -> DeviceModel {
    DeviceModel {
        id: 1,
        name: "X10 Controller".into(),
        r#type: DeviceType::X10,
        key_string: "A1".into(),
    }
}

fn auth_header() -> String {
    let token = zm_api::service::token::generate_tokens("tester".to_string())
        .expect("token")
        .access_token;
    format!("Bearer {}", token)
}

#[tokio::test]
async fn test_list_devices() {
    let devices = vec![sample_device()];
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results::<DeviceModel, _, _>(vec![devices])
        .into_connection();
    let state = zm_api::server::state::AppState::for_test_with_db(db);
    let app = zm_api::routes::create_router_app(state);

    let response = app
        .oneshot(
            Request::get("/api/v3/devices")
                .header(header::AUTHORIZATION, auth_header())
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = body::to_bytes(response.into_body(), 64 * 1024).await.unwrap();
    let body: Vec<DeviceResponse> = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body.len(), 1);
    assert_eq!(body[0].name, "X10 Controller");
}

#[tokio::test]
async fn test_get_device_by_id() {
    let device = sample_device();
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results::<DeviceModel, _, _>(vec![vec![device]])
        .into_connection();
    let state = zm_api::server::state::AppState::for_test_with_db(db);
    let app = zm_api::routes::create_router_app(state);

    let response = app
        .oneshot(
            Request::get("/api/v3/devices/1")
                .header(header::AUTHORIZATION, auth_header())
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = body::to_bytes(response.into_body(), 64 * 1024).await.unwrap();
    let body: DeviceResponse = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body.id, 1);
    assert_eq!(body.name, "X10 Controller");
    assert_eq!(body.key_string, "A1");
}

#[tokio::test]
async fn test_get_device_not_found() {
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results::<DeviceModel, _, _>(vec![vec![]])
        .into_connection();
    let state = zm_api::server::state::AppState::for_test_with_db(db);
    let app = zm_api::routes::create_router_app(state);

    let response = app
        .oneshot(
            Request::get("/api/v3/devices/999")
                .header(header::AUTHORIZATION, auth_header())
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_device() {
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_exec_results(vec![MockExecResult { last_insert_id: 0, rows_affected: 1 }])
        .into_connection();
    let state = zm_api::server::state::AppState::for_test_with_db(db);
    let app = zm_api::routes::create_router_app(state);

    let response = app
        .oneshot(
            Request::delete("/api/v3/devices/1")
                .header(header::AUTHORIZATION, auth_header())
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_delete_device_not_found() {
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_exec_results(vec![MockExecResult { last_insert_id: 0, rows_affected: 0 }])
        .into_connection();
    let state = zm_api::server::state::AppState::for_test_with_db(db);
    let app = zm_api::routes::create_router_app(state);

    let response = app
        .oneshot(
            Request::delete("/api/v3/devices/999")
                .header(header::AUTHORIZATION, auth_header())
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
