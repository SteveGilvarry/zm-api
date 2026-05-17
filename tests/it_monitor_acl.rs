//! Integration tests for row-level monitor ACLs.
//!
//! A user with `Monitors_Permissions` rows sees only the monitors they are
//! granted; a user with no rows is unrestricted (default-allow).
//!
//! Requires the test database — run with:
//!   APP_PROFILE=test-db cargo test --test it_monitor_acl -- --include-ignored

mod common;

use axum::http::{Method, StatusCode};
use common::fixtures::{
    cleanup_monitor_permissions, cleanup_user, delete_monitor, grant_monitor_permission,
    insert_monitor, insert_user_with_id,
};
use common::harness::{superuser_token, token_for, TestApp};
use zm_api::entity::sea_orm_active_enums::Permission;
use zm_api::util::authz::UserPermissions;

/// User ids unlikely to collide with real ZoneMinder users. Each test uses a
/// distinct id so the suite is safe to run concurrently; rows are cleaned up
/// by id after each test.
const ACL_TEST_UID: u32 = 990_001;
const ACL_TEST_UID_EVENTS: u32 = 990_002;

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn restricted_user_sees_only_permitted_monitors() {
    let app = TestApp::spawn().await;

    let monitor_a = insert_monitor(&app.db, "AclVisible")
        .await
        .expect("insert monitor A");
    let monitor_b = insert_monitor(&app.db, "AclHidden")
        .await
        .expect("insert monitor B");

    // The permission row's `UserId` FK requires the user to exist first.
    insert_user_with_id(&app.db, ACL_TEST_UID, "AclUser")
        .await
        .expect("insert acl user");

    // The user is granted View on A only — which makes their scope Restricted.
    grant_monitor_permission(&app.db, monitor_a.id, ACL_TEST_UID, Permission::View)
        .await
        .expect("grant permission");

    // Feature-level RBAC is satisfied (superuser perms); the row-level ACL is
    // driven entirely by the token's user id.
    let token = token_for(ACL_TEST_UID, UserPermissions::superuser());

    // List: only monitor A is visible.
    let list = app
        .get("/api/v3/monitors?page=1&page_size=1000", &token)
        .await;
    assert_eq!(list.status(), StatusCode::OK);
    let body = list.text();
    assert!(
        body.contains(&monitor_a.name),
        "permitted monitor should be listed"
    );
    assert!(
        !body.contains(&monitor_b.name),
        "monitor outside the ACL scope must not be listed"
    );

    // Item: the permitted monitor is reachable.
    let permitted = app
        .get(&format!("/api/v3/monitors/{}", monitor_a.id), &token)
        .await;
    assert_eq!(permitted.status(), StatusCode::OK);

    // Item: the hidden monitor 404s (not 403 — its existence is not revealed).
    let hidden = app
        .get(&format!("/api/v3/monitors/{}", monitor_b.id), &token)
        .await;
    assert_eq!(hidden.status(), StatusCode::NOT_FOUND);

    // A write to the hidden monitor is also a 404.
    let write_hidden = app
        .request(
            Method::DELETE,
            &format!("/api/v3/monitors/{}", monitor_b.id),
        )
        .bearer(&token)
        .send()
        .await;
    assert_eq!(write_hidden.status(), StatusCode::NOT_FOUND);

    cleanup_monitor_permissions(&app.db, ACL_TEST_UID)
        .await
        .expect("cleanup permissions");
    cleanup_user(&app.db, ACL_TEST_UID)
        .await
        .expect("cleanup acl user");
    delete_monitor(&app.db, monitor_a.id)
        .await
        .expect("cleanup monitor A");
    delete_monitor(&app.db, monitor_b.id)
        .await
        .expect("cleanup monitor B");
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn unrestricted_user_sees_all_monitors() {
    let app = TestApp::spawn().await;

    let monitor_a = insert_monitor(&app.db, "AclAll1")
        .await
        .expect("insert monitor A");
    let monitor_b = insert_monitor(&app.db, "AclAll2")
        .await
        .expect("insert monitor B");

    // `superuser_token()` carries user id 0, which has no `Monitors_Permissions`
    // rows — default-allow, so both monitors are visible.
    let token = superuser_token();

    let list = app
        .get("/api/v3/monitors?page=1&page_size=1000", &token)
        .await;
    assert_eq!(list.status(), StatusCode::OK);
    let body = list.text();
    assert!(body.contains(&monitor_a.name));
    assert!(body.contains(&monitor_b.name));

    delete_monitor(&app.db, monitor_a.id)
        .await
        .expect("cleanup monitor A");
    delete_monitor(&app.db, monitor_b.id)
        .await
        .expect("cleanup monitor B");
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn events_of_hidden_monitors_are_filtered() {
    use common::fixtures::unique_name;
    use sea_orm::{ActiveModelTrait, Set};

    let app = TestApp::spawn().await;

    let visible = insert_monitor(&app.db, "AclEvtVisible")
        .await
        .expect("insert visible monitor");
    let hidden = insert_monitor(&app.db, "AclEvtHidden")
        .await
        .expect("insert hidden monitor");

    // One event per monitor.
    let visible_event = zm_api::entity::events::ActiveModel {
        monitor_id: Set(visible.id),
        state_id: Set(1),
        name: Set(unique_name("AclEvtVisible")),
        ..Default::default()
    }
    .insert(&app.db)
    .await
    .expect("insert visible event");
    let hidden_event = zm_api::entity::events::ActiveModel {
        monitor_id: Set(hidden.id),
        state_id: Set(1),
        name: Set(unique_name("AclEvtHidden")),
        ..Default::default()
    }
    .insert(&app.db)
    .await
    .expect("insert hidden event");

    insert_user_with_id(&app.db, ACL_TEST_UID_EVENTS, "AclEvtUser")
        .await
        .expect("insert acl user");
    grant_monitor_permission(&app.db, visible.id, ACL_TEST_UID_EVENTS, Permission::View)
        .await
        .expect("grant permission");
    let token = token_for(ACL_TEST_UID_EVENTS, UserPermissions::superuser());

    let list = app
        .get("/api/v3/events?page=1&page_size=1000", &token)
        .await;
    assert_eq!(list.status(), StatusCode::OK);
    let body = list.text();
    assert!(
        body.contains(&visible_event.name),
        "event of a permitted monitor should be listed"
    );
    assert!(
        !body.contains(&hidden_event.name),
        "event of a hidden monitor must not be listed"
    );

    // Direct access to the hidden monitor's event 404s.
    let hidden_get = app
        .get(&format!("/api/v3/events/{}", hidden_event.id), &token)
        .await;
    assert_eq!(hidden_get.status(), StatusCode::NOT_FOUND);

    // Cleanup: events first (FK), then permissions and monitors.
    use sea_orm::EntityTrait;
    let _ = zm_api::entity::events::Entity::delete_by_id(visible_event.id)
        .exec(&app.db)
        .await;
    let _ = zm_api::entity::events::Entity::delete_by_id(hidden_event.id)
        .exec(&app.db)
        .await;
    cleanup_monitor_permissions(&app.db, ACL_TEST_UID_EVENTS)
        .await
        .expect("cleanup permissions");
    cleanup_user(&app.db, ACL_TEST_UID_EVENTS)
        .await
        .expect("cleanup acl user");
    delete_monitor(&app.db, visible.id)
        .await
        .expect("cleanup visible monitor");
    delete_monitor(&app.db, hidden.id)
        .await
        .expect("cleanup hidden monitor");
}
