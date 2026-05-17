//! Integration tests for the Groups-Monitors API — happy-path plus error paths.
//!
//! Requires the test database — run with:
//!   APP_PROFILE=test-db cargo test --test it_groups_monitors -- --include-ignored

mod common;

use axum::http::StatusCode;
use common::assertions::{assert_error, assert_status};
use common::fixtures::{delete_monitor, insert_monitor, unique_name};
use common::harness::{superuser_token, TestApp};
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use serde_json::json;
use zm_api::dto::response::{GroupMonitorResponse, PaginatedGroupMonitorsResponse};

/// Insert a `Groups` row and return its id.
async fn insert_group(db: &sea_orm::DatabaseConnection, label: &str) -> u32 {
    zm_api::entity::groups::ActiveModel {
        name: Set(unique_name(label)),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("insert group fixture")
    .id
}

/// Insert a `Groups_Monitors` row directly and return its id.
async fn insert_group_monitor(
    db: &sea_orm::DatabaseConnection,
    group_id: u32,
    monitor_id: u32,
) -> u32 {
    zm_api::entity::groups_monitors::ActiveModel {
        group_id: Set(group_id),
        monitor_id: Set(monitor_id),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("insert groups_monitors fixture")
    .id
}

async fn delete_group_monitor_row(db: &sea_orm::DatabaseConnection, id: u32) {
    let _ = zm_api::entity::groups_monitors::Entity::delete_by_id(id)
        .exec(db)
        .await;
}

async fn delete_group(db: &sea_orm::DatabaseConnection, id: u32) {
    let _ = zm_api::entity::groups::Entity::delete_by_id(id)
        .exec(db)
        .await;
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn list_groups_monitors_returns_inserted_row() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let monitor = insert_monitor(&app.db, "GmList")
        .await
        .expect("insert monitor");
    let group_id = insert_group(&app.db, "GmListGroup").await;
    let gm_id = insert_group_monitor(&app.db, group_id, monitor.id).await;

    let resp = app
        .get("/api/v3/groups-monitors?page=1&page_size=1000", &token)
        .await;
    assert_status(&resp, StatusCode::OK);
    let body: PaginatedGroupMonitorsResponse = resp.json();
    assert!(
        body.items.iter().any(|g| g.id == gm_id),
        "groups-monitors list should contain the fixture row"
    );

    delete_group_monitor_row(&app.db, gm_id).await;
    delete_group(&app.db, group_id).await;
    delete_monitor(&app.db, monitor.id).await.expect("cleanup");
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn get_group_monitor_returns_the_row() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let monitor = insert_monitor(&app.db, "GmGet")
        .await
        .expect("insert monitor");
    let group_id = insert_group(&app.db, "GmGetGroup").await;
    let gm_id = insert_group_monitor(&app.db, group_id, monitor.id).await;

    let resp = app
        .get(&format!("/api/v3/groups-monitors/{gm_id}"), &token)
        .await;
    assert_status(&resp, StatusCode::OK);
    let body: GroupMonitorResponse = resp.json();
    assert_eq!(body.id, gm_id);
    assert_eq!(body.group_id, group_id);
    assert_eq!(body.monitor_id, monitor.id);

    delete_group_monitor_row(&app.db, gm_id).await;
    delete_group(&app.db, group_id).await;
    delete_monitor(&app.db, monitor.id).await.expect("cleanup");
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn get_missing_group_monitor_is_not_found() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    let resp = app.get("/api/v3/groups-monitors/999000111", &token).await;
    assert_error(&resp, StatusCode::NOT_FOUND, "FILE_NOT_FOUND_ERROR");
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn create_then_delete_group_monitor_round_trips() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let monitor = insert_monitor(&app.db, "GmRoundTrip")
        .await
        .expect("insert monitor");
    let group_id = insert_group(&app.db, "GmRoundTripGroup").await;

    let body = json!({
        "group_id": group_id,
        "monitor_id": monitor.id,
    });
    let create = app
        .post_json("/api/v3/groups-monitors", &token, &body)
        .await;
    assert!(
        create.status().is_success(),
        "create should succeed; body: {}",
        create.text()
    );
    let created: GroupMonitorResponse = create.json();

    let delete = app
        .delete(&format!("/api/v3/groups-monitors/{}", created.id), &token)
        .await;
    assert!(
        delete.status().is_success(),
        "delete should succeed; got {}",
        delete.status()
    );

    let get = app
        .get(&format!("/api/v3/groups-monitors/{}", created.id), &token)
        .await;
    assert_eq!(get.status(), StatusCode::NOT_FOUND);

    delete_group(&app.db, group_id).await;
    delete_monitor(&app.db, monitor.id).await.expect("cleanup");
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn create_group_monitor_with_invalid_body_is_rejected() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    // Missing the required `monitor_id` field.
    let resp = app
        .post_json("/api/v3/groups-monitors", &token, &json!({ "group_id": 1 }))
        .await;
    assert!(
        resp.status().is_client_error(),
        "malformed create body should be a 4xx, got {}; body: {}",
        resp.status(),
        resp.text()
    );
}
