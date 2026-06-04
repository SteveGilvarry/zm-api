//! Integration tests for the TriggersX10 API — happy-path plus error paths.
//!
//! Requires the test database — run with:
//!   APP_PROFILE=test-db cargo test --test it_triggers_x10 -- --include-ignored

mod common;

use axum::http::StatusCode;
use common::assertions::{assert_error, assert_status};
use common::fixtures::{insert_monitor, RowGuard};
use common::harness::{superuser_token, TestApp};
use sea_orm::{ActiveModelTrait, Set};
use serde_json::json;
use zm_api::dto::response::{PaginatedTriggersX10Response, TriggerX10Response};

/// Insert a `TriggersX10` row for a monitor.
async fn insert_trigger(db: &sea_orm::DatabaseConnection, monitor_id: u32) {
    zm_api::entity::triggers_x10::ActiveModel {
        monitor_id: Set(monitor_id),
        activation: Set(Some("A1".to_string())),
        alarm_input: Set(Some("A2".to_string())),
        alarm_output: Set(Some("A3".to_string())),
    }
    .insert(db)
    .await
    .expect("insert triggers_x10 fixture");
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn list_triggers_x10_returns_inserted_row() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let monitor = insert_monitor(&app.db, "TrigList")
        .await
        .expect("insert monitor");
    let _mon = RowGuard::monitor(monitor.id);
    insert_trigger(&app.db, monitor.id).await;
    let _trig = RowGuard::triggers_x10(monitor.id);

    let resp = app
        .get("/api/v3/triggers_x10?page=1&page_size=1000", &token)
        .await;
    assert_status(&resp, StatusCode::OK);
    let body: PaginatedTriggersX10Response = resp.json();
    assert!(
        body.items.iter().any(|t| t.monitor_id == monitor.id),
        "triggers_x10 list should contain the fixture row"
    );
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn get_trigger_x10_returns_the_row() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let monitor = insert_monitor(&app.db, "TrigGet")
        .await
        .expect("insert monitor");
    let _mon = RowGuard::monitor(monitor.id);
    insert_trigger(&app.db, monitor.id).await;
    let _trig = RowGuard::triggers_x10(monitor.id);

    let resp = app
        .get(&format!("/api/v3/triggers_x10/{}", monitor.id), &token)
        .await;
    assert_status(&resp, StatusCode::OK);
    let body: TriggerX10Response = resp.json();
    assert_eq!(body.monitor_id, monitor.id);
    assert_eq!(body.activation.as_deref(), Some("A1"));
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn get_missing_trigger_x10_is_not_found() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    let resp = app.get("/api/v3/triggers_x10/999000111", &token).await;
    assert_error(&resp, StatusCode::NOT_FOUND, "MESSAGE_NOT_FOUND_ERROR");
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn create_then_delete_trigger_x10_round_trips() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let monitor = insert_monitor(&app.db, "TrigRoundTrip")
        .await
        .expect("insert monitor");
    let _mon = RowGuard::monitor(monitor.id);

    let body = json!({
        "monitor_id": monitor.id,
        "activation": "B1",
        "alarm_input": "B2",
        "alarm_output": "B3",
    });
    let create = app.post_json("/api/v3/triggers_x10", &token, &body).await;
    assert!(
        create.status().is_success(),
        "create should succeed; body: {}",
        create.text()
    );
    let created: TriggerX10Response = create.json();
    assert_eq!(created.monitor_id, monitor.id);
    // Safety net: the row is deleted through the API below, but if an
    // assertion before that panics the guard still reclaims it.
    let _trig = RowGuard::triggers_x10(monitor.id);

    let delete = app
        .delete(&format!("/api/v3/triggers_x10/{}", monitor.id), &token)
        .await;
    assert!(
        delete.status().is_success(),
        "delete should succeed; got {}",
        delete.status()
    );

    let get = app
        .get(&format!("/api/v3/triggers_x10/{}", monitor.id), &token)
        .await;
    assert_eq!(get.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn create_trigger_x10_with_invalid_body_is_rejected() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    // `monitor_id` must be a number, not a string, and is required.
    let resp = app
        .post_json(
            "/api/v3/triggers_x10",
            &token,
            &json!({ "monitor_id": "not-a-number" }),
        )
        .await;
    assert!(
        resp.status().is_client_error(),
        "malformed create body should be a 4xx, got {}; body: {}",
        resp.status(),
        resp.text()
    );
}
