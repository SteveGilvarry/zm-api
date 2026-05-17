//! Integration tests for the event-playback API.
//!
//! Exercises the four media endpoints under `media_auth_middleware`:
//!   - `GET /api/v3/events/{id}/stream/playlist.m3u8`
//!   - `GET /api/v3/events/{id}/stream/video.mp4`
//!   - `GET /api/v3/events/{id}/video`
//!   - `GET /api/v3/events/{id}/thumbnail`
//!
//! Covers not-found, unauthenticated (header + `?token=` query param), and the
//! event-exists-but-media-missing path.
//!
//! Requires the test database — run with:
//!   APP_PROFILE=test-db cargo test --test it_events_playback -- --include-ignored

mod common;

use axum::http::{Method, StatusCode};
use common::fixtures::{delete_monitor, insert_monitor, unique_name};
use common::harness::{superuser_token, TestApp};
use sea_orm::{ActiveModelTrait, EntityTrait, Set};

/// An event id far outside the range any real ZoneMinder row would use.
const MISSING_EVENT_ID: u64 = 999_000_111;

// ---------------------------------------------------------------------------
// not-found: a valid token, but the event does not exist
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn video_for_unknown_event_is_not_found() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    let resp = app
        .get(
            &format!("/api/v3/events/{}/video", MISSING_EVENT_ID),
            &token,
        )
        .await;
    assert_eq!(
        resp.status(),
        StatusCode::NOT_FOUND,
        "unknown event video should be 404; body: {}",
        resp.text()
    );
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn thumbnail_for_unknown_event_is_not_found() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    let resp = app
        .get(
            &format!("/api/v3/events/{}/thumbnail", MISSING_EVENT_ID),
            &token,
        )
        .await;
    assert_eq!(
        resp.status(),
        StatusCode::NOT_FOUND,
        "unknown event thumbnail should be 404; body: {}",
        resp.text()
    );
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn playlist_for_unknown_event_is_not_found() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    let resp = app
        .get(
            &format!("/api/v3/events/{}/stream/playlist.m3u8", MISSING_EVENT_ID),
            &token,
        )
        .await;
    assert_eq!(
        resp.status(),
        StatusCode::NOT_FOUND,
        "unknown event playlist should be 404; body: {}",
        resp.text()
    );
}

// ---------------------------------------------------------------------------
// unauthenticated: no token at all -> 401
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn playback_without_token_is_unauthorized() {
    let app = TestApp::spawn().await;

    for path in [
        format!("/api/v3/events/{}/video", MISSING_EVENT_ID),
        format!("/api/v3/events/{}/thumbnail", MISSING_EVENT_ID),
        format!("/api/v3/events/{}/stream/playlist.m3u8", MISSING_EVENT_ID),
        format!("/api/v3/events/{}/stream/video.mp4", MISSING_EVENT_ID),
    ] {
        let resp = app.request(Method::GET, &path).send().await;
        assert_eq!(
            resp.status(),
            StatusCode::UNAUTHORIZED,
            "unauthenticated request to {path} should be 401; body: {}",
            resp.text()
        );
    }
}

// ---------------------------------------------------------------------------
// query-param auth: `?token=<jwt>` (no header) must get *past* the middleware
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn playback_with_query_param_token_passes_auth() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    // No Authorization header — the token rides in the query string. The
    // middleware must accept it; the handler then 404s on the missing event.
    // A 404 (not 401) proves the request cleared authentication.
    let resp = app
        .request(
            Method::GET,
            &format!("/api/v3/events/{}/video?token={}", MISSING_EVENT_ID, token),
        )
        .send()
        .await;
    assert_eq!(
        resp.status(),
        StatusCode::NOT_FOUND,
        "query-param token should clear auth and then 404 on the missing event; body: {}",
        resp.text()
    );
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn playback_with_invalid_query_param_token_is_unauthorized() {
    let app = TestApp::spawn().await;

    let resp = app
        .request(
            Method::GET,
            &format!("/api/v3/events/{}/video?token=not-a-jwt", MISSING_EVENT_ID),
        )
        .send()
        .await;
    assert_eq!(
        resp.status(),
        StatusCode::UNAUTHORIZED,
        "a garbage query-param token must be rejected; body: {}",
        resp.text()
    );
}

// ---------------------------------------------------------------------------
// missing media file: the event row exists, but no file is on disk
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn video_for_event_with_no_media_file_errors() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    let monitor = insert_monitor(&app.db, "PlaybackNoMedia")
        .await
        .expect("insert monitor fixture");

    // An event with no `storage_id` and no on-disk directory: path resolution
    // falls back to the config events dir and finds nothing.
    let event = zm_api::entity::events::ActiveModel {
        monitor_id: Set(monitor.id),
        state_id: Set(1),
        name: Set(unique_name("PlaybackNoMedia")),
        ..Default::default()
    }
    .insert(&app.db)
    .await
    .expect("insert event fixture");

    let resp = app
        .get(&format!("/api/v3/events/{}/video", event.id), &token)
        .await;

    // The event exists but its media file does not — the handler must return
    // an error status, not a 200 and not a panic.
    assert!(
        resp.status().is_client_error() || resp.status().is_server_error(),
        "missing media file should yield a 4xx/5xx, got {}; body: {}",
        resp.status(),
        resp.text()
    );

    // Cleanup: event first (FK to monitor), then the monitor.
    let _ = zm_api::entity::events::Entity::delete_by_id(event.id)
        .exec(&app.db)
        .await;
    delete_monitor(&app.db, monitor.id)
        .await
        .expect("cleanup monitor");
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn thumbnail_for_event_with_no_media_file_errors() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    let monitor = insert_monitor(&app.db, "PlaybackNoThumb")
        .await
        .expect("insert monitor fixture");

    let event = zm_api::entity::events::ActiveModel {
        monitor_id: Set(monitor.id),
        state_id: Set(1),
        name: Set(unique_name("PlaybackNoThumb")),
        ..Default::default()
    }
    .insert(&app.db)
    .await
    .expect("insert event fixture");

    let resp = app
        .get(&format!("/api/v3/events/{}/thumbnail", event.id), &token)
        .await;

    // No snapshot/capture image on disk -> handler returns 404.
    assert_eq!(
        resp.status(),
        StatusCode::NOT_FOUND,
        "missing thumbnail should be 404, got {}; body: {}",
        resp.status(),
        resp.text()
    );

    let _ = zm_api::entity::events::Entity::delete_by_id(event.id)
        .exec(&app.db)
        .await;
    delete_monitor(&app.db, monitor.id)
        .await
        .expect("cleanup monitor");
}
