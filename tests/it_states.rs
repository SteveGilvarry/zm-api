//! Integration tests for the States API — happy-path plus error paths.
//!
//! Requires the test database — run with:
//!   APP_PROFILE=test-db cargo test --test it_states -- --include-ignored

mod common;

use axum::http::StatusCode;
use common::assertions::{assert_error, assert_status};
use common::fixtures::{unique_name, RowGuard};
use common::harness::{superuser_token, TestApp};
use sea_orm::{ActiveModelTrait, Set};
use serde_json::json;
use zm_api::dto::response::{PaginatedStatesResponse, StateResponse};

/// Insert a `States` row directly and return its id.
async fn insert_state(db: &sea_orm::DatabaseConnection, label: &str) -> u32 {
    zm_api::entity::states::ActiveModel {
        name: Set(unique_name(label)),
        definition: Set("1:Monitor:Modect".to_string()),
        is_active: Set(0),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("insert state fixture")
    .id
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn list_states_returns_inserted_row() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let id = insert_state(&app.db, "StList").await;
    let _guard = RowGuard::state(id);

    let resp = app
        .get("/api/v3/states?page=1&page_size=1000", &token)
        .await;
    assert_status(&resp, StatusCode::OK);
    let body: PaginatedStatesResponse = resp.json();
    assert!(
        body.items.iter().any(|s| s.id == id),
        "states list should contain the fixture row"
    );
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn get_state_returns_the_row() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let id = insert_state(&app.db, "StGet").await;
    let _guard = RowGuard::state(id);

    let resp = app.get(&format!("/api/v3/states/{id}"), &token).await;
    assert_status(&resp, StatusCode::OK);
    let body: StateResponse = resp.json();
    assert_eq!(body.id, id);
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn get_missing_state_is_not_found() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    let resp = app.get("/api/v3/states/999000111", &token).await;
    assert_error(&resp, StatusCode::NOT_FOUND, "MESSAGE_NOT_FOUND_ERROR");
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn create_then_delete_state_round_trips() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    let body = json!({
        "name": unique_name("StRoundTrip"),
        "definition": "1:Monitor:Modect",
        "is_active": 0,
    });
    let create = app.post_json("/api/v3/states", &token, &body).await;
    assert!(
        create.status().is_success(),
        "create should succeed; body: {}",
        create.text()
    );
    let created: StateResponse = create.json();
    let _guard = RowGuard::state(created.id);

    let delete = app
        .delete(&format!("/api/v3/states/{}", created.id), &token)
        .await;
    assert!(
        delete.status().is_success(),
        "delete should succeed; got {}",
        delete.status()
    );

    let get = app
        .get(&format!("/api/v3/states/{}", created.id), &token)
        .await;
    assert_eq!(get.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn create_state_with_invalid_body_is_rejected() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    // Missing the required `definition` and `is_active` fields.
    let resp = app
        .post_json("/api/v3/states", &token, &json!({ "name": "x" }))
        .await;
    assert!(
        resp.status().is_client_error(),
        "malformed create body should be a 4xx, got {}; body: {}",
        resp.status(),
        resp.text()
    );
}
