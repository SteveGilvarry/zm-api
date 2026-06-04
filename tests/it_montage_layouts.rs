//! Integration tests for the Montage Layouts API — happy-path plus error paths.
//!
//! Requires the test database — run with:
//!   APP_PROFILE=test-db cargo test --test it_montage_layouts -- --include-ignored

mod common;

use axum::http::StatusCode;
use common::assertions::{assert_error, assert_status};
use common::fixtures::{unique_name, RowGuard};
use common::harness::{superuser_token, TestApp};
use sea_orm::{ActiveModelTrait, Set};
use serde_json::json;
use zm_api::dto::response::{MontageLayoutResponse, PaginatedMontageLayoutsResponse};

/// Insert a `MontageLayouts` row directly and return its id.
async fn insert_layout(db: &sea_orm::DatabaseConnection, label: &str) -> u32 {
    zm_api::entity::montage_layouts::ActiveModel {
        name: Set(unique_name(label)),
        user_id: Set(0),
        positions: Set(None),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("insert montage layout fixture")
    .id
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn list_montage_layouts_returns_inserted_row() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let id = insert_layout(&app.db, "MlList").await;
    let _guard = RowGuard::montage_layout(id);

    let resp = app
        .get("/api/v3/montage_layouts?page=1&page_size=1000", &token)
        .await;
    assert_status(&resp, StatusCode::OK);
    let body: PaginatedMontageLayoutsResponse = resp.json();
    assert!(
        body.items.iter().any(|m| m.id == id),
        "montage layouts list should contain the fixture row"
    );
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn get_montage_layout_returns_the_row() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let id = insert_layout(&app.db, "MlGet").await;
    let _guard = RowGuard::montage_layout(id);

    let resp = app
        .get(&format!("/api/v3/montage_layouts/{id}"), &token)
        .await;
    assert_status(&resp, StatusCode::OK);
    let body: MontageLayoutResponse = resp.json();
    assert_eq!(body.id, id);
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn get_missing_montage_layout_is_not_found() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    let resp = app.get("/api/v3/montage_layouts/999000111", &token).await;
    assert_error(&resp, StatusCode::NOT_FOUND, "MESSAGE_NOT_FOUND_ERROR");
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn create_then_delete_montage_layout_round_trips() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    let body = json!({
        "name": unique_name("MlRoundTrip"),
        "user_id": 0,
        "positions": "{}",
    });
    let create = app
        .post_json("/api/v3/montage_layouts", &token, &body)
        .await;
    assert!(
        create.status().is_success(),
        "create should succeed; body: {}",
        create.text()
    );
    let created: MontageLayoutResponse = create.json();
    let _guard = RowGuard::montage_layout(created.id);

    let delete = app
        .delete(&format!("/api/v3/montage_layouts/{}", created.id), &token)
        .await;
    assert!(
        delete.status().is_success(),
        "delete should succeed; got {}",
        delete.status()
    );

    let get = app
        .get(&format!("/api/v3/montage_layouts/{}", created.id), &token)
        .await;
    assert_eq!(get.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn create_montage_layout_with_invalid_body_is_rejected() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    // Missing the required `name` and `user_id` fields.
    let resp = app
        .post_json(
            "/api/v3/montage_layouts",
            &token,
            &json!({ "positions": "{}" }),
        )
        .await;
    assert!(
        resp.status().is_client_error(),
        "malformed create body should be a 4xx, got {}; body: {}",
        resp.status(),
        resp.text()
    );
}
