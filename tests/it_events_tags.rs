//! Integration tests for the Events-Tags API — happy-path plus error paths.
//!
//! Requires the test database — run with:
//!   APP_PROFILE=test-db cargo test --test it_events_tags -- --include-ignored

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
use zm_api::dto::response::{EventTagResponse, PaginatedEventsTagsResponse};
use zm_api::entity::sea_orm_active_enums::Permission;
use zm_api::util::authz::UserPermissions;

/// User ids for the ACL cases, chosen to avoid colliding with real users.
const ETAG_ACL_UID: u32 = 990_401;

fn monitor_permissions_guard(user_id: u32) -> RowGuard {
    RowGuard::new(
        format!("Monitors_Permissions(user={user_id})"),
        move |db| async move {
            let _ = cleanup_monitor_permissions(&db, user_id).await;
        },
    )
}

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

/// Insert a `Tags` row and return its id.
async fn insert_tag(db: &sea_orm::DatabaseConnection, label: &str) -> u64 {
    zm_api::entity::tags::ActiveModel {
        name: Set(unique_name(label)),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("insert tag fixture")
    .id
}

/// Insert an `Events_Tags` association directly.
async fn insert_event_tag(db: &sea_orm::DatabaseConnection, tag_id: u64, event_id: u64) {
    zm_api::entity::events_tags::ActiveModel {
        tag_id: Set(tag_id),
        event_id: Set(event_id),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("insert events_tags fixture");
}

/// Guard an `Events` row (no typed constructor — u64 PK).
fn guard_event(id: u64) -> RowGuard {
    RowGuard::new(format!("Events#{id}"), move |db| async move {
        let _ = zm_api::entity::events::Entity::delete_by_id(id)
            .exec(&db)
            .await;
    })
}

/// Guard an `Events_Tags` association row (composite PK: tag_id + event_id).
fn guard_event_tag(tag_id: u64, event_id: u64) -> RowGuard {
    RowGuard::new(
        format!("Events_Tags#({tag_id},{event_id})"),
        move |db| async move {
            let _ = zm_api::entity::events_tags::Entity::delete_by_id((tag_id, event_id))
                .exec(&db)
                .await;
        },
    )
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn list_events_tags_returns_inserted_row() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let monitor = insert_monitor(&app.db, "EvtTagList")
        .await
        .expect("insert monitor");
    let _mon = RowGuard::monitor(monitor.id);
    let event_id = insert_event(&app.db, monitor.id, "EvtTagListEvt").await;
    let _evt = guard_event(event_id);
    let tag_id = insert_tag(&app.db, "EvtTagListTag").await;
    let _tag = RowGuard::tag(tag_id);
    insert_event_tag(&app.db, tag_id, event_id).await;
    let _link = guard_event_tag(tag_id, event_id);

    let resp = app
        .get(
            &format!("/api/v3/events-tags?event_id={event_id}&page=1&page_size=1000"),
            &token,
        )
        .await;
    assert_status(&resp, StatusCode::OK);
    let body: PaginatedEventsTagsResponse = resp.json();
    assert!(
        body.items
            .iter()
            .any(|t| t.tag_id == tag_id && t.event_id == event_id),
        "list should contain the fixture association"
    );
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn get_event_tag_returns_the_row() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let monitor = insert_monitor(&app.db, "EvtTagGet")
        .await
        .expect("insert monitor");
    let _mon = RowGuard::monitor(monitor.id);
    let event_id = insert_event(&app.db, monitor.id, "EvtTagGetEvt").await;
    let _evt = guard_event(event_id);
    let tag_id = insert_tag(&app.db, "EvtTagGetTag").await;
    let _tag = RowGuard::tag(tag_id);
    insert_event_tag(&app.db, tag_id, event_id).await;
    let _link = guard_event_tag(tag_id, event_id);

    let resp = app
        .get(&format!("/api/v3/events-tags/{tag_id}/{event_id}"), &token)
        .await;
    assert_status(&resp, StatusCode::OK);
    let body: EventTagResponse = resp.json();
    assert_eq!(body.tag_id, tag_id);
    assert_eq!(body.event_id, event_id);
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn get_missing_event_tag_is_not_found() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    let resp = app
        .get("/api/v3/events-tags/999000111/999000222", &token)
        .await;
    assert_error(&resp, StatusCode::NOT_FOUND, "EVENT_TAG_NOT_FOUND_ERROR");
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn create_then_delete_event_tag_round_trips() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let monitor = insert_monitor(&app.db, "EvtTagRoundTrip")
        .await
        .expect("insert monitor");
    let _mon = RowGuard::monitor(monitor.id);
    let event_id = insert_event(&app.db, monitor.id, "EvtTagRoundTripEvt").await;
    let _evt = guard_event(event_id);
    let tag_id = insert_tag(&app.db, "EvtTagRoundTripTag").await;
    let _tag = RowGuard::tag(tag_id);

    let body = json!({ "event_id": event_id, "tag_id": tag_id });
    let create = app.post_json("/api/v3/events-tags", &token, &body).await;
    assert!(
        create.status().is_success(),
        "create should succeed; body: {}",
        create.text()
    );
    // Safety net: the association is deleted through the API below, but if an
    // assertion before that panics the guard still reclaims it.
    let _link = guard_event_tag(tag_id, event_id);

    let delete = app
        .delete(&format!("/api/v3/events-tags/{tag_id}/{event_id}"), &token)
        .await;
    assert!(
        delete.status().is_success(),
        "delete should succeed; got {}",
        delete.status()
    );

    let get = app
        .get(&format!("/api/v3/events-tags/{tag_id}/{event_id}"), &token)
        .await;
    assert_eq!(get.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn create_event_tag_with_invalid_body_is_rejected() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    // `event_id` must be a number, not a string.
    let resp = app
        .post_json(
            "/api/v3/events-tags",
            &token,
            &json!({ "event_id": "not-a-number", "tag_id": 1 }),
        )
        .await;
    assert!(
        resp.status().is_client_error(),
        "malformed create body should be a 4xx, got {}; body: {}",
        resp.status(),
        resp.text()
    );
}

// --- Row-level monitor ACL -------------------------------------------------
//
// A user restricted to specific monitors (via Monitors_Permissions) must not
// be able to read or mutate tag associations for events of monitors outside
// their scope. Hidden monitors return 404 (existence is not revealed), mirror-
// ing the events ACL policy in `it_monitor_acl.rs`.

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn restricted_user_cannot_read_event_tag_of_hidden_monitor() {
    let app = TestApp::spawn().await;

    let visible = insert_monitor(&app.db, "EtagAclVisible")
        .await
        .expect("insert visible monitor");
    let _mv = RowGuard::monitor(visible.id);
    let hidden = insert_monitor(&app.db, "EtagAclHidden")
        .await
        .expect("insert hidden monitor");
    let _mh = RowGuard::monitor(hidden.id);

    let event_id = insert_event(&app.db, hidden.id, "EtagAclHiddenEvt").await;
    let _evt = guard_event(event_id);
    let tag_id = insert_tag(&app.db, "EtagAclTag").await;
    let _tag = RowGuard::tag(tag_id);
    insert_event_tag(&app.db, tag_id, event_id).await;
    let _link = guard_event_tag(tag_id, event_id);

    // The user is granted access to the visible monitor only, making them
    // restricted (non-empty permission set) with no access to `hidden`.
    insert_user_with_id(&app.db, ETAG_ACL_UID, "EtagAclUser")
        .await
        .expect("insert user");
    let _ug = RowGuard::new(format!("Users#{ETAG_ACL_UID}"), move |db| async move {
        let _ = common::fixtures::cleanup_user(&db, ETAG_ACL_UID).await;
    });
    grant_monitor_permission(&app.db, visible.id, ETAG_ACL_UID, Permission::View)
        .await
        .expect("grant permission");
    let _pg = monitor_permissions_guard(ETAG_ACL_UID);

    let token = token_for(ETAG_ACL_UID, UserPermissions::superuser());

    // GET by composite id: hidden monitor's association is 404, not 200.
    let get = app
        .get(&format!("/api/v3/events-tags/{tag_id}/{event_id}"), &token)
        .await;
    assert_eq!(
        get.status(),
        StatusCode::NOT_FOUND,
        "hidden monitor's event tag must be 404, got {}",
        get.status()
    );

    // LIST filtered to the hidden event returns nothing for this user.
    let list = app
        .get(
            &format!("/api/v3/events-tags?event_id={event_id}&page=1&page_size=1000"),
            &token,
        )
        .await;
    assert_status(&list, StatusCode::OK);
    let body: PaginatedEventsTagsResponse = list.json();
    assert!(
        !body.items.iter().any(|t| t.event_id == event_id),
        "restricted user must not see associations of hidden monitors"
    );
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn restricted_user_cannot_tag_event_of_hidden_monitor() {
    let app = TestApp::spawn().await;

    let visible = insert_monitor(&app.db, "EtagAclCreateVisible")
        .await
        .expect("insert visible monitor");
    let _mv = RowGuard::monitor(visible.id);
    let hidden = insert_monitor(&app.db, "EtagAclCreateHidden")
        .await
        .expect("insert hidden monitor");
    let _mh = RowGuard::monitor(hidden.id);

    let event_id = insert_event(&app.db, hidden.id, "EtagAclCreateEvt").await;
    let _evt = guard_event(event_id);
    let tag_id = insert_tag(&app.db, "EtagAclCreateTag").await;
    let _tag = RowGuard::tag(tag_id);

    insert_user_with_id(&app.db, ETAG_ACL_UID, "EtagAclCreateUser")
        .await
        .expect("insert user");
    let _ug = RowGuard::new(format!("Users#{ETAG_ACL_UID}"), move |db| async move {
        let _ = common::fixtures::cleanup_user(&db, ETAG_ACL_UID).await;
    });
    grant_monitor_permission(&app.db, visible.id, ETAG_ACL_UID, Permission::View)
        .await
        .expect("grant permission");
    let _pg = monitor_permissions_guard(ETAG_ACL_UID);
    // Reclaim the association if the (buggy) create unexpectedly succeeds.
    let _link = guard_event_tag(tag_id, event_id);

    let token = token_for(ETAG_ACL_UID, UserPermissions::superuser());
    let create = app
        .post_json(
            "/api/v3/events-tags",
            &token,
            &json!({ "event_id": event_id, "tag_id": tag_id }),
        )
        .await;
    assert_eq!(
        create.status(),
        StatusCode::NOT_FOUND,
        "tagging a hidden monitor's event must be refused (404), got {}; body: {}",
        create.status(),
        create.text()
    );
}
