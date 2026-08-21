//! Integration coverage for applying a ZoneMinder run state
//! (`POST /api/v3/system/state`) in **passive mode**.
//!
//! The test harness builds the app with no daemon manager (the shipped
//! default: zm_api is a REST API and `zoneminder.service` owns the daemons).
//! Applying a state must write the monitor + state rows and return success
//! WITHOUT attempting a daemon restart — previously it errored with "Daemon
//! manager not available" after the DB was already mutated.
//!
//! Requires the test database — run with:
//!   APP_PROFILE=test-db cargo test --test it_apply_state -- --include-ignored

mod common;

use axum::http::StatusCode;
use common::fixtures::{insert_monitor, unique_name, RowGuard};
use common::harness::{superuser_token, TestApp};
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use serde_json::json;
use zm_api::entity::sea_orm_active_enums::{Analysing, Capturing, Recording};

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn apply_state_reconfigures_monitors_and_activates_in_passive_mode() {
    let app = TestApp::spawn().await; // passive: daemon_manager is None
    let token = superuser_token();

    let monitor = insert_monitor(&app.db, "StateMon")
        .await
        .expect("insert monitor");
    let _mg = RowGuard::monitor(monitor.id);

    // A 4-part run-state definition setting this monitor to Always/Always/OnMotion.
    let state_name = unique_name("Night");
    let definition = format!("{}:Always:Always:OnMotion", monitor.id);
    let state = zm_api::entity::states::ActiveModel {
        name: Set(state_name.clone()),
        definition: Set(definition),
        is_active: Set(0),
        ..Default::default()
    }
    .insert(&app.db)
    .await
    .expect("insert state");
    let _sg = RowGuard::state(state.id);

    let resp = app
        .post_json(
            "/api/v3/system/state",
            &token,
            &json!({ "state_name": state_name }),
        )
        .await;
    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "apply_state must succeed in passive mode (no daemon restart); body: {}",
        resp.text()
    );

    // The monitor's columns now reflect the state definition.
    let m = zm_api::entity::monitors::Entity::find_by_id(monitor.id)
        .one(&app.db)
        .await
        .unwrap()
        .expect("monitor still present");
    assert_eq!(m.capturing, Capturing::Always, "capturing applied");
    assert_eq!(m.analysing, Analysing::Always, "analysing applied");
    assert_eq!(m.recording, Recording::OnMotion, "recording applied");

    // The applied state is marked active.
    let s = zm_api::entity::states::Entity::find_by_id(state.id)
        .one(&app.db)
        .await
        .unwrap()
        .expect("state still present");
    assert_eq!(s.is_active, 1, "the applied state is marked active");
}
