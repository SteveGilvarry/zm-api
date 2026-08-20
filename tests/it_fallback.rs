//! Router fallback: unknown paths return a 404 JSON envelope, not a 500.
//!
//! No database needed — the fallback is reached before any handler. Regression
//! test for GH #17, where the fallback extracted `MatchedPath` (which fails to
//! extract on an unmatched route) and produced a 500 "No matched path found".

mod common;

use axum::http::{Method, StatusCode};
use common::assertions::assert_error;
use common::harness::TestApp;

#[tokio::test]
async fn unknown_route_returns_404_json_envelope() {
    let app = TestApp::mock();
    let resp = app
        .request(Method::GET, "/api/v3/definitely-not-a-route")
        .send()
        .await;
    assert_error(&resp, StatusCode::NOT_FOUND, "NOT_FOUND_ERROR");
}

#[tokio::test]
async fn unknown_route_is_never_500() {
    let app = TestApp::mock();
    for path in [
        "/api/v3/system/version",
        "/api/v3/discovery",
        "/nope",
        "/api/v3/frames/1/image",
    ] {
        let resp = app.request(Method::GET, path).send().await;
        assert_eq!(
            resp.status(),
            StatusCode::NOT_FOUND,
            "{path} should be a 404, got {}",
            resp.status()
        );
    }
}
