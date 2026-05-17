//! Integration tests for row-level group ACLs.
//!
//! A user with `Groups_Permissions` rows sees only the groups they are
//! granted; a user with no rows is unrestricted (default-allow).
//!
//! Requires the test database — run with:
//!   APP_PROFILE=test-db cargo test --test it_group_acl -- --include-ignored

mod common;

use axum::http::{Method, StatusCode};
use common::fixtures::{
    cleanup_group_permissions, cleanup_user, delete_group, grant_group_permission, insert_group,
    insert_user_with_id,
};
use common::harness::{superuser_token, token_for, TestApp};
use zm_api::entity::sea_orm_active_enums::Permission;
use zm_api::util::authz::UserPermissions;

/// User ids unlikely to collide with real ZoneMinder users. Each test uses a
/// distinct id so the suite is safe to run concurrently; rows are cleaned up
/// by id after each test.
const ACL_TEST_UID_VIEW: u32 = 990_021;
const ACL_TEST_UID_EDIT: u32 = 990_022;

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn restricted_user_sees_only_permitted_groups() {
    let app = TestApp::spawn().await;

    let group_a = insert_group(&app.db, "AclGrpVisible")
        .await
        .expect("insert group A");
    let group_b = insert_group(&app.db, "AclGrpHidden")
        .await
        .expect("insert group B");

    // The permission row's `UserId` FK requires the user to exist first.
    insert_user_with_id(&app.db, ACL_TEST_UID_VIEW, "AclGrpUser")
        .await
        .expect("insert acl user");

    // The user is granted View on A only — which makes their scope Restricted.
    grant_group_permission(&app.db, group_a.id, ACL_TEST_UID_VIEW, Permission::View)
        .await
        .expect("grant permission");

    // Feature-level RBAC is satisfied (superuser perms); the row-level ACL is
    // driven entirely by the token's user id.
    let token = token_for(ACL_TEST_UID_VIEW, UserPermissions::superuser());

    // List: only group A is visible.
    let list = app
        .get("/api/v3/groups?page=1&page_size=1000", &token)
        .await;
    assert_eq!(list.status(), StatusCode::OK);
    let body = list.text();
    assert!(
        body.contains(&group_a.name),
        "permitted group should be listed"
    );
    assert!(
        !body.contains(&group_b.name),
        "group outside the ACL scope must not be listed"
    );

    // Item: the permitted group is reachable.
    let permitted = app
        .get(&format!("/api/v3/groups/{}", group_a.id), &token)
        .await;
    assert_eq!(permitted.status(), StatusCode::OK);

    // Item: the hidden group 404s (not 403 — its existence is not revealed).
    let hidden = app
        .get(&format!("/api/v3/groups/{}", group_b.id), &token)
        .await;
    assert_eq!(hidden.status(), StatusCode::NOT_FOUND);

    cleanup_group_permissions(&app.db, ACL_TEST_UID_VIEW)
        .await
        .expect("cleanup permissions");
    cleanup_user(&app.db, ACL_TEST_UID_VIEW)
        .await
        .expect("cleanup acl user");
    delete_group(&app.db, group_a.id)
        .await
        .expect("cleanup group A");
    delete_group(&app.db, group_b.id)
        .await
        .expect("cleanup group B");
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn view_only_grant_blocks_writes() {
    let app = TestApp::spawn().await;

    let group = insert_group(&app.db, "AclGrpViewOnly")
        .await
        .expect("insert group");
    insert_user_with_id(&app.db, ACL_TEST_UID_EDIT, "AclGrpEditUser")
        .await
        .expect("insert acl user");

    // View only — reads are allowed, writes are not.
    grant_group_permission(&app.db, group.id, ACL_TEST_UID_EDIT, Permission::View)
        .await
        .expect("grant permission");
    let token = token_for(ACL_TEST_UID_EDIT, UserPermissions::superuser());

    // Read succeeds.
    let read = app
        .get(&format!("/api/v3/groups/{}", group.id), &token)
        .await;
    assert_eq!(read.status(), StatusCode::OK);

    // Delete requires Edit — the View grant makes it a 404 (existence hidden).
    let delete = app
        .request(Method::DELETE, &format!("/api/v3/groups/{}", group.id))
        .bearer(&token)
        .send()
        .await;
    assert_eq!(delete.status(), StatusCode::NOT_FOUND);

    cleanup_group_permissions(&app.db, ACL_TEST_UID_EDIT)
        .await
        .expect("cleanup permissions");
    cleanup_user(&app.db, ACL_TEST_UID_EDIT)
        .await
        .expect("cleanup acl user");
    delete_group(&app.db, group.id)
        .await
        .expect("cleanup group");
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn unrestricted_user_sees_all_groups() {
    let app = TestApp::spawn().await;

    let group_a = insert_group(&app.db, "AclGrpAll1")
        .await
        .expect("insert group A");
    let group_b = insert_group(&app.db, "AclGrpAll2")
        .await
        .expect("insert group B");

    // `superuser_token()` carries user id 0, which has no `Groups_Permissions`
    // rows — default-allow, so both groups are visible.
    let token = superuser_token();

    let list = app
        .get("/api/v3/groups?page=1&page_size=1000", &token)
        .await;
    assert_eq!(list.status(), StatusCode::OK);
    let body = list.text();
    assert!(body.contains(&group_a.name));
    assert!(body.contains(&group_b.name));

    delete_group(&app.db, group_a.id)
        .await
        .expect("cleanup group A");
    delete_group(&app.db, group_b.id)
        .await
        .expect("cleanup group B");
}
