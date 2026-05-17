//! Integration tests for the Config API — happy-path plus error paths.
//!
//! Requires the test database — run with:
//!   APP_PROFILE=test-db cargo test --test it_configs -- --include-ignored

mod common;

use axum::http::{Method, StatusCode};
use common::assertions::{assert_error, assert_status};
use common::harness::{superuser_token, TestApp};
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use serde_json::json;
use zm_api::dto::response::{ConfigResponse, PaginatedConfigsResponse};

/// Insert a `Config` row directly and return its name (the primary key).
///
/// `Config.Name` is a short column, so the fixture name is derived from the
/// caller-supplied id rather than the usual long test prefix. `Config.Id` is a
/// `NOT NULL UNIQUE` non-auto-increment column, so it too must be supplied.
/// The insert is idempotent — any row left by a crashed run is cleared first.
async fn insert_config(db: &sea_orm::DatabaseConnection, id: u16) -> String {
    let name = format!("ZM_IT_CFG_{id}");
    delete_config(db, &name).await;
    zm_api::entity::config::ActiveModel {
        id: Set(id),
        name: Set(name.clone()),
        value: Set("initial".to_string()),
        r#type: Set("string".to_string()),
        category: Set("test".to_string()),
        readonly: Set(0),
        private: Set(0),
        system: Set(0),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("insert config fixture");
    name
}

async fn delete_config(db: &sea_orm::DatabaseConnection, name: &str) {
    let _ = zm_api::entity::config::Entity::delete_by_id(name.to_string())
        .exec(db)
        .await;
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn list_configs_returns_inserted_row() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let name = insert_config(&app.db, 60_001).await;

    let resp = app
        .get("/api/v3/configs?page=1&page_size=1000", &token)
        .await;
    assert_status(&resp, StatusCode::OK);
    let body: PaginatedConfigsResponse = resp.json();
    assert!(
        body.items.iter().any(|c| c.name == name),
        "list should contain the fixture config"
    );

    delete_config(&app.db, &name).await;
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn list_config_categories_succeeds() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let name = insert_config(&app.db, 60_002).await;

    let resp = app.get("/api/v3/configs/categories", &token).await;
    assert_status(&resp, StatusCode::OK);
    let body: serde_json::Value = resp.json();
    assert!(body.is_array(), "categories response should be an array");

    delete_config(&app.db, &name).await;
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn get_config_returns_the_row() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let name = insert_config(&app.db, 60_003).await;

    let resp = app.get(&format!("/api/v3/configs/{name}"), &token).await;
    assert_status(&resp, StatusCode::OK);
    let body: ConfigResponse = resp.json();
    assert_eq!(body.name, name);

    delete_config(&app.db, &name).await;
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn get_missing_config_is_not_found() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    let resp = app
        .get("/api/v3/configs/ZM_NO_SUCH_CONFIG_KEY", &token)
        .await;
    assert_error(&resp, StatusCode::NOT_FOUND, "CONFIG_NOT_FOUND_ERROR");
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn update_config_value_round_trips() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let name = insert_config(&app.db, 60_004).await;

    let update = app
        .request(Method::PUT, &format!("/api/v3/configs/{name}"))
        .bearer(&token)
        .json(&json!({ "value": "updated" }))
        .send()
        .await;
    assert_status(&update, StatusCode::OK);
    let updated: ConfigResponse = update.json();
    assert_eq!(updated.value, "updated");

    let get = app.get(&format!("/api/v3/configs/{name}"), &token).await;
    let fetched: ConfigResponse = get.json();
    assert_eq!(fetched.value, "updated");

    delete_config(&app.db, &name).await;
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn update_config_with_invalid_body_is_rejected() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let name = insert_config(&app.db, 60_005).await;

    // Missing the required `value` field.
    let resp = app
        .request(Method::PUT, &format!("/api/v3/configs/{name}"))
        .bearer(&token)
        .json(&json!({}))
        .send()
        .await;
    assert!(
        resp.status().is_client_error(),
        "malformed update body should be a 4xx, got {}; body: {}",
        resp.status(),
        resp.text()
    );

    delete_config(&app.db, &name).await;
}
