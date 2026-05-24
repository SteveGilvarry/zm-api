//! Integration tests for the live-streaming API (`src/handlers/live.rs`).
//!
//! Exercises the non-WebSocket endpoints through the full middleware + routing
//! stack via `oneshot`:
//!   - per-monitor: `/live/{id}/{stats,hls/*}`
//!   - global:      `/live/sessions`, `/live/sources`
//!   - snapshot:    `/monitors/{id}/snapshot`
//!
//! The test `AppState` is built with no streaming services wired (see
//! `AppState::for_test_with_db`), so any request that clears authentication
//! reaches a handler that returns `503 Service Unavailable`. The tests assert
//! on that contract: auth gating yields `401`, and a cleared request never
//! yields `200` for a stream-less monitor.
//!
//! The `/webrtc/ws` WebSocket-upgrade route is intentionally not covered —
//! `oneshot` cannot drive a protocol upgrade.
//!
//! Requires the test database — run with:
//!   APP_PROFILE=test-db cargo test --test it_live -- --include-ignored

mod common;

use axum::http::{Method, StatusCode};
use common::assertions::assert_error;
use common::fixtures::{delete_monitor, insert_monitor};
use common::harness::{superuser_token, TestApp};

/// A monitor id far outside the range any real ZoneMinder row would use.
const MISSING_MONITOR_ID: u32 = 999_000_111;

/// Assert the response is an error status (4xx or 5xx) and explicitly not 200.
fn assert_is_error(resp: &common::harness::TestResponse, ctx: &str) {
    assert!(
        resp.status().is_client_error() || resp.status().is_server_error(),
        "{ctx}: expected a 4xx/5xx error status, got {}; body: {}",
        resp.status(),
        resp.text()
    );
}

// ---------------------------------------------------------------------------
// unauthenticated: no token at all -> 401
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn live_endpoints_reject_unauthenticated_access() {
    let app = TestApp::spawn().await;

    let per_monitor = [
        format!("/api/v3/live/{MISSING_MONITOR_ID}/stats"),
        format!("/api/v3/live/{MISSING_MONITOR_ID}/hls/master.m3u8"),
        format!("/api/v3/live/{MISSING_MONITOR_ID}/hls/live.m3u8"),
        format!("/api/v3/live/{MISSING_MONITOR_ID}/hls/init.mp4"),
        format!("/api/v3/live/{MISSING_MONITOR_ID}/hls/segment_00001.m4s"),
    ];
    for path in &per_monitor {
        let resp = app.request(Method::GET, path).send().await;
        assert_error(&resp, StatusCode::UNAUTHORIZED, "UNAUTHORIZED_ERROR");
    }

    for path in [
        "/api/v3/live/sessions",
        "/api/v3/live/sources",
        &format!("/api/v3/monitors/{MISSING_MONITOR_ID}/snapshot"),
    ] {
        let resp = app.request(Method::GET, path).send().await;
        assert_eq!(
            resp.status(),
            StatusCode::UNAUTHORIZED,
            "unauthenticated {path} should be 401; body: {}",
            resp.text()
        );
    }
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn stop_live_stream_rejects_unauthenticated_access() {
    let app = TestApp::spawn().await;
    let resp = app
        .request(
            Method::DELETE,
            &format!("/api/v3/live/{MISSING_MONITOR_ID}/stop"),
        )
        .send()
        .await;
    assert_error(&resp, StatusCode::UNAUTHORIZED, "UNAUTHORIZED_ERROR");
}

// ---------------------------------------------------------------------------
// global endpoints: authenticated, but streaming is not configured in tests
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn list_live_sessions_reports_service_unavailable() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    // With no `live_coordinator` wired into the test state the handler returns
    // 503. A cleared-auth request must never 401 here.
    let resp = app.get("/api/v3/live/sessions", &token).await;
    assert_eq!(
        resp.status(),
        StatusCode::SERVICE_UNAVAILABLE,
        "sessions without a coordinator should be 503; body: {}",
        resp.text()
    );
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn list_live_sources_reports_service_unavailable() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    let resp = app.get("/api/v3/live/sources", &token).await;
    assert_eq!(
        resp.status(),
        StatusCode::SERVICE_UNAVAILABLE,
        "sources without a coordinator should be 503; body: {}",
        resp.text()
    );
}

// ---------------------------------------------------------------------------
// per-monitor HLS / stats on a stream-less monitor: never 200
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn hls_and_stats_for_streamless_monitor_error() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let monitor = insert_monitor(&app.db, "LiveStreamless")
        .await
        .expect("insert monitor fixture");

    let paths = [
        format!("/api/v3/live/{}/stats", monitor.id),
        format!("/api/v3/live/{}/hls/master.m3u8", monitor.id),
        format!("/api/v3/live/{}/hls/live.m3u8", monitor.id),
        format!("/api/v3/live/{}/hls/init.mp4", monitor.id),
        format!("/api/v3/live/{}/hls/segment_00001.m4s", monitor.id),
    ];
    for path in &paths {
        let resp = app.get(path, &token).await;
        assert_is_error(&resp, path);
        assert_ne!(
            resp.status(),
            StatusCode::OK,
            "{path}: a stream-less monitor must not yield 200"
        );
    }

    delete_monitor(&app.db, monitor.id)
        .await
        .expect("cleanup monitor");
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn invalid_hls_segment_name_is_rejected() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let monitor = insert_monitor(&app.db, "LiveBadSegment")
        .await
        .expect("insert monitor fixture");

    // A garbage segment name fails `parse_segment_sequence` -> 4xx error.
    let resp = app
        .get(
            &format!("/api/v3/live/{}/hls/not-a-segment.txt", monitor.id),
            &token,
        )
        .await;
    assert_is_error(&resp, "invalid segment name");

    delete_monitor(&app.db, monitor.id)
        .await
        .expect("cleanup monitor");
}

// ---------------------------------------------------------------------------
// non-existent monitor: handlers still return an error status, not 200
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn hls_and_stats_for_unknown_monitor_error() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    let paths = [
        format!("/api/v3/live/{MISSING_MONITOR_ID}/stats"),
        format!("/api/v3/live/{MISSING_MONITOR_ID}/hls/master.m3u8"),
        format!("/api/v3/live/{MISSING_MONITOR_ID}/hls/live.m3u8"),
        format!("/api/v3/live/{MISSING_MONITOR_ID}/hls/init.mp4"),
    ];
    for path in &paths {
        let resp = app.get(path, &token).await;
        assert_is_error(&resp, path);
        assert_ne!(resp.status(), StatusCode::OK, "{path}: must not yield 200");
    }
}

// ---------------------------------------------------------------------------
// snapshot endpoint
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn snapshot_for_streamless_monitor_errors() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let monitor = insert_monitor(&app.db, "LiveSnapshot")
        .await
        .expect("insert monitor fixture");

    // No snapshot service is wired in the test state -> the handler errors
    // rather than returning a JPEG.
    let resp = app
        .get(&format!("/api/v3/monitors/{}/snapshot", monitor.id), &token)
        .await;
    assert_is_error(&resp, "snapshot for stream-less monitor");
    assert_ne!(
        resp.status(),
        StatusCode::OK,
        "snapshot without a service must not yield 200"
    );

    delete_monitor(&app.db, monitor.id)
        .await
        .expect("cleanup monitor");
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn snapshot_without_token_is_unauthorized() {
    let app = TestApp::spawn().await;
    let resp = app
        .request(
            Method::GET,
            &format!("/api/v3/monitors/{MISSING_MONITOR_ID}/snapshot"),
        )
        .send()
        .await;
    assert_eq!(
        resp.status(),
        StatusCode::UNAUTHORIZED,
        "unauthenticated snapshot should be 401; body: {}",
        resp.text()
    );
}
