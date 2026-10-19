//! MonitorActions (#179): CRUD limited to what the target's control can do,
//! the targets list, and running only enabled Manual actions.
//!
//! Requires the test database:
//!   APP_PROFILE=test-db cargo nextest run --test it_monitor_actions --run-ignored all

mod common;

use axum::http::StatusCode;
use common::fixtures::{insert_monitor, unique_name, RowGuard};
use common::harness::{superuser_token, token_with, TestApp};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde_json::json;
use zm_api::util::authz::{Level, UserPermissions};

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn actions_are_limited_to_what_the_target_can_do() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    // A speaker: audio (sounds 1-4), no lights.
    let control: serde_json::Value = app
        .post_json(
            "/api/v3/controls",
            &token,
            &json!({ "name": unique_name("ActCtl"), "can_audio_play": 1,
                     "min_audio_file": 1, "max_audio_file": 4 }),
        )
        .await
        .json();
    let control_id = control["id"].as_u64().unwrap() as u32;
    let _c = RowGuard::control(control_id);
    let speaker = insert_monitor(&app.db, "ActSpeaker").await.unwrap();
    let _s = RowGuard::monitor(speaker.id);
    let mut am: zm_api::entity::monitors::ActiveModel = speaker.clone().into();
    am.controllable = Set(1);
    am.control_id = Set(Some(control_id));
    am.update(&app.db).await.unwrap();
    let camera = insert_monitor(&app.db, "ActCamera").await.unwrap();
    let _m = RowGuard::monitor(camera.id);
    let cam = camera.id;
    let _a = RowGuard::new(format!("MonitorActions#{cam}"), move |db| async move {
        use zm_api::entity::monitor_actions as ma;
        let _ = ma::Entity::delete_many()
            .filter(ma::Column::MonitorId.eq(cam))
            .exec(&db)
            .await;
    });
    let base = format!("/api/v3/monitors/{cam}/actions");

    // The speaker is a target, with its types and range; the camera is not.
    let targets: serde_json::Value = app.get(&format!("{base}/targets"), &token).await.json();
    let t = targets
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["id"] == speaker.id)
        .unwrap_or_else(|| panic!("speaker missing from {targets}"));
    assert_eq!(t["types"], json!(["AudioPlay", "AudioStop"]));
    assert_eq!(
        (t["min_audio_file"].clone(), t["max_audio_file"].clone()),
        (json!(1), json!(4))
    );
    assert!(!targets.as_array().unwrap().iter().any(|t| t["id"] == cam));

    // A light on a speaker with none, a sound out of range, a target with no control.
    for bad in [
        json!({ "action_type": "LightOn", "target_monitor_id": speaker.id }),
        json!({ "action_type": "AudioPlay", "target_monitor_id": speaker.id, "audio_file": 9 }),
        json!({ "action_type": "AudioPlay", "target_monitor_id": cam }),
    ] {
        let r = app.post_json(&base, &token, &bad).await;
        assert_eq!(r.status(), StatusCode::BAD_REQUEST, "{bad}: {}", r.text());
    }

    let r = app
        .post_json(
            &base,
            &token,
            &json!({ "action_type": "AudioPlay", "target_monitor_id": speaker.id,
                     "audio_file": 2, "label": "Siren" }),
        )
        .await;
    assert_eq!(r.status(), StatusCode::CREATED, "{}", r.text());
    let created: serde_json::Value = r.json();
    assert_eq!(created["trigger_on"], "EventStart", "ZoneMinder's default");
    assert_eq!(created["enabled"], true);
    let aid = created["id"].as_u64().unwrap();

    let list: serde_json::Value = app.get(&base, &token).await.json();
    assert_eq!(list.as_array().unwrap().len(), 1, "{list}");

    // Only an enabled Manual action runs.
    let run = format!("{base}/{aid}/run");
    let r = app.post_json(&run, &token, &json!({})).await;
    assert_eq!(r.status(), StatusCode::BAD_REQUEST, "{}", r.text());

    // Changing the type is checked against the target too.
    let r = app
        .request(axum::http::Method::PATCH, &format!("{base}/{aid}"))
        .bearer(&token)
        .json(&json!({ "action_type": "IndicatorLightOn" }))
        .send()
        .await;
    assert_eq!(r.status(), StatusCode::BAD_REQUEST, "{}", r.text());
    let r = app
        .request(axum::http::Method::PATCH, &format!("{base}/{aid}"))
        .bearer(&token)
        .json(&json!({ "trigger_on": "Manual", "audio_file": null }))
        .send()
        .await;
    assert_eq!(r.status(), StatusCode::OK, "{}", r.text());
    let patched: serde_json::Value = r.json();
    assert_eq!(patched["trigger_on"], "Manual");
    assert!(patched["audio_file"].is_null());

    // Running needs Control: a Monitors-only user is refused.
    let monitors_only = token_with(UserPermissions {
        monitors: Level::Edit,
        ..Default::default()
    });
    let r = app.post_json(&run, &monitors_only, &json!({})).await;
    assert_eq!(r.status(), StatusCode::FORBIDDEN, "{}", r.text());

    let r = app
        .request(axum::http::Method::DELETE, &format!("{base}/{aid}"))
        .bearer(&token)
        .send()
        .await;
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
    let r = app.get(&format!("{base}/{aid}"), &token).await;
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}
