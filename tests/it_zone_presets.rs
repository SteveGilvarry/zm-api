//! Integration tests for the Zone Presets API — happy-path plus error paths.
//!
//! Requires the test database — run with:
//!   APP_PROFILE=test-db cargo test --test it_zone_presets -- --include-ignored

mod common;

use axum::http::StatusCode;
use common::assertions::{assert_error, assert_status};
use common::fixtures::{unique_name, RowGuard};
use common::harness::{superuser_token, TestApp};
use sea_orm::{ActiveModelTrait, Set};
use serde_json::json;
use zm_api::dto::response::{PaginatedZonePresetsResponse, ZonePresetResponse};
use zm_api::entity::sea_orm_active_enums::{CheckMethod, Units, ZoneType};

/// Insert a `ZonePresets` row directly and return its id.
async fn insert_zone_preset(db: &sea_orm::DatabaseConnection, label: &str) -> u32 {
    zm_api::entity::zone_presets::ActiveModel {
        name: Set(unique_name(label)),
        r#type: Set(ZoneType::Active),
        units: Set(Units::Pixels),
        check_method: Set(CheckMethod::AlarmedPixels),
        overload_frames: Set(0),
        extend_alarm_frames: Set(0),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("insert zone preset fixture")
    .id
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn list_zone_presets_returns_inserted_row() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let id = insert_zone_preset(&app.db, "ZpList").await;
    let _guard = RowGuard::zone_preset(id);

    let resp = app
        .get("/api/v3/zone-presets?page=1&page_size=1000", &token)
        .await;
    assert_status(&resp, StatusCode::OK);
    let body: PaginatedZonePresetsResponse = resp.json();
    assert!(
        body.items.iter().any(|z| z.id == id),
        "zone presets list should contain the fixture row"
    );
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn get_zone_preset_returns_the_row() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let id = insert_zone_preset(&app.db, "ZpGet").await;
    let _guard = RowGuard::zone_preset(id);

    let resp = app.get(&format!("/api/v3/zone-presets/{id}"), &token).await;
    assert_status(&resp, StatusCode::OK);
    let body: ZonePresetResponse = resp.json();
    assert_eq!(body.id, id);
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn get_missing_zone_preset_is_not_found() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    let resp = app.get("/api/v3/zone-presets/999000111", &token).await;
    assert_error(&resp, StatusCode::NOT_FOUND, "MESSAGE_NOT_FOUND_ERROR");
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn create_then_delete_zone_preset_round_trips() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    let body = json!({
        "name": unique_name("ZpRoundTrip"),
        "type": "Active",
        "units": "Pixels",
        "check_method": "AlarmedPixels",
    });
    let create = app.post_json("/api/v3/zone-presets", &token, &body).await;
    assert!(
        create.status().is_success(),
        "create should succeed; body: {}",
        create.text()
    );
    let created: ZonePresetResponse = create.json();
    let _guard = RowGuard::zone_preset(created.id);

    let delete = app
        .delete(&format!("/api/v3/zone-presets/{}", created.id), &token)
        .await;
    assert!(
        delete.status().is_success(),
        "delete should succeed; got {}",
        delete.status()
    );

    let get = app
        .get(&format!("/api/v3/zone-presets/{}", created.id), &token)
        .await;
    assert_eq!(get.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn create_zone_preset_with_invalid_body_is_rejected() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    // Missing the required `type`, `units`, and `check_method` fields.
    let resp = app
        .post_json("/api/v3/zone-presets", &token, &json!({ "name": "x" }))
        .await;
    assert!(
        resp.status().is_client_error(),
        "malformed create body should be a 4xx, got {}; body: {}",
        resp.status(),
        resp.text()
    );
}
