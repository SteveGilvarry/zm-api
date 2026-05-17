//! Integration tests for the Monitors-Permissions API — happy-path plus error paths.
//!
//! Requires the test database — run with:
//!   APP_PROFILE=test-db cargo test --test it_monitors_permissions -- --include-ignored

mod common;

use axum::http::StatusCode;
use common::assertions::{assert_error, assert_status};
use common::fixtures::{cleanup_user, delete_monitor, insert_monitor, insert_user_with_id};
use common::harness::{superuser_token, TestApp};
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use serde_json::json;
use zm_api::dto::response::{MonitorPermissionResponse, PaginatedMonitorPermissionsResponse};
use zm_api::entity::sea_orm_active_enums::Permission;

/// Insert a `Monitors_Permissions` row directly and return its id.
async fn insert_permission(db: &sea_orm::DatabaseConnection, monitor_id: u32, user_id: u32) -> u32 {
    zm_api::entity::monitors_permissions::ActiveModel {
        monitor_id: Set(monitor_id),
        user_id: Set(user_id),
        permission: Set(Permission::View),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("insert monitor permission fixture")
    .id
}

async fn delete_permission(db: &sea_orm::DatabaseConnection, id: u32) {
    let _ = zm_api::entity::monitors_permissions::Entity::delete_by_id(id)
        .exec(db)
        .await;
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn list_monitors_permissions_returns_inserted_row() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let monitor = insert_monitor(&app.db, "MonPermList")
        .await
        .expect("insert monitor");
    let user_id = 990_011;
    insert_user_with_id(&app.db, user_id, "MonPermListUser")
        .await
        .expect("insert user");
    let id = insert_permission(&app.db, monitor.id, user_id).await;

    let resp = app
        .get("/api/v3/monitors-permissions?page=1&page_size=1000", &token)
        .await;
    assert_status(&resp, StatusCode::OK);
    let body: PaginatedMonitorPermissionsResponse = resp.json();
    assert!(
        body.items.iter().any(|p| p.id == id),
        "list should contain the fixture permission"
    );

    delete_permission(&app.db, id).await;
    cleanup_user(&app.db, user_id).await.expect("cleanup user");
    delete_monitor(&app.db, monitor.id).await.expect("cleanup");
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn get_monitor_permission_returns_the_row() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let monitor = insert_monitor(&app.db, "MonPermGet")
        .await
        .expect("insert monitor");
    let user_id = 990_012;
    insert_user_with_id(&app.db, user_id, "MonPermGetUser")
        .await
        .expect("insert user");
    let id = insert_permission(&app.db, monitor.id, user_id).await;

    let resp = app
        .get(&format!("/api/v3/monitors-permissions/{id}"), &token)
        .await;
    assert_status(&resp, StatusCode::OK);
    let body: MonitorPermissionResponse = resp.json();
    assert_eq!(body.id, id);
    assert_eq!(body.monitor_id, monitor.id);

    delete_permission(&app.db, id).await;
    cleanup_user(&app.db, user_id).await.expect("cleanup user");
    delete_monitor(&app.db, monitor.id).await.expect("cleanup");
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn get_missing_monitor_permission_is_not_found() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    let resp = app
        .get("/api/v3/monitors-permissions/999000111", &token)
        .await;
    assert_error(&resp, StatusCode::NOT_FOUND, "FILE_NOT_FOUND_ERROR");
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn create_then_delete_monitor_permission_round_trips() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let monitor = insert_monitor(&app.db, "MonPermRoundTrip")
        .await
        .expect("insert monitor");
    let user_id = 990_013;
    insert_user_with_id(&app.db, user_id, "MonPermRoundTripUser")
        .await
        .expect("insert user");

    let body = json!({
        "monitor_id": monitor.id,
        "user_id": user_id,
        "permission": "View",
    });
    let create = app
        .post_json("/api/v3/monitors-permissions", &token, &body)
        .await;
    assert!(
        create.status().is_success(),
        "create should succeed; body: {}",
        create.text()
    );
    let created: MonitorPermissionResponse = create.json();

    let delete = app
        .delete(
            &format!("/api/v3/monitors-permissions/{}", created.id),
            &token,
        )
        .await;
    assert!(
        delete.status().is_success(),
        "delete should succeed; got {}",
        delete.status()
    );

    let get = app
        .get(
            &format!("/api/v3/monitors-permissions/{}", created.id),
            &token,
        )
        .await;
    assert_eq!(get.status(), StatusCode::NOT_FOUND);

    cleanup_user(&app.db, user_id).await.expect("cleanup user");
    delete_monitor(&app.db, monitor.id).await.expect("cleanup");
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn create_monitor_permission_with_invalid_body_is_rejected() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    // Missing the required `user_id` and `permission` fields.
    let resp = app
        .post_json(
            "/api/v3/monitors-permissions",
            &token,
            &json!({ "monitor_id": 1 }),
        )
        .await;
    assert!(
        resp.status().is_client_error(),
        "malformed create body should be a 4xx, got {}; body: {}",
        resp.status(),
        resp.text()
    );
}
