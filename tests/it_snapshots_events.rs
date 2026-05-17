//! Integration tests for the Snapshots-Events API — happy-path plus error paths.
//!
//! Requires the test database — run with:
//!   APP_PROFILE=test-db cargo test --test it_snapshots_events -- --include-ignored

mod common;

use axum::http::StatusCode;
use common::assertions::{assert_error, assert_status};
use common::fixtures::{delete_monitor, insert_monitor, unique_name};
use common::harness::{superuser_token, TestApp};
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use serde_json::json;
use zm_api::dto::response::{PaginatedSnapshotEventsResponse, SnapshotEventResponse};

/// Insert an `Events` row for a monitor and return its id.
async fn insert_event(db: &sea_orm::DatabaseConnection, monitor_id: u32, label: &str) -> u64 {
    zm_api::entity::events::ActiveModel {
        monitor_id: Set(monitor_id),
        state_id: Set(1),
        name: Set(unique_name(label)),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("insert event fixture")
    .id
}

/// Insert a `Snapshots` row and return its id.
async fn insert_snapshot(db: &sea_orm::DatabaseConnection, label: &str) -> u32 {
    zm_api::entity::snapshots::ActiveModel {
        name: Set(Some(unique_name(label))),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("insert snapshot fixture")
    .id
}

/// Insert a `Snapshots_Events` association directly and return its id.
async fn insert_snapshot_event(
    db: &sea_orm::DatabaseConnection,
    snapshot_id: u32,
    event_id: u64,
) -> u32 {
    zm_api::entity::snapshots_events::ActiveModel {
        snapshot_id: Set(snapshot_id),
        event_id: Set(event_id),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("insert snapshots_events fixture")
    .id
}

async fn delete_snapshot_event_row(db: &sea_orm::DatabaseConnection, id: u32) {
    let _ = zm_api::entity::snapshots_events::Entity::delete_by_id(id)
        .exec(db)
        .await;
}

async fn delete_event(db: &sea_orm::DatabaseConnection, id: u64) {
    let _ = zm_api::entity::events::Entity::delete_by_id(id)
        .exec(db)
        .await;
}

async fn delete_snapshot(db: &sea_orm::DatabaseConnection, id: u32) {
    let _ = zm_api::entity::snapshots::Entity::delete_by_id(id)
        .exec(db)
        .await;
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn list_snapshot_events_returns_inserted_row() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let monitor = insert_monitor(&app.db, "SnapEvtList")
        .await
        .expect("insert monitor");
    let event_id = insert_event(&app.db, monitor.id, "SnapEvtListEvt").await;
    let snapshot_id = insert_snapshot(&app.db, "SnapEvtListSnap").await;
    let id = insert_snapshot_event(&app.db, snapshot_id, event_id).await;

    let resp = app
        .get("/api/v3/snapshots-events?page=1&page_size=1000", &token)
        .await;
    assert_status(&resp, StatusCode::OK);
    let body: PaginatedSnapshotEventsResponse = resp.json();
    assert!(
        body.items.iter().any(|s| s.id == id),
        "list should contain the fixture association"
    );

    delete_snapshot_event_row(&app.db, id).await;
    delete_event(&app.db, event_id).await;
    delete_snapshot(&app.db, snapshot_id).await;
    delete_monitor(&app.db, monitor.id).await.expect("cleanup");
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn get_snapshot_event_returns_the_row() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let monitor = insert_monitor(&app.db, "SnapEvtGet")
        .await
        .expect("insert monitor");
    let event_id = insert_event(&app.db, monitor.id, "SnapEvtGetEvt").await;
    let snapshot_id = insert_snapshot(&app.db, "SnapEvtGetSnap").await;
    let id = insert_snapshot_event(&app.db, snapshot_id, event_id).await;

    let resp = app
        .get(&format!("/api/v3/snapshots-events/{id}"), &token)
        .await;
    assert_status(&resp, StatusCode::OK);
    let body: SnapshotEventResponse = resp.json();
    assert_eq!(body.id, id);
    assert_eq!(body.event_id, event_id);

    delete_snapshot_event_row(&app.db, id).await;
    delete_event(&app.db, event_id).await;
    delete_snapshot(&app.db, snapshot_id).await;
    delete_monitor(&app.db, monitor.id).await.expect("cleanup");
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn get_missing_snapshot_event_is_not_found() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    let resp = app.get("/api/v3/snapshots-events/999000111", &token).await;
    assert_error(&resp, StatusCode::NOT_FOUND, "FILE_NOT_FOUND_ERROR");
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn create_then_delete_snapshot_event_round_trips() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let monitor = insert_monitor(&app.db, "SnapEvtRoundTrip")
        .await
        .expect("insert monitor");
    let event_id = insert_event(&app.db, monitor.id, "SnapEvtRoundTripEvt").await;
    let snapshot_id = insert_snapshot(&app.db, "SnapEvtRoundTripSnap").await;

    let body = json!({ "snapshot_id": snapshot_id, "event_id": event_id });
    let create = app
        .post_json("/api/v3/snapshots-events", &token, &body)
        .await;
    assert!(
        create.status().is_success(),
        "create should succeed; body: {}",
        create.text()
    );
    let created: SnapshotEventResponse = create.json();

    let delete = app
        .delete(&format!("/api/v3/snapshots-events/{}", created.id), &token)
        .await;
    assert!(
        delete.status().is_success(),
        "delete should succeed; got {}",
        delete.status()
    );

    let get = app
        .get(&format!("/api/v3/snapshots-events/{}", created.id), &token)
        .await;
    assert_eq!(get.status(), StatusCode::NOT_FOUND);

    delete_event(&app.db, event_id).await;
    delete_snapshot(&app.db, snapshot_id).await;
    delete_monitor(&app.db, monitor.id).await.expect("cleanup");
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn create_snapshot_event_with_invalid_body_is_rejected() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    // `snapshot_id` must be a number, not a string.
    let resp = app
        .post_json(
            "/api/v3/snapshots-events",
            &token,
            &json!({ "snapshot_id": "not-a-number", "event_id": 1 }),
        )
        .await;
    assert!(
        resp.status().is_client_error(),
        "malformed create body should be a 4xx, got {}; body: {}",
        resp.status(),
        resp.text()
    );
}
