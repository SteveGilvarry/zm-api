//! A user who can view a monitor but not edit it doesn't see the camera
//! password inside its URLs; an editor does, since they edit them.
//!
//!   APP_PROFILE=test-db cargo test --test it_monitor_credentials -- --include-ignored

mod common;

use common::fixtures::{unique_name, RowGuard};
use common::harness::{superuser_token, token_for, TestApp};
use serde_json::{json, Value};
use zm_api::util::authz::{Level, UserPermissions};

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn a_viewer_gets_monitor_urls_without_passwords() {
    let app = TestApp::spawn().await;
    let admin = superuser_token();
    let resp = app
        .post_json(
            "/api/v3/monitors",
            &admin,
            &json!({
                "name": unique_name("Creds"), "type": "Ffmpeg",
                "path": "rtsp://admin:hunter2@10.9.9.9:554/s1",
                "control_address": "admin:hunter2@10.9.9.9",
            }),
        )
        .await;
    assert!(resp.status().is_success(), "{}", resp.text());
    let id = resp.json::<Value>()["id"].as_u64().unwrap() as u32;
    let _m = RowGuard::monitor(id);

    let viewer = token_for(
        990_001,
        UserPermissions {
            monitors: Level::View,
            ..Default::default()
        },
    );
    for path in [
        format!("/api/v3/monitors/{id}"),
        "/api/v3/monitors?page_size=1000".to_string(),
    ] {
        let resp = app.get(&path, &viewer).await;
        assert!(resp.status().is_success(), "{path}: {}", resp.text());
        let body = resp.text();
        assert!(
            body.contains(&format!("\"id\":{id}")),
            "{path}: monitor missing"
        );
        assert!(
            !body.contains("hunter2"),
            "{path} leaked a password: {body}"
        );
        assert!(body.contains("rtsp://admin:***@10.9.9.9:554/s1"), "{path}");
    }

    let body = app
        .get(&format!("/api/v3/monitors/{id}"), &admin)
        .await
        .text();
    assert!(
        body.contains("rtsp://admin:hunter2@10.9.9.9:554/s1"),
        "{body}"
    );
}
