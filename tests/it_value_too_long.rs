//! Over-long values return 400, not 500, on endpoints with no length rule of
//! their own (GH #55).
//!
//! Roughly forty request fields write to fixed-width columns without
//! validating against them, and every one turned an over-long value into
//! `DATABASE_ERROR` / 500 — telling the caller nothing about what was wrong.
//! Per-DTO rules are still better, because they reject before the round trip
//! and can state the limit; this is the net under the ones nobody has reached.
//!
//! Requires the test database — run with:
//!   APP_PROFILE=test-db cargo test --test it_value_too_long -- --include-ignored

mod common;

use axum::http::StatusCode;
use common::harness::{superuser_token, TestApp};
use serde_json::json;

/// `Tags.Name` is varchar(64) and `CreateTagRequest` has no `Validate` derive,
/// so this is exactly the untouched-DTO case the net exists for.
#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn an_over_long_tag_name_is_a_400_naming_the_column() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    let resp = app
        .post_json("/api/v3/tags", &token, &json!({ "name": "t".repeat(500) }))
        .await;

    assert_eq!(
        resp.status(),
        StatusCode::BAD_REQUEST,
        "an over-long value is bad input, not a server fault; body: {}",
        resp.text()
    );

    let body = resp.text();
    assert!(
        body.contains("VALUE_TOO_LONG"),
        "the error kind should be actionable, got: {body}"
    );
    assert!(
        body.contains("Name"),
        "the offending column should be named, got: {body}"
    );
    // The driver's message can carry the value and surrounding SQL. Neither
    // may reach the client — that is what the redaction is for.
    assert!(
        !body.contains("ttttt"),
        "the rejected value must not be echoed back: {body}"
    );
    assert!(
        !body.to_uppercase().contains("INSERT INTO"),
        "no SQL may leak into the response: {body}"
    );
}

/// A value that fits must still succeed — the net must not reject good input.
#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn a_value_within_the_column_still_succeeds() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    let name = format!("zmapi-len-{}", std::process::id());
    let resp = app
        .post_json("/api/v3/tags", &token, &json!({ "name": name }))
        .await;

    assert!(
        resp.status().is_success(),
        "a name well inside varchar(64) must be accepted; body: {}",
        resp.text()
    );
}
