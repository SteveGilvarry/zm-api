//! zm-next worker control protocol, zm-api side, against a fake Phase 1
//! worker on a real socket: the hello, describe_plugins, schema validation on
//! validate and save.
//!
//! The worker's messages follow the JSON examples in zm-next
//! `docs/Worker_Control_Protocol.md`; replace them with zm-next's
//! `tests/contract/` transcripts once published.
//!
//!   APP_PROFILE=test-db cargo test --test it_zmnext_control -- --include-ignored

mod common;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use axum::http::{Method, StatusCode};
use common::fixtures::{insert_monitor, RowGuard};
use common::harness::{superuser_token, TestApp};
use common::test_db::get_test_db;
use serde_json::{json, Value};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use zm_api::server::state::AppState;
use zm_api::streaming::source::SourceRouter;

fn frame(msg_type: u8, payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(24 + payload.len());
    out.extend_from_slice(&(20 + payload.len() as u32).to_le_bytes());
    out.extend_from_slice(&[1, msg_type, 2, 0]);
    out.extend_from_slice(&[0u8; 16]);
    out.extend_from_slice(payload);
    out
}

/// Serve one connection as a Phase 1 worker: hello, then answer
/// describe_plugins with a tracker schema. Counts describe_plugins calls.
fn spawn_phase1_worker(
    sock: PathBuf,
    calls: Arc<std::sync::atomic::AtomicU32>,
) -> tokio::task::JoinHandle<()> {
    let listener = tokio::net::UnixListener::bind(&sock).expect("bind fake worker");
    tokio::spawn(async move {
        loop {
            let Ok((mut stream, _)) = listener.accept().await else {
                return;
            };
            let calls = calls.clone();
            tokio::spawn(async move {
                let hello = json!({
                    "protocol": {"canonical": 1, "control": 1},
                    "zm_next": {"version": "0.1.0", "commit": "test"},
                    "state": "running", "pipeline_hash": "sha256:00",
                    "plugins": [
                        {"kind": "decode_detect", "version": "1.0.0", "schema_sha256": "dd01"},
                        {"kind": "tracker", "version": "1.2.0", "schema_sha256": "77e0"}
                    ],
                    "control_peer": true
                });
                if stream
                    .write_all(&frame(0x14, hello.to_string().as_bytes()))
                    .await
                    .is_err()
                {
                    return;
                }
                loop {
                    let mut head = [0u8; 24];
                    if stream.read_exact(&mut head).await.is_err() {
                        return;
                    }
                    let len = u32::from_le_bytes(head[0..4].try_into().unwrap()) as usize - 20;
                    let mut body = vec![0u8; len];
                    if stream.read_exact(&mut body).await.is_err() {
                        return;
                    }
                    let cmd: Value = serde_json::from_slice(&body).unwrap_or(Value::Null);
                    let data = match cmd["cmd"].as_str() {
                        Some("describe_plugins") => {
                            calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                            json!({
                                "decode_detect": {"version": "1.0.0", "schema": {"type": "object",
                                    "properties": {"conf_threshold": {"type": "number", "minimum": 0, "maximum": 1}}}},
                                "tracker": {"version": "1.2.0", "schema": {"type": "object",
                                    "additionalProperties": false,
                                    "properties": {"iou_threshold": {"type": "number", "minimum": 0, "maximum": 1}}}}
                            })
                        }
                        _ => Value::Null,
                    };
                    let resp = json!({"request_id": cmd["request_id"], "ok": data != Value::Null,
                                      "message": if data == Value::Null { "unknown_command" } else { "" },
                                      "data": data.to_string()});
                    if stream
                        .write_all(&frame(0x12, resp.to_string().as_bytes()))
                        .await
                        .is_err()
                    {
                        return;
                    }
                }
            });
        }
    })
}

fn sock_dir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("zmctl_{tag}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

async fn app_with_router(socks: &Path) -> TestApp {
    let db = get_test_db().await.expect("test database");
    let _ = zm_api::client::database::migrate_database(&db).await;
    let mut state = AppState::for_test_with_db(db);
    let mut config = (*state.config).clone();
    config.zmnext.secrets.key_file = socks.join("key");
    state.config = Arc::new(config);
    let router =
        SourceRouter::from_zoneminder_config(zm_api::configure::streaming::ZoneMinderConfig {
            socks_path: socks.to_string_lossy().into_owned(),
            ..Default::default()
        });
    state.source_router = Some(Arc::new(router));
    TestApp::from_state(state)
}

async fn wait_for_hello(app_state_router: &TestApp, monitor_id: u32, token: &str) {
    // The reader starts lazily; validate triggers it. Poll until the hello has
    // been seen (validate reports worker schemas).
    for _ in 0..50 {
        let r = app_state_router
            .post_json(
                &format!("/api/v3/monitors/{monitor_id}/pipeline/validate"),
                token,
                &json!({"plugins": [{"kind": "tracker", "cfg": {}}]}),
            )
            .await;
        let body: Value = r.json();
        if body["checked_against"]
            .as_str()
            .is_some_and(|s| s.contains("zm-next"))
        {
            return;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    panic!("worker hello never arrived");
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn validate_and_save_use_the_workers_schemas() {
    let db = get_test_db().await.unwrap();
    let monitor = insert_monitor(&db, "zmnext_control").await.unwrap();
    let _mon = RowGuard::monitor(monitor.id);
    let dir = sock_dir("schemas");
    let calls = Arc::new(std::sync::atomic::AtomicU32::new(0));
    let worker = spawn_phase1_worker(
        dir.join(format!("stream_{}.sock", monitor.id)),
        calls.clone(),
    );
    let app = app_with_router(&dir).await;
    let token = superuser_token();
    wait_for_hello(&app, monitor.id, &token).await;

    let bad = json!({"plugins": [
        {"id": "detect", "kind": "decode_detect", "cfg": {"conf_threshold": 0.4}, "children": [
            {"id": "track", "kind": "tracker", "cfg": {"iou_threshold": 1.5, "max_age": 30}}
        ]}
    ]});
    let v = app
        .post_json(
            &format!("/api/v3/monitors/{}/pipeline/validate", monitor.id),
            &token,
            &bad,
        )
        .await;
    assert_eq!(v.status(), StatusCode::OK, "{}", v.text());
    let body: Value = v.json();
    assert_eq!(body["valid"], false);
    let paths: Vec<&str> = body["errors"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["path"].as_str().unwrap())
        .collect();
    assert_eq!(
        paths,
        [
            "plugins[0].children[0].cfg.iou_threshold",
            "plugins[0].children[0].cfg.max_age"
        ],
        "{body}"
    );

    // Saving the same graph is refused with the same errors, and nothing is stored.
    let put = app
        .request(
            Method::PUT,
            &format!("/api/v3/monitors/{}/pipeline", monitor.id),
        )
        .bearer(&token)
        .json(&bad)
        .send()
        .await;
    assert_eq!(put.status(), StatusCode::BAD_REQUEST, "{}", put.text());
    let err: Value = put.json();
    assert_eq!(err["kind"], "INVALID_PIPELINE_ERROR");
    assert_eq!(
        err["details"][0][0],
        "plugins[0].children[0].cfg.iou_threshold"
    );
    assert!(
        zm_api::repo::monitor_pipeline::find_by_monitor(&db, monitor.id)
            .await
            .unwrap()
            .is_none()
    );

    // Schemas are cached by sha256: repeated validation doesn't ask again.
    assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 1);

    worker.abort();
    let _ = std::fs::remove_dir_all(&dir);
}

/// No worker (or one without a hello): validation falls back to zm-api's
/// built-in plugin list.
#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn validate_falls_back_to_builtin_kinds_without_a_worker() {
    let db = get_test_db().await.unwrap();
    let monitor = insert_monitor(&db, "zmnext_control_none").await.unwrap();
    let _mon = RowGuard::monitor(monitor.id);
    let dir = sock_dir("none");
    let app = app_with_router(&dir).await;
    let token = superuser_token();

    let ok = app
        .post_json(
            &format!("/api/v3/monitors/{}/pipeline/validate", monitor.id),
            &token,
            &json!({"plugins": [{"kind": "tracker", "cfg": {"iou_threshold": 1.5}}]}),
        )
        .await;
    let body: Value = ok.json();
    assert_eq!(body["checked_against"], "builtin");
    assert_eq!(
        body["valid"], true,
        "builtin list can't check values: {body}"
    );

    let unknown = app
        .post_json(
            &format!("/api/v3/monitors/{}/pipeline/validate", monitor.id),
            &token,
            &json!({"plugins": [{"kind": "output_webrtc"}]}),
        )
        .await;
    let body: Value = unknown.json();
    assert_eq!(body["valid"], false);
    assert!(body["errors"][0]["message"]
        .as_str()
        .unwrap()
        .contains("unknown plugin kind"));
    let _ = std::fs::remove_dir_all(&dir);
}
