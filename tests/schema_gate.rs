//! A ZoneMinder schema older than this build: the API answers 503 naming
//! the upgrade command, and liveness still answers (#147). No database.

mod common;

use axum::http::StatusCode;
use common::harness::{superuser_token, TestApp};

#[tokio::test]
async fn a_schema_behind_the_build_is_a_503_with_the_fix() {
    let msg = "the ZoneMinder database is at 1.39.22, older than the 1.39.41 this zm-api \
               needs; run `zm-api-db up` to upgrade it, then restart zm-api";
    let app = TestApp::mock_with(|s| s.schema_behind = Some(msg.into()));
    let token = superuser_token();

    let r = app.get("/api/v3/monitors", &token).await;
    assert_eq!(r.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert!(r.text().contains("zm-api-db up"), "{}", r.text());
    // Even without a token: the cause, not a 401.
    let r = app
        .request(axum::http::Method::GET, "/api/v3/events")
        .send()
        .await;
    assert_eq!(r.status(), StatusCode::SERVICE_UNAVAILABLE);

    let r = app
        .request(axum::http::Method::GET, "/api/v3/host/getVersion")
        .send()
        .await;
    assert_ne!(r.status(), StatusCode::SERVICE_UNAVAILABLE, "{}", r.text());
}

#[tokio::test]
async fn a_current_schema_is_not_gated() {
    let app = TestApp::mock();
    let r = app.get("/api/v3/monitors", "not-a-token").await;
    assert_eq!(r.status(), StatusCode::UNAUTHORIZED);
}
