//! Zones accept their motion-detection settings on create and update (GH #22).
//!
//! Requires the test database — run with:
//!   APP_PROFILE=test-db cargo test --test it_zone_settings -- --include-ignored

mod common;

use axum::http::{Method, StatusCode};
use common::fixtures::{insert_monitor, RowGuard};
use common::harness::{superuser_token, TestApp};
use serde_json::{json, Value};

async fn create_zone(app: &TestApp, monitor_id: u32, body: Value) -> common::harness::TestResponse {
    app.post_json(
        &format!("/api/v3/monitors/{monitor_id}/zones"),
        &superuser_token(),
        &body,
    )
    .await
}

async fn put_zone(app: &TestApp, id: u64, body: Value) -> common::harness::TestResponse {
    app.request(Method::PUT, &format!("/api/v3/zones/{id}"))
        .bearer(&superuser_token())
        .json(&body)
        .send()
        .await
}

fn zone_guard(id: u64) -> RowGuard {
    RowGuard::new(format!("Zones#{id}"), move |db| async move {
        let _ = zm_api::entity::zones::Entity::delete_by_id(id as u32)
            .exec(&db)
            .await;
    })
}

use sea_orm::EntityTrait;

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn create_takes_every_motion_setting() {
    let app = TestApp::spawn().await;
    let monitor = insert_monitor(&app.db, "zone_settings_create")
        .await
        .unwrap();
    let _mon = RowGuard::monitor(monitor.id);

    let resp = create_zone(
        &app,
        monitor.id,
        json!({
            "name": "Driveway", "type": "Inclusive", "units": "Percent",
            "coords": "0,0 50,0 50,50 0,50", "num_coords": 9,
            "check_method": "Blobs", "alarm_rgb": 16711680,
            "min_pixel_threshold": 25, "max_pixel_threshold": 200,
            "min_alarm_pixels": 3.5, "max_alarm_pixels": 75.25,
            "filter_x": 3, "filter_y": 3,
            "min_filter_pixels": 2.0, "max_filter_pixels": 70.0,
            "min_blob_pixels": 1.5, "max_blob_pixels": 60.0,
            "min_blobs": 1, "max_blobs": 10,
            "overload_frames": 5, "extend_alarm_frames": 2
        }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::CREATED, "{}", resp.text());
    let z: Value = resp.json();
    let _zone = zone_guard(z["id"].as_u64().unwrap());
    assert_eq!(z["type"], "Inclusive");
    assert_eq!(z["units"], "Percent");
    assert_eq!(z["check_method"], "Blobs");
    assert_eq!(
        z["num_coords"], 4,
        "derived from coords, not the client's 9"
    );
    assert_eq!(z["min_pixel_threshold"], 25);
    assert_eq!(z["max_alarm_pixels"], 75.25);
    assert_eq!(z["filter_y"], 3);
    assert_eq!(z["min_blob_pixels"], 1.5);
    assert_eq!(z["max_blobs"], 10);
    assert_eq!(z["overload_frames"], 5);
    assert_eq!(z["extend_alarm_frames"], 2);
    assert_eq!(z["alarm_rgb"], 16711680);
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn update_changes_type_units_coords_and_thresholds_and_null_clears() {
    let app = TestApp::spawn().await;
    let monitor = insert_monitor(&app.db, "zone_settings_update")
        .await
        .unwrap();
    let _mon = RowGuard::monitor(monitor.id);
    let created: Value = create_zone(
        &app,
        monitor.id,
        json!({
            "name": "Yard", "type": "Active", "units": "Pixels",
            "coords": "0,0 100,0 100,100 0,100", "num_coords": 4,
            "min_pixel_threshold": 30, "min_alarm_pixels": 10.0
        }),
    )
    .await
    .json();
    let id = created["id"].as_u64().unwrap();
    let _zone = zone_guard(id);
    let area_before = created["area"].as_u64().unwrap();

    let resp = put_zone(
        &app,
        id,
        json!({
            "type": "Exclusive", "units": "Percent",
            "coords": "0,0 40,0 40,40 20,60 0,40",
            "check_method": "FilteredPixels",
            "max_pixel_threshold": 180, "filter_x": 5,
            "min_pixel_threshold": null
        }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK, "{}", resp.text());
    let z: Value = resp.json();
    assert_eq!(z["type"], "Exclusive");
    assert_eq!(z["units"], "Percent");
    assert_eq!(z["check_method"], "FilteredPixels");
    assert_eq!(z["coords"], "0,0 40,0 40,40 20,60 0,40");
    assert_eq!(z["num_coords"], 5);
    assert_ne!(
        z["area"].as_u64().unwrap(),
        area_before,
        "area follows coords"
    );
    assert_eq!(z["max_pixel_threshold"], 180);
    assert_eq!(z["filter_x"], 5);
    assert!(z["min_pixel_threshold"].is_null(), "null clears: {z}");
    assert_eq!(z["min_alarm_pixels"], 10.0, "fields not sent are unchanged");
    assert_eq!(z["name"], "Yard");

    // `polygon`, the old name for coords, still works.
    let old: Value = put_zone(&app, id, json!({"polygon": "0,0 10,0 10,10"}))
        .await
        .json();
    assert_eq!(old["coords"], "0,0 10,0 10,10");
    assert_eq!(old["num_coords"], 3);
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn bad_values_are_rejected_not_silently_defaulted() {
    let app = TestApp::spawn().await;
    let monitor = insert_monitor(&app.db, "zone_settings_bad").await.unwrap();
    let _mon = RowGuard::monitor(monitor.id);

    // Unknown type used to become Active without a word.
    let bad_type = create_zone(&app, monitor.id, json!({
        "name": "Typo", "type": "Activ", "units": "Pixels", "coords": "0,0 9,0 9,9", "num_coords": 3
    })).await;
    assert_eq!(
        bad_type.status(),
        StatusCode::BAD_REQUEST,
        "{}",
        bad_type.text()
    );

    let created: Value = create_zone(&app, monitor.id, json!({
        "name": "Ok", "type": "Active", "units": "Pixels", "coords": "0,0 9,0 9,9", "num_coords": 3,
        "max_alarm_pixels": 50.0
    })).await.json();
    let id = created["id"].as_u64().unwrap();
    let _zone = zone_guard(id);

    for (body, why) in [
        (json!({"units": "Metres"}), "unknown units"),
        (json!({"check_method": "Magic"}), "unknown check method"),
        (
            json!({"min_pixel_threshold": 300}),
            "pixel difference is 0-255",
        ),
        (
            json!({"min_alarm_pixels": -1.0}),
            "decimal columns are unsigned",
        ),
        (
            json!({"min_alarm_pixels": 60.0}),
            "min above the stored max",
        ),
        (json!({"coords": "0,0 1,1"}), "not a polygon"),
    ] {
        let resp = put_zone(&app, id, body).await;
        assert_eq!(
            resp.status(),
            StatusCode::BAD_REQUEST,
            "{why}: {}",
            resp.text()
        );
    }
    // Nothing above changed the zone.
    let get: Value = app
        .get(&format!("/api/v3/zones/{id}"), &superuser_token())
        .await
        .json();
    assert_eq!(get["units"], "Pixels");
    assert!(get["min_alarm_pixels"].is_null());
}
