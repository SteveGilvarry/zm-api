//! Smoke test for the shared integration-test harness (`tests/common`).
//! Uses the mock-database harness so it runs in the standard `cargo test`.

mod common;

use axum::http::{Method, StatusCode};
use common::assertions::assert_error;
use common::harness::{superuser_token, TestApp};

#[tokio::test]
async fn harness_rejects_unauthenticated_request() {
    let app = TestApp::mock();
    let resp = app.request(Method::GET, "/api/v3/monitors").send().await;
    assert_error(&resp, StatusCode::UNAUTHORIZED, "UNAUTHORIZED_ERROR");
}

#[tokio::test]
async fn harness_superuser_token_clears_auth_and_rbac() {
    let app = TestApp::mock();
    let resp = app.get("/api/v3/monitors", &superuser_token()).await;
    // Past auth + RBAC: the request reached the handler, so it is neither
    // 401 nor 403 (the mock DB may still make the handler itself fail).
    assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
    assert_ne!(resp.status(), StatusCode::FORBIDDEN);
}
