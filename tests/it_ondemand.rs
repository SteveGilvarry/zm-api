//! On-demand zm-next commands over HTTP: `POST /monitors/{id}/snapshot` and
//! `POST /monitors/{id}/describe`.
//!
//! The DB-backed tests cover everything up to the worker: auth, unknown
//! monitor, a monitor that isn't on zm-next, and body validation. The success
//! path needs the fork's `Monitors.UseZmNext` column, which the stock test
//! schema lacks, so it is covered one layer down by
//! `real_worker_answers_snapshot_now`, which drives a real `zm-core` socket.
//!
//!   APP_PROFILE=test-db cargo test --test it_ondemand -- --include-ignored
//!
//! The real-worker test also needs `ZMNEXT_TEST_SOCKS_DIR` pointing at a
//! directory holding `stream_{ZMNEXT_TEST_MONITOR_ID}.sock` (default id 1)
//! served by a zm-core pipeline with `store_snapshot`.

mod common;

use std::sync::Arc;
use std::time::Duration;

use axum::http::{Method, StatusCode};
use common::fixtures::{insert_monitor, RowGuard};
use common::harness::{superuser_token, TestApp};
use common::test_db::get_test_db;
use zm_api::server::state::AppState;

const MISSING_MONITOR_ID: u32 = 999_000_222;

fn paths(id: u32) -> [String; 2] {
    [
        format!("/api/v3/monitors/{id}/snapshot"),
        format!("/api/v3/monitors/{id}/describe"),
    ]
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn ondemand_endpoints_require_a_token() {
    let app = TestApp::spawn().await;
    for path in paths(MISSING_MONITOR_ID) {
        let resp = app.request(Method::POST, &path).send().await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED, "{path}");
    }
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn ondemand_endpoints_404_for_an_unknown_monitor() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    for path in paths(MISSING_MONITOR_ID) {
        let resp = app.request(Method::POST, &path).bearer(&token).send().await;
        assert_eq!(
            resp.status(),
            StatusCode::NOT_FOUND,
            "{path}: {}",
            resp.text()
        );
    }
}

/// The default test config has `[zmnext]` disabled: a clear 409, not a hang.
#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn ondemand_endpoints_409_when_zmnext_is_disabled() {
    let app = TestApp::spawn().await;
    let monitor = insert_monitor(&app.db, "ondemand_off").await.unwrap();
    let _mon = RowGuard::monitor(monitor.id);
    let token = superuser_token();

    for path in paths(monitor.id) {
        let resp = app.request(Method::POST, &path).bearer(&token).send().await;
        assert_eq!(
            resp.status(),
            StatusCode::CONFLICT,
            "{path}: {}",
            resp.text()
        );
        assert!(resp.text().contains("zm-next"), "{path}: {}", resp.text());
    }
}

/// `[zmnext]` on, but this monitor isn't flagged (or the column is absent,
/// which reads as not flagged): still 409, and it must not wait on a socket.
#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn ondemand_endpoints_409_for_a_monitor_not_on_zmnext() {
    let fixture_db = get_test_db().await.unwrap();
    let monitor = insert_monitor(&fixture_db, "ondemand_legacy")
        .await
        .unwrap();
    let _mon = RowGuard::monitor(monitor.id);

    let mut state = AppState::for_test_with_db(get_test_db().await.unwrap());
    let mut config = (*state.config).clone();
    config.zmnext.enabled = true;
    state.config = Arc::new(config);
    let app = TestApp::from_state(state);
    let token = superuser_token();

    for path in paths(monitor.id) {
        let started = std::time::Instant::now();
        let resp = app.request(Method::POST, &path).bearer(&token).send().await;
        assert_eq!(
            resp.status(),
            StatusCode::CONFLICT,
            "{path}: {}",
            resp.text()
        );
        assert!(
            started.elapsed() < Duration::from_secs(5),
            "{path} took too long"
        );
    }
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn describe_rejects_an_overlong_prompt() {
    let app = TestApp::spawn().await;
    let monitor = insert_monitor(&app.db, "ondemand_prompt").await.unwrap();
    let _mon = RowGuard::monitor(monitor.id);

    let resp = app
        .post_json(
            &format!("/api/v3/monitors/{}/describe", monitor.id),
            &superuser_token(),
            &serde_json::json!({ "prompt": "a".repeat(2001) }),
        )
        .await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST, "{}", resp.text());
}

/// Wire compatibility with the real worker: the router sends `snapshot_now`
/// on a real zm-core socket and gets back a JPEG path that exists.
#[tokio::test]
#[ignore = "requires a running zm-core worker (ZMNEXT_TEST_SOCKS_DIR)"]
async fn real_worker_answers_snapshot_now() {
    use zm_api::configure::streaming::ZoneMinderConfig;
    use zm_api::streaming::source::SourceRouter;

    let Ok(socks) = std::env::var("ZMNEXT_TEST_SOCKS_DIR") else {
        eprintln!("ZMNEXT_TEST_SOCKS_DIR not set; skipping");
        return;
    };
    let monitor_id: u32 = std::env::var("ZMNEXT_TEST_MONITOR_ID")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1);
    let router = SourceRouter::from_zoneminder_config(ZoneMinderConfig {
        socks_path: socks,
        ..ZoneMinderConfig::default()
    });

    let detail = router
        .send_command(
            monitor_id,
            serde_json::json!({ "cmd": "snapshot_now" }),
            Duration::from_secs(10),
        )
        .await
        .expect("worker answers snapshot_now");
    assert_eq!(detail["ok"], true);
    assert_eq!(detail["on_demand"], true);
    let path = detail["path"].as_str().expect("path");
    let jpeg = std::fs::read(path).expect("snapshot file exists");
    assert_eq!(&jpeg[..2], &[0xFF, 0xD8], "JPEG SOI marker");

    // A describe on a pipeline whose VLM is unreachable fails with the
    // plugin's error, not a timeout.
    if std::env::var("ZMNEXT_TEST_EXPECT_DESCRIBE_FAILURE").is_ok() {
        let outcome = router
            .send_command(
                monitor_id,
                serde_json::json!({ "cmd": "describe_now" }),
                Duration::from_secs(30),
            )
            .await;
        match outcome {
            Err(zm_api::streaming::source::command::CommandError::Failed(msg)) => {
                assert!(msg.contains("VLM"), "{msg}")
            }
            other => panic!("expected the plugin's failure, got {other:?}"),
        }
    }

    // An unknown command is rejected by zm-core straight away.
    let rejected = router
        .send_command(
            monitor_id,
            serde_json::json!({ "cmd": "no_such_command" }),
            Duration::from_secs(10),
        )
        .await;
    assert!(
        matches!(
            rejected,
            Err(zm_api::streaming::source::command::CommandError::Rejected(ref m)) if m.contains("unknown_command")
        ),
        "{rejected:?}"
    );

    let _ = router.stop_reader(monitor_id).await;
}

/// The worker status endpoint reports a monitor that isn't on zm-next, on a
/// server where zm-api doesn't supervise daemons (the test state has no manager).
#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn zmnext_status_reports_an_unsupervised_legacy_monitor() {
    let app = TestApp::spawn().await;
    let monitor = insert_monitor(&app.db, "zmnext_status").await.unwrap();
    let _mon = RowGuard::monitor(monitor.id);

    let resp = app
        .get(
            &format!("/api/v3/monitors/{}/zmnext", monitor.id),
            &superuser_token(),
        )
        .await;
    assert_eq!(resp.status(), StatusCode::OK, "{}", resp.text());
    let body: serde_json::Value = resp.json();
    assert_eq!(body["monitor_id"], monitor.id);
    assert_eq!(body["use_zmnext"], false);
    assert_eq!(body["supervised"], false);
    assert!(body["worker"].is_null());

    let missing = app
        .get(
            &format!("/api/v3/monitors/{MISSING_MONITOR_ID}/zmnext"),
            &superuser_token(),
        )
        .await;
    assert_eq!(missing.status(), StatusCode::NOT_FOUND);
}
