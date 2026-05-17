//! Integration tests for the Models API — happy-path plus error paths.
//!
//! Requires the test database — run with:
//!   APP_PROFILE=test-db cargo test --test it_models -- --include-ignored

mod common;

use axum::http::StatusCode;
use common::assertions::{assert_error, assert_status};
use common::fixtures::unique_name;
use common::harness::{superuser_token, TestApp};
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use serde_json::json;
use zm_api::dto::response::{ModelResponse, PaginatedModelsResponse};

/// Insert a `Manufacturers` row to satisfy the optional `manufacturer_id` FK.
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

/// Insert a `Models` row directly and return its id.
async fn insert_model(db: &sea_orm::DatabaseConnection, label: &str) -> u32 {
    zm_api::entity::models::ActiveModel {
        name: Set(unique_name(label)),
        manufacturer_id: Set(None),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("insert model fixture")
    .id
}

async fn delete_model(db: &sea_orm::DatabaseConnection, id: u32) {
    let _ = zm_api::entity::models::Entity::delete_by_id(id)
        .exec(db)
        .await;
}

async fn delete_manufacturer(db: &sea_orm::DatabaseConnection, id: u32) {
    let _ = zm_api::entity::manufacturers::Entity::delete_by_id(id)
        .exec(db)
        .await;
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn list_models_returns_inserted_row() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let id = insert_model(&app.db, "ModelList").await;

    // `ModelListQuery` flattens `PaginationParams`; the handler uses
    // `axum_extra`'s Query so numeric/flattened query params deserialize
    // correctly. A large page size guarantees the fixture lands on the page.
    let resp = app
        .get("/api/v3/models?page=1&page_size=1000", &token)
        .await;
    assert_status(&resp, StatusCode::OK);
    let body: PaginatedModelsResponse = resp.json();
    assert!(
        body.items.iter().any(|m| m.id == id),
        "list should contain the fixture model"
    );

    delete_model(&app.db, id).await;
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn get_model_returns_the_row() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let id = insert_model(&app.db, "ModelGet").await;

    let resp = app.get(&format!("/api/v3/models/{id}"), &token).await;
    assert_status(&resp, StatusCode::OK);
    let body: ModelResponse = resp.json();
    assert_eq!(body.id, id);

    delete_model(&app.db, id).await;
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn get_missing_model_is_not_found() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    let resp = app.get("/api/v3/models/999000111", &token).await;
    assert_error(&resp, StatusCode::NOT_FOUND, "MESSAGE_NOT_FOUND_ERROR");
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn create_then_delete_model_round_trips() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let manufacturer_id = insert_manufacturer(&app.db, "ModelRoundTripMfg").await;

    let body = json!({
        "name": unique_name("ModelRoundTrip"),
        "manufacturer_id": manufacturer_id,
    });
    let create = app.post_json("/api/v3/models", &token, &body).await;
    assert!(
        create.status().is_success(),
        "create should succeed; body: {}",
        create.text()
    );
    let created: ModelResponse = create.json();

    let delete = app
        .delete(&format!("/api/v3/models/{}", created.id), &token)
        .await;
    assert!(
        delete.status().is_success(),
        "delete should succeed; got {}",
        delete.status()
    );

    let get = app
        .get(&format!("/api/v3/models/{}", created.id), &token)
        .await;
    assert_eq!(get.status(), StatusCode::NOT_FOUND);

    delete_manufacturer(&app.db, manufacturer_id).await;
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn create_model_with_invalid_body_is_rejected() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    // Missing the required `name` field.
    let resp = app
        .post_json("/api/v3/models", &token, &json!({ "manufacturer_id": 1 }))
        .await;
    assert!(
        resp.status().is_client_error(),
        "malformed create body should be a 4xx, got {}; body: {}",
        resp.status(),
        resp.text()
    );
}
