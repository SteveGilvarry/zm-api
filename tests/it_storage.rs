//! Integration tests for the Storage API — happy-path plus error paths.
//!
//! Requires the test database — run with:
//!   APP_PROFILE=test-db cargo test --test it_storage -- --include-ignored

mod common;

use axum::http::StatusCode;
use common::assertions::{assert_error, assert_status};
use common::fixtures::{unique_name, RowGuard};
use common::harness::{superuser_token, TestApp};
use sea_orm::{ActiveModelTrait, Set};
use serde_json::json;
use zm_api::dto::response::{PaginatedStorageResponse, StorageResponse};
use zm_api::entity::sea_orm_active_enums::{Scheme, StorageType};

/// Insert a `Storage` row directly and return its id.
async fn insert_storage(db: &sea_orm::DatabaseConnection, label: &str) -> u16 {
    zm_api::entity::storage::ActiveModel {
        name: Set(unique_name(label)),
        path: Set("/var/tmp/zm-test-storage".to_string()),
        r#type: Set(StorageType::Local),
        scheme: Set(Scheme::Medium),
        do_delete: Set(0),
        enabled: Set(1),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("insert storage fixture")
    .id
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn list_storage_returns_inserted_row() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let id = insert_storage(&app.db, "StorageList").await;
    let _guard = RowGuard::storage(id);

    let resp = app
        .get("/api/v3/storage?page=1&page_size=1000", &token)
        .await;
    assert_status(&resp, StatusCode::OK);
    let body: PaginatedStorageResponse = resp.json();
    assert!(
        body.items.iter().any(|s| s.id == id),
        "list should contain the fixture storage"
    );
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn get_storage_returns_the_row() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let id = insert_storage(&app.db, "StorageGet").await;
    let _guard = RowGuard::storage(id);

    let resp = app.get(&format!("/api/v3/storage/{id}"), &token).await;
    assert_status(&resp, StatusCode::OK);
    let body: StorageResponse = resp.json();
    assert_eq!(body.id, id);
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn get_missing_storage_is_not_found() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    let resp = app.get("/api/v3/storage/65000", &token).await;
    assert_error(&resp, StatusCode::NOT_FOUND, "MESSAGE_NOT_FOUND_ERROR");
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn create_then_delete_storage_round_trips() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    let body = json!({
        "name": unique_name("StorageRoundTrip"),
        "path": "/var/tmp/zm-test-storage",
        "type": "local",
        "enabled": 1,
        "scheme": "Medium",
    });
    let create = app.post_json("/api/v3/storage", &token, &body).await;
    assert!(
        create.status().is_success(),
        "create should succeed; body: {}",
        create.text()
    );
    let created: StorageResponse = create.json();
    let _guard = RowGuard::storage(created.id);

    let delete = app
        .delete(&format!("/api/v3/storage/{}", created.id), &token)
        .await;
    assert!(
        delete.status().is_success(),
        "delete should succeed; got {}",
        delete.status()
    );

    let get = app
        .get(&format!("/api/v3/storage/{}", created.id), &token)
        .await;
    assert_eq!(get.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn create_storage_with_invalid_body_is_rejected() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    // Missing the required `path`, `type` and `enabled` fields.
    let resp = app
        .post_json("/api/v3/storage", &token, &json!({ "name": "incomplete" }))
        .await;
    assert!(
        resp.status().is_client_error(),
        "malformed create body should be a 4xx, got {}; body: {}",
        resp.status(),
        resp.text()
    );
}
