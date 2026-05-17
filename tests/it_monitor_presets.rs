//! Integration tests for the Monitor Presets API — happy-path plus error paths.
//!
//! Requires the test database — run with:
//!   APP_PROFILE=test-db cargo test --test it_monitor_presets -- --include-ignored

mod common;

use axum::http::StatusCode;
use common::assertions::{assert_error, assert_status};
use common::fixtures::unique_name;
use common::harness::{superuser_token, TestApp};
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use serde_json::json;
use zm_api::dto::response::{MonitorPresetResponse, PaginatedMonitorPresetsResponse};
use zm_api::entity::sea_orm_active_enums::MonitorType;

/// Insert a `MonitorPresets` row directly and return its id.
async fn insert_preset(db: &sea_orm::DatabaseConnection, label: &str) -> u32 {
    zm_api::entity::monitor_presets::ActiveModel {
        name: Set(unique_name(label)),
        r#type: Set(MonitorType::Ffmpeg),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("insert monitor preset fixture")
    .id
}

async fn delete_preset(db: &sea_orm::DatabaseConnection, id: u32) {
    let _ = zm_api::entity::monitor_presets::Entity::delete_by_id(id)
        .exec(db)
        .await;
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn list_monitor_presets_returns_inserted_row() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let id = insert_preset(&app.db, "MpList").await;

    let resp = app
        .get("/api/v3/monitor_presets?page=1&page_size=1000", &token)
        .await;
    assert_status(&resp, StatusCode::OK);
    let body: PaginatedMonitorPresetsResponse = resp.json();
    assert!(
        body.items.iter().any(|m| m.id == id),
        "monitor presets list should contain the fixture row"
    );

    delete_preset(&app.db, id).await;
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn get_monitor_preset_returns_the_row() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let id = insert_preset(&app.db, "MpGet").await;

    let resp = app
        .get(&format!("/api/v3/monitor_presets/{id}"), &token)
        .await;
    assert_status(&resp, StatusCode::OK);
    let body: MonitorPresetResponse = resp.json();
    assert_eq!(body.id, id);

    delete_preset(&app.db, id).await;
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn get_missing_monitor_preset_is_not_found() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    let resp = app.get("/api/v3/monitor_presets/999000111", &token).await;
    assert_error(&resp, StatusCode::NOT_FOUND, "MESSAGE_NOT_FOUND_ERROR");
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn create_then_delete_monitor_preset_round_trips() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    let body = json!({
        "name": unique_name("MpRoundTrip"),
        "type": "Ffmpeg",
    });
    let create = app
        .post_json("/api/v3/monitor_presets", &token, &body)
        .await;
    assert!(
        create.status().is_success(),
        "create should succeed; body: {}",
        create.text()
    );
    let created: MonitorPresetResponse = create.json();

    let delete = app
        .delete(&format!("/api/v3/monitor_presets/{}", created.id), &token)
        .await;
    assert!(
        delete.status().is_success(),
        "delete should succeed; got {}",
        delete.status()
    );

    let get = app
        .get(&format!("/api/v3/monitor_presets/{}", created.id), &token)
        .await;
    assert_eq!(get.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn create_monitor_preset_with_invalid_body_is_rejected() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    // Missing the required `name` and `type` fields.
    let resp = app
        .post_json("/api/v3/monitor_presets", &token, &json!({ "device": "x" }))
        .await;
    assert!(
        resp.status().is_client_error(),
        "malformed create body should be a 4xx, got {}; body: {}",
        resp.status(),
        resp.text()
    );
}
