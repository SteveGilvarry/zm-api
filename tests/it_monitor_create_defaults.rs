//! `POST /api/v3/monitors` with only the fields a client cares about: the
//! rest take ZoneMinder's column defaults (#159).
//!
//! Requires the test database:
//!   APP_PROFILE=test-db cargo nextest run --test it_monitor_create_defaults --run-ignored all

mod common;

use axum::http::StatusCode;
use common::fixtures::{unique_name, RowGuard};
use common::harness::{superuser_token, TestApp};
use serde_json::json;
use zm_api::dto::response::MonitorResponse;

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn a_sparse_create_takes_zoneminders_defaults() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let resp = app
        .post_json(
            "/api/v3/monitors",
            &token,
            &json!({ "name": unique_name("SparseCreate"), "type": "Ffmpeg", "go2rtc_enabled": 1 }),
        )
        .await;
    assert!(
        resp.status().is_success(),
        "{}: {}",
        resp.status(),
        resp.text()
    );
    let m: MonitorResponse = resp.json();
    let _g = RowGuard::monitor(m.id);

    // zm_create.sql.in's Monitors defaults, not the onboarding Default.
    assert_eq!(m.pre_event_count, 10);
    assert_eq!(m.save_jpe_gs, 3);
    assert_eq!(m.fps_report_interval, 250);
    assert_eq!(m.brightness, Some(-1));
    assert_eq!(m.event_close_mode, "system");
    // Settable now; it was response-only.
    assert_eq!(m.go2rtc_enabled, 1);
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn a_create_without_a_name_is_rejected() {
    let app = TestApp::spawn().await;
    let resp = app
        .post_json(
            "/api/v3/monitors",
            &superuser_token(),
            &json!({ "type": "Ffmpeg" }),
        )
        .await;
    assert!(resp.status().is_client_error(), "{}", resp.status());
    assert_ne!(resp.status(), StatusCode::NOT_FOUND);
}
