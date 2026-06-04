//! Integration tests for the Controls API — happy-path plus error paths.
//!
//! Requires the test database — run with:
//!   APP_PROFILE=test-db cargo test --test it_controls -- --include-ignored

mod common;

use axum::http::StatusCode;
use common::assertions::{assert_error, assert_status};
use common::fixtures::{unique_name, RowGuard};
use common::harness::{superuser_token, TestApp};
use sea_orm::{ActiveModelTrait, Set};
use serde_json::json;
use zm_api::dto::response::{ControlResponse, PaginatedControlsResponse};
use zm_api::entity::sea_orm_active_enums::MonitorType;

/// Insert a `Controls` row directly and return its id.
async fn insert_control(db: &sea_orm::DatabaseConnection, label: &str) -> u32 {
    zm_api::entity::controls::ActiveModel {
        name: Set(unique_name(label)),
        r#type: Set(MonitorType::Ffmpeg),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("insert control fixture")
    .id
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn list_controls_returns_inserted_row() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let id = insert_control(&app.db, "ControlList").await;
    let _guard = RowGuard::control(id);

    let resp = app
        .get("/api/v3/controls?page=1&page_size=1000", &token)
        .await;
    assert_status(&resp, StatusCode::OK);
    let body: PaginatedControlsResponse = resp.json();
    assert!(
        body.items.iter().any(|c| c.id == id),
        "list should contain the fixture control"
    );
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn get_control_returns_the_row() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let id = insert_control(&app.db, "ControlGet").await;
    let _guard = RowGuard::control(id);

    let resp = app.get(&format!("/api/v3/controls/{id}"), &token).await;
    assert_status(&resp, StatusCode::OK);
    let body: ControlResponse = resp.json();
    assert_eq!(body.id, id);
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn get_missing_control_is_not_found() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    let resp = app.get("/api/v3/controls/999000111", &token).await;
    assert_error(&resp, StatusCode::NOT_FOUND, "MESSAGE_NOT_FOUND_ERROR");
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn create_then_delete_control_round_trips() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    let body = json!({
        "name": unique_name("ControlRoundTrip"),
        "type": "Ffmpeg",
    });
    let create = app.post_json("/api/v3/controls", &token, &body).await;
    assert!(
        create.status().is_success(),
        "create should succeed; body: {}",
        create.text()
    );
    let created: ControlResponse = create.json();
    let _guard = RowGuard::control(created.id);

    let delete = app
        .delete(&format!("/api/v3/controls/{}", created.id), &token)
        .await;
    assert!(
        delete.status().is_success(),
        "delete should succeed; got {}",
        delete.status()
    );

    let get = app
        .get(&format!("/api/v3/controls/{}", created.id), &token)
        .await;
    assert_eq!(get.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn create_control_with_invalid_body_is_rejected() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    // Missing the required `name` field.
    let resp = app
        .post_json("/api/v3/controls", &token, &json!({ "type": "Ffmpeg" }))
        .await;
    assert!(
        resp.status().is_client_error(),
        "malformed create body should be a 4xx, got {}; body: {}",
        resp.status(),
        resp.text()
    );
}
