//! Integration tests for the Snapshots-Events API — happy-path plus error paths.
//!
//! Requires the test database — run with:
//!   APP_PROFILE=test-db cargo test --test it_snapshots_events -- --include-ignored

mod common;

use axum::http::StatusCode;
use common::assertions::{assert_error, assert_status};
use common::fixtures::{
    cleanup_monitor_permissions, grant_monitor_permission, insert_monitor, insert_user_with_id,
    unique_name, RowGuard,
};
use common::harness::{superuser_token, token_for, TestApp};
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use serde_json::json;
use zm_api::dto::response::{PaginatedSnapshotEventsResponse, SnapshotEventResponse};
use zm_api::entity::sea_orm_active_enums::Permission;
use zm_api::util::authz::UserPermissions;

/// Insert an `Events` row for a monitor and return its id.
async fn insert_event(db: &sea_orm::DatabaseConnection, monitor_id: u32, label: &str) -> u64 {
    zm_api::entity::events::ActiveModel {
        monitor_id: Set(monitor_id),
        state_id: Set(1),
        name: Set(unique_name(label)),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("insert event fixture")
    .id
}

/// Insert a `Snapshots` row and return its id.
async fn insert_snapshot(db: &sea_orm::DatabaseConnection, label: &str) -> u32 {
    zm_api::entity::snapshots::ActiveModel {
        name: Set(Some(unique_name(label))),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("insert snapshot fixture")
    .id
}

/// Insert a `Snapshots_Events` association directly and return its id.
async fn insert_snapshot_event(
    db: &sea_orm::DatabaseConnection,
    snapshot_id: u32,
    event_id: u64,
) -> u32 {
    zm_api::entity::snapshots_events::ActiveModel {
        snapshot_id: Set(snapshot_id),
        event_id: Set(event_id),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("insert snapshots_events fixture")
    .id
}

/// Guard a `Snapshots_Events` association row (no typed constructor — u32 PK).
fn guard_snapshot_event(id: u32) -> RowGuard {
    RowGuard::new(format!("Snapshots_Events#{id}"), move |db| async move {
        let _ = zm_api::entity::snapshots_events::Entity::delete_by_id(id)
            .exec(&db)
            .await;
    })
}

/// Guard an `Events` row (no typed constructor — u64 PK).
fn guard_event(id: u64) -> RowGuard {
    RowGuard::new(format!("Events#{id}"), move |db| async move {
        let _ = zm_api::entity::events::Entity::delete_by_id(id)
            .exec(&db)
            .await;
    })
}

/// Guard a `Snapshots` row (no typed constructor — u32 PK).
fn guard_snapshot(id: u32) -> RowGuard {
    RowGuard::new(format!("Snapshots#{id}"), move |db| async move {
        let _ = zm_api::entity::snapshots::Entity::delete_by_id(id)
            .exec(&db)
            .await;
    })
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn list_snapshot_events_returns_inserted_row() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let monitor = insert_monitor(&app.db, "SnapEvtList")
        .await
        .expect("insert monitor");
    let _mon = RowGuard::monitor(monitor.id);
    let event_id = insert_event(&app.db, monitor.id, "SnapEvtListEvt").await;
    let _evt = guard_event(event_id);
    let snapshot_id = insert_snapshot(&app.db, "SnapEvtListSnap").await;
    let _snap = guard_snapshot(snapshot_id);
    let id = insert_snapshot_event(&app.db, snapshot_id, event_id).await;
    let _link = guard_snapshot_event(id);

    let resp = app
        .get("/api/v3/snapshots-events?page=1&page_size=1000", &token)
        .await;
    assert_status(&resp, StatusCode::OK);
    let body: PaginatedSnapshotEventsResponse = resp.json();
    assert!(
        body.items.iter().any(|s| s.id == id),
        "list should contain the fixture association"
    );
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn get_snapshot_event_returns_the_row() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let monitor = insert_monitor(&app.db, "SnapEvtGet")
        .await
        .expect("insert monitor");
    let _mon = RowGuard::monitor(monitor.id);
    let event_id = insert_event(&app.db, monitor.id, "SnapEvtGetEvt").await;
    let _evt = guard_event(event_id);
    let snapshot_id = insert_snapshot(&app.db, "SnapEvtGetSnap").await;
    let _snap = guard_snapshot(snapshot_id);
    let id = insert_snapshot_event(&app.db, snapshot_id, event_id).await;
    let _link = guard_snapshot_event(id);

    let resp = app
        .get(&format!("/api/v3/snapshots-events/{id}"), &token)
        .await;
    assert_status(&resp, StatusCode::OK);
    let body: SnapshotEventResponse = resp.json();
    assert_eq!(body.id, id);
    assert_eq!(body.event_id, event_id);
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn get_missing_snapshot_event_is_not_found() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    let resp = app.get("/api/v3/snapshots-events/999000111", &token).await;
    assert_error(&resp, StatusCode::NOT_FOUND, "FILE_NOT_FOUND_ERROR");
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn create_then_delete_snapshot_event_round_trips() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let monitor = insert_monitor(&app.db, "SnapEvtRoundTrip")
        .await
        .expect("insert monitor");
    let _mon = RowGuard::monitor(monitor.id);
    let event_id = insert_event(&app.db, monitor.id, "SnapEvtRoundTripEvt").await;
    let _evt = guard_event(event_id);
    let snapshot_id = insert_snapshot(&app.db, "SnapEvtRoundTripSnap").await;
    let _snap = guard_snapshot(snapshot_id);

    let body = json!({ "snapshot_id": snapshot_id, "event_id": event_id });
    let create = app
        .post_json("/api/v3/snapshots-events", &token, &body)
        .await;
    assert!(
        create.status().is_success(),
        "create should succeed; body: {}",
        create.text()
    );
    let created: SnapshotEventResponse = create.json();
    // Safety net: the row is deleted through the API below, but if an
    // assertion before that panics the guard still reclaims it.
    let _link = guard_snapshot_event(created.id);

    let delete = app
        .delete(&format!("/api/v3/snapshots-events/{}", created.id), &token)
        .await;
    assert!(
        delete.status().is_success(),
        "delete should succeed; got {}",
        delete.status()
    );

    let get = app
        .get(&format!("/api/v3/snapshots-events/{}", created.id), &token)
        .await;
    assert_eq!(get.status(), StatusCode::NOT_FOUND);
}

/// User id used for the row-level ACL regression below. Picked far above
/// the seeded users so it doesn't collide.
const ACL_TEST_UID_SNAP_EVTS: u32 = 9_876_543;

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn snapshot_events_of_hidden_monitors_are_filtered() {
    // A snapshot-event row points at an event, which points at a monitor.
    // A scoped user granted View on monitor A only must:
    //   1. see their own snapshot-event row in /snapshots-events
    //   2. not see the one belonging to monitor B (would leak via the join)
    //   3. get 404 when probing the hidden row by id
    let app = TestApp::spawn().await;

    let monitor_a = insert_monitor(&app.db, "AclSnapEvtA")
        .await
        .expect("insert monitor A");
    let _mon_a = RowGuard::monitor(monitor_a.id);
    let monitor_b = insert_monitor(&app.db, "AclSnapEvtB")
        .await
        .expect("insert monitor B");
    let _mon_b = RowGuard::monitor(monitor_b.id);

    let event_a = insert_event(&app.db, monitor_a.id, "AclSnapEvtAEvt").await;
    let _evt_a = guard_event(event_a);
    let event_b = insert_event(&app.db, monitor_b.id, "AclSnapEvtBEvt").await;
    let _evt_b = guard_event(event_b);
    let snap_a = insert_snapshot(&app.db, "AclSnapEvtASnap").await;
    let _snap_a = guard_snapshot(snap_a);
    let snap_b = insert_snapshot(&app.db, "AclSnapEvtBSnap").await;
    let _snap_b = guard_snapshot(snap_b);
    let link_a = insert_snapshot_event(&app.db, snap_a, event_a).await;
    let _link_a = guard_snapshot_event(link_a);
    let link_b = insert_snapshot_event(&app.db, snap_b, event_b).await;
    let _link_b = guard_snapshot_event(link_b);

    insert_user_with_id(&app.db, ACL_TEST_UID_SNAP_EVTS, "AclSnapEvtUser")
        .await
        .expect("insert acl user");
    let _user = RowGuard::user(ACL_TEST_UID_SNAP_EVTS);
    grant_monitor_permission(
        &app.db,
        monitor_a.id,
        ACL_TEST_UID_SNAP_EVTS,
        Permission::View,
    )
    .await
    .expect("grant permission");
    let token = token_for(ACL_TEST_UID_SNAP_EVTS, UserPermissions::superuser());

    // List: only link_a should be returned.
    let list = app
        .get("/api/v3/snapshots-events?page=1&page_size=1000", &token)
        .await;
    assert_status(&list, StatusCode::OK);
    let body: PaginatedSnapshotEventsResponse = list.json();
    let ids: Vec<u32> = body.items.iter().map(|x| x.id).collect();
    assert!(
        ids.contains(&link_a),
        "permitted monitor's link must appear in list"
    );
    assert!(
        !ids.contains(&link_b),
        "hidden monitor's link must NOT appear in list"
    );

    // Direct probe of the hidden row must 404.
    let hidden_get = app
        .get(&format!("/api/v3/snapshots-events/{link_b}"), &token)
        .await;
    assert_eq!(hidden_get.status(), StatusCode::NOT_FOUND);

    // Direct probe of the permitted row succeeds.
    let visible_get = app
        .get(&format!("/api/v3/snapshots-events/{link_a}"), &token)
        .await;
    assert_status(&visible_get, StatusCode::OK);

    // The `Monitors_Permissions` rows have no typed guard; remove them before
    // the user/monitor guards drop so the foreign keys stay satisfied.
    cleanup_monitor_permissions(&app.db, ACL_TEST_UID_SNAP_EVTS)
        .await
        .expect("cleanup permissions");
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn create_snapshot_event_with_invalid_body_is_rejected() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    // `snapshot_id` must be a number, not a string.
    let resp = app
        .post_json(
            "/api/v3/snapshots-events",
            &token,
            &json!({ "snapshot_id": "not-a-number", "event_id": 1 }),
        )
        .await;
    assert!(
        resp.status().is_client_error(),
        "malformed create body should be a 4xx, got {}; body: {}",
        resp.status(),
        resp.text()
    );
}
