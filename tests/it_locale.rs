//! GET /api/v3/system/locale (GH #33): server timezone + date/time formats.
//!
//! Requires the test database — run with:
//!   APP_PROFILE=test-db cargo test --test it_locale -- --include-ignored

mod common;

use axum::http::StatusCode;
use common::assertions::assert_status;
use common::harness::{superuser_token, TestApp};
use zm_api::dto::response::LocaleResponse;

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn locale_returns_offset_and_formats() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    let resp = app.get("/api/v3/system/locale", &token).await;
    assert_status(&resp, StatusCode::OK);
    let body: LocaleResponse = resp.json();

    // Offset always resolves from the process zone (e.g. "+10:00" / "+00:00").
    assert!(
        body.utc_offset.starts_with('+') || body.utc_offset.starts_with('-'),
        "utc_offset should be signed: {}",
        body.utc_offset
    );
    assert_eq!(
        body.utc_offset_seconds % 60,
        0,
        "offset seconds should be whole minutes"
    );
    // Format patterns may be absent on a bare schema; that's allowed (Option).
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn locale_requires_auth() {
    let app = TestApp::spawn().await;
    let resp = app
        .request(axum::http::Method::GET, "/api/v3/system/locale")
        .send()
        .await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}
