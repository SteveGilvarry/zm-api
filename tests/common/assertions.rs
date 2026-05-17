//! Assertion helpers for integration tests — the single place that knows the
//! shape of the API's error responses.
#![allow(dead_code)]

use axum::http::StatusCode;
use serde::de::DeserializeOwned;
use zm_api::error::AppResponseError;

use super::harness::TestResponse;

/// Assert the response is an error with the expected HTTP status and
/// `AppResponseError.kind` (e.g. `"NOT_FOUND_ERROR"`, `"UNAUTHORIZED_ERROR"`,
/// `"PERMISSION_DENIED_ERROR"`).
pub fn assert_error(resp: &TestResponse, expected_status: StatusCode, expected_kind: &str) {
    assert_eq!(
        resp.status,
        expected_status,
        "unexpected status code; body: {}",
        resp.text()
    );
    let err: AppResponseError = serde_json::from_slice(&resp.body).unwrap_or_else(|e| {
        panic!(
            "error response did not deserialize as AppResponseError: {e}\nbody: {}",
            resp.text()
        )
    });
    assert_eq!(
        err.kind,
        expected_kind,
        "unexpected error kind; body: {}",
        resp.text()
    );
}

/// Assert a specific HTTP status, with the response body included in the
/// failure message for easier debugging.
pub fn assert_status(resp: &TestResponse, expected: StatusCode) {
    assert_eq!(
        resp.status,
        expected,
        "unexpected status code; body: {}",
        resp.text()
    );
}

/// Assert the response is a 2xx success and deserialize its JSON body.
pub fn assert_ok_json<T: DeserializeOwned>(resp: &TestResponse) -> T {
    assert!(
        resp.status.is_success(),
        "expected 2xx success, got {}; body: {}",
        resp.status,
        resp.text()
    );
    resp.json()
}
