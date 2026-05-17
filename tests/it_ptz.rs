//! Integration tests for the PTZ API.
//!
//! PTZ movement endpoints actuate real camera hardware, so this suite covers
//! only the hardware-independent surface: the static protocol list and the
//! not-found paths for status/capabilities on a missing monitor.
//!
//! Requires the test database — run with:
//!   APP_PROFILE=test-db cargo test --test it_ptz -- --include-ignored

mod common;

use axum::http::StatusCode;
use common::assertions::{assert_error, assert_status};
use common::harness::{superuser_token, TestApp};

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn list_ptz_protocols_succeeds() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    let resp = app.get("/api/v3/ptz/protocols", &token).await;
    assert_status(&resp, StatusCode::OK);
    let body: serde_json::Value = resp.json();
    assert!(
        body.get("protocols").map(|p| p.is_array()).unwrap_or(false),
        "protocols response should carry a `protocols` array; body: {}",
        resp.text()
    );
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn get_status_for_missing_monitor_is_not_found() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    let resp = app
        .get("/api/v3/ptz/monitors/999000111/status", &token)
        .await;
    assert_error(&resp, StatusCode::NOT_FOUND, "MONITOR_NOT_FOUND_ERROR");
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn get_capabilities_for_missing_monitor_is_not_found() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    let resp = app
        .get("/api/v3/ptz/monitors/999000111/capabilities", &token)
        .await;
    assert_error(&resp, StatusCode::NOT_FOUND, "MONITOR_NOT_FOUND_ERROR");
}
