//! The dedicated auth-endpoint rate limiter throttles rapid requests.
//!
//! No database needed: `/auth/refresh` with a bogus (but well-formed) token is
//! rejected at token decode before any query, and the rate-limit layer runs
//! ahead of the handler — so a burst of requests from one peer IP trips the
//! limiter regardless of the handler outcome. The harness injects a fixed
//! `ConnectInfo`, so every request shares one rate-limit bucket.

mod common;

use axum::http::StatusCode;
use common::harness::TestApp;
use serde_json::json;

#[tokio::test]
async fn auth_endpoint_rate_limit_trips_after_burst() {
    let app = TestApp::mock();
    // Long enough to pass the `token` length validation (garde min 30); decode
    // then fails, so each accepted request is a fast 401 with no DB access.
    let body = json!({ "token": "x".repeat(60) });

    let mut statuses = Vec::new();
    // Default config: burst 10, so the first 10 pass the limiter (→ 401) and the
    // rest are throttled. Send comfortably more than the burst.
    for _ in 0..20 {
        let resp = app
            .request(axum::http::Method::POST, "/api/v3/auth/refresh")
            .json(&body)
            .send()
            .await;
        statuses.push(resp.status);
    }

    let too_many = statuses
        .iter()
        .filter(|s| **s == StatusCode::TOO_MANY_REQUESTS)
        .count();
    assert!(
        too_many > 0,
        "expected the auth limiter to reject some requests with 429; got {statuses:?}"
    );

    // The throttling should kick in only after the burst is spent, not on the
    // very first request.
    assert_ne!(
        statuses[0],
        StatusCode::TOO_MANY_REQUESTS,
        "the first request must not be rate limited"
    );
}
