//! Integration tests for the Devices API — happy-path plus error paths.
//!
//! This file is the reference template for per-domain integration coverage
//! using the shared harness (`tests/common`): each route is exercised for its
//! success case, its not-found case and its bad-input case. Other domains
//! (`it_monitors.rs`, `it_events.rs`, …) follow the same shape.
//!
//! Requires the test database — run with:
//!   APP_PROFILE=test-db cargo test --test it_devices -- --include-ignored

mod common;

use axum::http::StatusCode;
use common::assertions::{assert_error, assert_status};
use common::fixtures::{insert_device, unique_name, RowGuard};
use common::harness::{superuser_token, TestApp};
use serde_json::json;
use zm_api::dto::response::{DeviceResponse, PaginatedDevicesResponse};

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn list_devices_returns_inserted_device() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let device = insert_device(&app.db, "ListController")
        .await
        .expect("insert device fixture");
    let _dev = RowGuard::device(device.id);

    let resp = app
        .get("/api/v3/devices?page=1&page_size=1000", &token)
        .await;
    assert_status(&resp, StatusCode::OK);
    let body: PaginatedDevicesResponse = resp.json();
    assert!(
        body.items.iter().any(|d| d.id == device.id),
        "device list should contain the fixture device"
    );
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn get_device_returns_the_device() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let device = insert_device(&app.db, "GetController")
        .await
        .expect("insert device fixture");
    let _dev = RowGuard::device(device.id);

    let resp = app
        .get(&format!("/api/v3/devices/{}", device.id), &token)
        .await;
    assert_status(&resp, StatusCode::OK);
    let body: DeviceResponse = resp.json();
    assert_eq!(body.id, device.id);
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn get_missing_device_is_not_found() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    let resp = app.get("/api/v3/devices/999000111", &token).await;
    assert_eq!(
        resp.status(),
        StatusCode::NOT_FOUND,
        "unknown device id should be 404; body: {}",
        resp.text()
    );
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn create_device_with_invalid_body_is_rejected() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    // Missing the required `type` and `key_string` fields.
    let resp = app
        .post_json("/api/v3/devices", &token, &json!({ "name": "incomplete" }))
        .await;
    assert!(
        resp.status().is_client_error(),
        "malformed create body should be a 4xx, got {}; body: {}",
        resp.status(),
        resp.text()
    );
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn create_then_delete_device_round_trips() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    let body = json!({
        "name": unique_name("RoundTrip"),
        "type": "X10",
        "key_string": "B2",
    });
    let create = app.post_json("/api/v3/devices", &token, &body).await;
    assert!(
        create.status().is_success(),
        "create should succeed; body: {}",
        create.text()
    );
    let created: DeviceResponse = create.json();
    // Safety net: the row is deleted through the API below, but if an
    // assertion before that panics the guard still reclaims it.
    let _dev = RowGuard::device(created.id);

    let delete = app
        .delete(&format!("/api/v3/devices/{}", created.id), &token)
        .await;
    assert!(
        delete.status().is_success(),
        "delete should succeed; got {}",
        delete.status()
    );

    // The device is gone.
    let get = app
        .get(&format!("/api/v3/devices/{}", created.id), &token)
        .await;
    assert_eq!(get.status(), StatusCode::NOT_FOUND);
    // The round-trip already deleted the device; nothing left to clean up.
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn devices_reject_unauthenticated_access() {
    // Mirrors the central auth_failure sweep, kept here as a per-domain guard.
    let app = TestApp::spawn().await;
    let resp = app
        .request(axum::http::Method::GET, "/api/v3/devices")
        .send()
        .await;
    assert_error(&resp, StatusCode::UNAUTHORIZED, "UNAUTHORIZED_ERROR");
}
