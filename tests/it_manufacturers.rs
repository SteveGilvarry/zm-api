//! Integration tests for the Manufacturers API — happy-path plus error paths.
//!
//! Requires the test database — run with:
//!   APP_PROFILE=test-db cargo test --test it_manufacturers -- --include-ignored

mod common;

use axum::http::StatusCode;
use common::assertions::{assert_error, assert_status};
use common::fixtures::{unique_name, RowGuard};
use common::harness::{superuser_token, TestApp};
use sea_orm::{ActiveModelTrait, Set};
use serde_json::json;
use zm_api::dto::response::{ManufacturerResponse, PaginatedManufacturersResponse};

/// Insert a `Manufacturers` row directly and return its id.
async fn insert_manufacturer(db: &sea_orm::DatabaseConnection, label: &str) -> u32 {
    zm_api::entity::manufacturers::ActiveModel {
        name: Set(unique_name(label)),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("insert manufacturer fixture")
    .id
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn list_manufacturers_returns_inserted_row() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let id = insert_manufacturer(&app.db, "MfgList").await;
    let _guard = RowGuard::manufacturer(id);

    let resp = app
        .get("/api/v3/manufacturers?page=1&page_size=1000", &token)
        .await;
    assert_status(&resp, StatusCode::OK);
    let body: PaginatedManufacturersResponse = resp.json();
    assert!(
        body.items.iter().any(|m| m.id == id),
        "manufacturers list should contain the fixture row"
    );
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn get_manufacturer_returns_the_row() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let id = insert_manufacturer(&app.db, "MfgGet").await;
    let _guard = RowGuard::manufacturer(id);

    let resp = app
        .get(&format!("/api/v3/manufacturers/{id}"), &token)
        .await;
    assert_status(&resp, StatusCode::OK);
    let body: ManufacturerResponse = resp.json();
    assert_eq!(body.id, id);
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn get_missing_manufacturer_is_not_found() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    let resp = app.get("/api/v3/manufacturers/999000111", &token).await;
    assert_error(&resp, StatusCode::NOT_FOUND, "MESSAGE_NOT_FOUND_ERROR");
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn create_then_delete_manufacturer_round_trips() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    let body = json!({ "name": unique_name("MfgRoundTrip") });
    let create = app.post_json("/api/v3/manufacturers", &token, &body).await;
    assert!(
        create.status().is_success(),
        "create should succeed; body: {}",
        create.text()
    );
    let created: ManufacturerResponse = create.json();
    let _guard = RowGuard::manufacturer(created.id);

    let delete = app
        .delete(&format!("/api/v3/manufacturers/{}", created.id), &token)
        .await;
    assert!(
        delete.status().is_success(),
        "delete should succeed; got {}",
        delete.status()
    );

    let get = app
        .get(&format!("/api/v3/manufacturers/{}", created.id), &token)
        .await;
    assert_eq!(get.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn create_manufacturer_with_invalid_body_is_rejected() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    // Missing the required `name` field.
    let resp = app
        .post_json("/api/v3/manufacturers", &token, &json!({}))
        .await;
    assert!(
        resp.status().is_client_error(),
        "malformed create body should be a 4xx, got {}; body: {}",
        resp.status(),
        resp.text()
    );
}
