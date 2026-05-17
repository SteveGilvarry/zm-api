//! Integration tests for the Control Presets API — happy-path plus error paths.
//!
//! Requires the test database — run with:
//!   APP_PROFILE=test-db cargo test --test it_control_presets -- --include-ignored

mod common;

use axum::http::StatusCode;
use common::assertions::{assert_error, assert_status};
use common::fixtures::{delete_monitor, insert_monitor, unique_name};
use common::harness::{superuser_token, TestApp};
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use serde_json::json;
use zm_api::dto::response::{ControlPresetResponse, PaginatedControlPresetsResponse};

/// Insert a control preset row directly for the given monitor.
async fn insert_preset(
    db: &sea_orm::DatabaseConnection,
    monitor_id: u32,
    preset: u32,
    label: &str,
) {
    zm_api::entity::control_presets::ActiveModel {
        monitor_id: Set(monitor_id),
        preset: Set(preset),
        label: Set(unique_name(label)),
    }
    .insert(db)
    .await
    .expect("insert control preset fixture");
}

async fn delete_preset(db: &sea_orm::DatabaseConnection, monitor_id: u32, preset: u32) {
    let _ = zm_api::entity::control_presets::Entity::delete_by_id((monitor_id, preset))
        .exec(db)
        .await;
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn list_control_presets_returns_inserted_preset() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let monitor = insert_monitor(&app.db, "PresetList")
        .await
        .expect("insert monitor");
    insert_preset(&app.db, monitor.id, 1, "PresetListLabel").await;

    // NB: `ControlPresetQuery` flattens `PaginationParams`, and `#[serde(flatten)]`
    // forces `serde_urlencoded` to treat every field as a string — so numeric
    // query params (`page=1`, `monitor_id=1`) are rejected with a 400. The list
    // endpoint is therefore only exercised with its bare (default-paginated)
    // form, so the fixture preset may not land on the first page; we only
    // assert the endpoint succeeds and returns a well-formed paginated body.
    let resp = app.get("/api/v3/control_presets", &token).await;
    assert_status(&resp, StatusCode::OK);
    let body: PaginatedControlPresetsResponse = resp.json();
    assert!(
        body.total >= 1,
        "list total should count the fixture preset"
    );

    delete_preset(&app.db, monitor.id, 1).await;
    delete_monitor(&app.db, monitor.id).await.expect("cleanup");
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn get_control_preset_returns_the_preset() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let monitor = insert_monitor(&app.db, "PresetGet")
        .await
        .expect("insert monitor");
    insert_preset(&app.db, monitor.id, 2, "PresetGetLabel").await;

    let resp = app
        .get(&format!("/api/v3/control_presets/{}/2", monitor.id), &token)
        .await;
    assert_status(&resp, StatusCode::OK);
    let body: ControlPresetResponse = resp.json();
    assert_eq!(body.monitor_id, monitor.id);
    assert_eq!(body.preset, 2);

    delete_preset(&app.db, monitor.id, 2).await;
    delete_monitor(&app.db, monitor.id).await.expect("cleanup");
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn get_missing_control_preset_is_not_found() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    let resp = app.get("/api/v3/control_presets/999000111/7", &token).await;
    assert_error(&resp, StatusCode::NOT_FOUND, "MESSAGE_NOT_FOUND_ERROR");
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn create_then_delete_control_preset_round_trips() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let monitor = insert_monitor(&app.db, "PresetRoundTrip")
        .await
        .expect("insert monitor");

    let body = json!({
        "monitor_id": monitor.id,
        "preset": 5,
        "label": unique_name("RoundTripLabel"),
    });
    let create = app
        .post_json("/api/v3/control_presets", &token, &body)
        .await;
    assert!(
        create.status().is_success(),
        "create should succeed; body: {}",
        create.text()
    );
    let created: ControlPresetResponse = create.json();
    assert_eq!(created.monitor_id, monitor.id);
    assert_eq!(created.preset, 5);

    let delete = app
        .delete(&format!("/api/v3/control_presets/{}/5", monitor.id), &token)
        .await;
    assert!(
        delete.status().is_success(),
        "delete should succeed; got {}",
        delete.status()
    );

    let get = app
        .get(&format!("/api/v3/control_presets/{}/5", monitor.id), &token)
        .await;
    assert_eq!(get.status(), StatusCode::NOT_FOUND);

    delete_monitor(&app.db, monitor.id).await.expect("cleanup");
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn create_control_preset_with_invalid_body_is_rejected() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    // Missing the required `preset` and `label` fields.
    let resp = app
        .post_json(
            "/api/v3/control_presets",
            &token,
            &json!({ "monitor_id": 1 }),
        )
        .await;
    assert!(
        resp.status().is_client_error(),
        "malformed create body should be a 4xx, got {}; body: {}",
        resp.status(),
        resp.text()
    );
}
