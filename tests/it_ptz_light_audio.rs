//! PTZ light and audio commands (#190): gated on the control's capabilities,
//! the audio file checked against its range, and advertised in capabilities.
//!
//! Requires the test database:
//!   APP_PROFILE=test-db cargo nextest run --test it_ptz_light_audio --run-ignored all

mod common;

use axum::http::StatusCode;
use common::fixtures::{insert_monitor, unique_name, RowGuard};
use common::harness::{superuser_token, TestApp};
use sea_orm::{ActiveModelTrait, Set};
use serde_json::json;

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn light_and_audio_follow_the_controls_capabilities() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    // A control with audio (sounds 1-4) and no lights.
    let control: serde_json::Value = app
        .post_json(
            "/api/v3/controls",
            &token,
            &json!({ "name": unique_name("AudioCtl"), "can_audio_play": 1,
                     "min_audio_file": 1, "max_audio_file": 4 }),
        )
        .await
        .json();
    let control_id = control["id"].as_u64().unwrap() as u32;
    let _c = RowGuard::control(control_id);
    let m = insert_monitor(&app.db, "AudioPtz").await.unwrap();
    let _m = RowGuard::monitor(m.id);
    let mut am: zm_api::entity::monitors::ActiveModel = m.clone().into();
    am.controllable = Set(1);
    am.control_id = Set(Some(control_id));
    am.update(&app.db).await.unwrap();
    let base = format!("/api/v3/ptz/monitors/{}", m.id);

    let caps: serde_json::Value = app
        .get(&format!("{base}/capabilities"), &token)
        .await
        .json();
    assert_eq!(caps["audio"]["can_play"], true, "{caps}");
    assert_eq!(
        (
            caps["audio"]["min_file"].clone(),
            caps["audio"]["max_file"].clone()
        ),
        (json!(1), json!(4))
    );
    assert_eq!(caps["light"]["can_light"], false);

    // No light on this control.
    let r = app
        .post_json(&format!("{base}/light/on"), &token, &json!({}))
        .await;
    assert_eq!(r.status(), StatusCode::BAD_REQUEST, "{}", r.text());
    // Out of the control's range.
    let r = app
        .post_json(&format!("{base}/audio/play/9"), &token, &json!({}))
        .await;
    assert_eq!(r.status(), StatusCode::BAD_REQUEST, "{}", r.text());
    // No volume control.
    let r = app
        .post_json(&format!("{base}/audio/volume/up"), &token, &json!({}))
        .await;
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
    // No light, so nothing to ask: both null.
    let r = app.get(&format!("{base}/light"), &token).await;
    assert_eq!(r.status(), StatusCode::OK, "{}", r.text());
    let body: serde_json::Value = r.json();
    assert!(body["light"].is_null() && body["indicator_light"].is_null());
}
