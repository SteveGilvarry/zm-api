//! Integration tests for the Stats API — happy-path plus error paths.
//!
//! Requires the test database — run with:
//!   APP_PROFILE=test-db cargo test --test it_stats -- --include-ignored

mod common;

use axum::http::StatusCode;
use common::assertions::{assert_error, assert_status};
use common::fixtures::{insert_monitor, RowGuard};
use common::harness::{superuser_token, TestApp};
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use serde_json::json;
use zm_api::dto::response::{PaginatedStatsResponse, StatResponse};

/// Insert a `Stats` row directly and return its id.
async fn insert_stat(db: &sea_orm::DatabaseConnection, monitor_id: u32) -> u32 {
    zm_api::entity::stats::ActiveModel {
        monitor_id: Set(monitor_id),
        zone_id: Set(1),
        event_id: Set(1),
        frame_id: Set(1),
        pixel_diff: Set(0),
        alarm_pixels: Set(0),
        filter_pixels: Set(0),
        blob_pixels: Set(0),
        blobs: Set(0),
        min_blob_size: Set(0),
        max_blob_size: Set(0),
        min_x: Set(0),
        max_x: Set(0),
        min_y: Set(0),
        max_y: Set(0),
        score: Set(0),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("insert stat fixture")
    .id
}

/// Guard a `Stats` row by id.
fn stat_guard(id: u32) -> RowGuard {
    RowGuard::new(format!("Stats#{id}"), move |db| async move {
        let _ = zm_api::entity::stats::Entity::delete_by_id(id)
            .exec(&db)
            .await;
    })
}

fn stat_body(monitor_id: u32) -> serde_json::Value {
    json!({
        "monitor_id": monitor_id,
        "zone_id": 1,
        "event_id": 1,
        "frame_id": 1,
        "pixel_diff": 0,
        "alarm_pixels": 0,
        "filter_pixels": 0,
        "blob_pixels": 0,
        "blobs": 0,
        "min_blob_size": 0,
        "max_blob_size": 0,
        "min_x": 0,
        "max_x": 0,
        "min_y": 0,
        "max_y": 0,
        "score": 0,
    })
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn list_stats_returns_inserted_row() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let monitor = insert_monitor(&app.db, "StatList")
        .await
        .expect("insert monitor");
    let _mon = RowGuard::monitor(monitor.id);
    let id = insert_stat(&app.db, monitor.id).await;
    let _stat = stat_guard(id);

    let resp = app.get("/api/v3/stats?page=1&page_size=1000", &token).await;
    assert_status(&resp, StatusCode::OK);
    let body: PaginatedStatsResponse = resp.json();
    assert!(
        body.items.iter().any(|s| s.id == id),
        "list should contain the fixture stat"
    );
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn get_stat_returns_the_row() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let monitor = insert_monitor(&app.db, "StatGet")
        .await
        .expect("insert monitor");
    let _mon = RowGuard::monitor(monitor.id);
    let id = insert_stat(&app.db, monitor.id).await;
    let _stat = stat_guard(id);

    let resp = app.get(&format!("/api/v3/stats/{id}"), &token).await;
    assert_status(&resp, StatusCode::OK);
    let body: StatResponse = resp.json();
    assert_eq!(body.id, id);
    assert_eq!(body.monitor_id, monitor.id);
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn get_missing_stat_is_not_found() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    let resp = app.get("/api/v3/stats/999000111", &token).await;
    assert_error(&resp, StatusCode::NOT_FOUND, "MESSAGE_NOT_FOUND_ERROR");
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn create_then_delete_stat_round_trips() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let monitor = insert_monitor(&app.db, "StatRoundTrip")
        .await
        .expect("insert monitor");
    let _mon = RowGuard::monitor(monitor.id);

    let create = app
        .post_json("/api/v3/stats", &token, &stat_body(monitor.id))
        .await;
    assert!(
        create.status().is_success(),
        "create should succeed; body: {}",
        create.text()
    );
    let created: StatResponse = create.json();
    // Safety net: the row is deleted through the API below, but a panic before
    // that still reclaims it.
    let _stat = stat_guard(created.id);

    let delete = app
        .delete(&format!("/api/v3/stats/{}", created.id), &token)
        .await;
    assert!(
        delete.status().is_success(),
        "delete should succeed; got {}",
        delete.status()
    );

    let get = app
        .get(&format!("/api/v3/stats/{}", created.id), &token)
        .await;
    assert_eq!(get.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn create_stat_with_invalid_body_is_rejected() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    // Missing every required field.
    let resp = app
        .post_json("/api/v3/stats", &token, &json!({ "monitor_id": 1 }))
        .await;
    assert!(
        resp.status().is_client_error(),
        "malformed create body should be a 4xx, got {}; body: {}",
        resp.status(),
        resp.text()
    );
}
