//! A new monitor gets ZoneMinder's full-frame "All" zone, and turning a
//! monitor's Orientation carries its zones round with the picture.
//!
//!   APP_PROFILE=test-db cargo test --test it_monitor_zones -- --include-ignored

mod common;

use common::fixtures::{unique_name, RowGuard};
use common::harness::{superuser_token, TestApp};
use serde_json::{json, Value};
use zm_api::dto::response::MonitorResponse;

fn zones_guard(monitor_id: u32) -> RowGuard {
    RowGuard::new(
        format!("Zones of Monitors#{monitor_id}"),
        move |db| async move {
            use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
            let _ = zm_api::entity::zones::Entity::delete_many()
                .filter(zm_api::entity::zones::Column::MonitorId.eq(monitor_id))
                .exec(&db)
                .await;
        },
    )
}

async fn zones(app: &TestApp, token: &str, monitor_id: u32) -> Vec<Value> {
    let resp = app
        .get(&format!("/api/v3/monitors/{monitor_id}/zones"), token)
        .await;
    assert!(resp.status().is_success(), "{}", resp.text());
    let v: Value = resp.json();
    v["items"].as_array().cloned().unwrap_or_default()
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn a_new_monitor_gets_a_full_frame_zone() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let resp = app
        .post_json(
            "/api/v3/monitors",
            &token,
            &json!({ "name": unique_name("ZoneDefault"), "type": "Ffmpeg",
                     "width": 1920, "height": 1080 }),
        )
        .await;
    assert!(resp.status().is_success(), "{}", resp.text());
    let m: MonitorResponse = resp.json();
    let _z = zones_guard(m.id);
    let _m = RowGuard::monitor(m.id);

    let zs = zones(&app, &token, m.id).await;
    assert_eq!(zs.len(), 1, "{zs:?}");
    let z = &zs[0];
    assert_eq!(z["name"], "All");
    assert_eq!(z["type"], "Active");
    assert_eq!(z["units"], "Percent");
    assert_eq!(
        z["coords"],
        "0.00,0.00 100.00,0.00 100.00,100.00 0.00,100.00"
    );
    // Thresholds are a percentage of the zone (ZoneMinder 1.39.3 on), so
    // they stay under 100 whatever the resolution.
    for k in [
        "min_alarm_pixels",
        "max_alarm_pixels",
        "min_filter_pixels",
        "max_filter_pixels",
        "min_blob_pixels",
    ] {
        let v = z[k].as_f64().unwrap_or_else(|| panic!("{k} missing: {z}"));
        assert!(v > 0.0 && v <= 100.0, "{k} = {v}");
    }
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn rotating_a_monitor_turns_its_zones() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let resp = app
        .post_json(
            "/api/v3/monitors",
            &token,
            &json!({ "name": unique_name("ZoneRotate"), "type": "Ffmpeg",
                     "width": 1920, "height": 1080 }),
        )
        .await;
    assert!(resp.status().is_success(), "{}", resp.text());
    let m: MonitorResponse = resp.json();
    let _z = zones_guard(m.id);
    let _m = RowGuard::monitor(m.id);

    // A zone over the top-left quarter.
    let resp = app
        .post_json(
            &format!("/api/v3/monitors/{}/zones", m.id),
            &token,
            &json!({ "name": "TopLeft", "type": "Active", "units": "Percent",
                     "coords": "0.00,0.00 50.00,0.00 50.00,50.00 0.00,50.00" }),
        )
        .await;
    assert!(resp.status().is_success(), "{}", resp.text());

    // Turned 90° clockwise, the top-left of the scene is at the top right.
    let resp = app
        .patch_json(
            &format!("/api/v3/monitors/{}", m.id),
            &token,
            &json!({ "orientation": "ROTATE_90" }),
        )
        .await;
    assert!(resp.status().is_success(), "{}", resp.text());
    let zs = zones(&app, &token, m.id).await;
    let tl = zs.iter().find(|z| z["name"] == "TopLeft").expect("zone");
    assert_eq!(
        tl["coords"],
        "100.00,0.00 100.00,50.00 50.00,50.00 50.00,0.00"
    );
    let all = zs.iter().find(|z| z["name"] == "All").expect("All zone");
    assert_eq!(
        all["coords"],
        "100.00,0.00 100.00,100.00 0.00,100.00 0.00,0.00"
    );

    // Back to upright puts it where it was.
    let resp = app
        .patch_json(
            &format!("/api/v3/monitors/{}", m.id),
            &token,
            &json!({ "orientation": "ROTATE_0" }),
        )
        .await;
    assert!(resp.status().is_success(), "{}", resp.text());
    let zs = zones(&app, &token, m.id).await;
    let tl = zs.iter().find(|z| z["name"] == "TopLeft").expect("zone");
    assert_eq!(tl["coords"], "0.00,0.00 50.00,0.00 50.00,50.00 0.00,50.00");
}
