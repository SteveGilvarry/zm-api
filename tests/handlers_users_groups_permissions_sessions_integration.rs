// Integration tests for users, groups, permissions, and sessions with a real database
// Run with: cargo test --test handlers_users_groups_permissions_sessions_integration -- --include-ignored
#![allow(clippy::needless_borrows_for_generic_args)]

mod common;

use axum::body::{self, Body};
use axum::http::{header, Request, StatusCode};
use common::fixtures::RowGuard;
use common::test_db::{get_test_db, test_prefix};
use sea_orm::{ActiveModelTrait, DatabaseConnection, DbErr, EntityTrait, Set};
use tower::ServiceExt;
use zm_api::dto::request::groups_permissions::CreateGroupPermissionRequest;
use zm_api::dto::request::monitors_permissions::CreateMonitorPermissionRequest;
use zm_api::dto::request::sessions::{CreateSessionRequest, UpdateSessionRequest};
use zm_api::dto::request::{CreateGroupRequest, CreateUserRequest};
use zm_api::dto::response::{
    GroupPermissionResponse, GroupResponse, MonitorPermissionResponse, SessionResponse,
    UserResponse,
};
use zm_api::entity::{groups_permissions, monitors, monitors_permissions, sessions, users};

fn auth_header() -> String {
    let token = zm_api::service::token::generate_tokens(
        "tester".to_string(),
        1,
        zm_api::util::authz::UserPermissions::superuser(),
    )
    .expect("token")
    .access_token;
    format!("Bearer {}", token)
}

fn build_app(db: DatabaseConnection) -> axum::Router {
    let state = zm_api::server::state::AppState::for_test_with_db(db);
    zm_api::routes::create_router_app(state)
}

async fn create_monitor_db(db: &DatabaseConnection) -> Result<monitors::Model, DbErr> {
    let name = format!("{}perm_monitor", test_prefix());
    let model = monitors::ActiveModel {
        name: Set(name),
        ..Default::default()
    };
    model.insert(db).await
}

#[tokio::test]
#[ignore = "Requires running test database - run with: ./scripts/db-manager.sh mysql"]
async fn test_api_users_create_get_delete() {
    let db = get_test_db()
        .await
        .expect("Failed to connect to test database");
    // Second handle to read the stored row back after creation (the `mock`
    // feature disables `DatabaseConnection: Clone`, so open a fresh one).
    let probe_db = get_test_db()
        .await
        .expect("Failed to connect to test database");
    let app = build_app(db);

    let username = format!("{}user", test_prefix());
    let create_body = serde_json::to_vec(&CreateUserRequest {
        username: username.clone(),
        password: "testpass".to_string(),
        email: format!("{}@example.com", username),
        name: Some("Test User".to_string()),
        phone: None,
        enabled: Some(1),
    })
    .expect("serialize user");

    let response = app
        .clone()
        .oneshot(
            Request::post("/api/v3/users")
                .header(header::AUTHORIZATION, auth_header())
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(create_body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let bytes = body::to_bytes(response.into_body(), 64 * 1024)
        .await
        .unwrap();
    let created: UserResponse = serde_json::from_slice(&bytes).unwrap();
    let _g_user = RowGuard::user(created.id);
    assert_eq!(created.username, username);

    // Regression for REVIEW_FIXES_PLAN §1.1: the stored password must be a
    // bcrypt hash, never the plaintext, and it must verify against the
    // original plaintext (proving the create→login round-trip works).
    let stored = users::Entity::find_by_id(created.id)
        .one(&probe_db)
        .await
        .expect("query user")
        .expect("user row exists");
    assert_ne!(stored.password, "testpass", "password stored in plaintext");
    assert!(
        stored.password.starts_with("$2"),
        "password is not a bcrypt hash: {}",
        stored.password
    );
    assert!(
        zm_api::util::password::verify_existing_or_dummy(
            "testpass".to_string(),
            Some(stored.password.clone()),
        )
        .await,
        "stored hash does not verify against the original password"
    );

    let response = app
        .clone()
        .oneshot(
            Request::get(&format!("/api/v3/users/{}", created.id))
                .header(header::AUTHORIZATION, auth_header())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let response = app
        .oneshot(
            Request::delete(&format!("/api/v3/users/{}", created.id))
                .header(header::AUTHORIZATION, auth_header())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
#[ignore = "Requires running test database - run with: ./scripts/db-manager.sh mysql"]
async fn test_api_groups_create_get_delete() {
    let db = get_test_db()
        .await
        .expect("Failed to connect to test database");
    let app = build_app(db);

    let name = format!("{}group", test_prefix());
    let create_body = serde_json::to_vec(&CreateGroupRequest {
        name: name.clone(),
        parent_id: None,
    })
    .expect("serialize group");

    let response = app
        .clone()
        .oneshot(
            Request::post("/api/v3/groups")
                .header(header::AUTHORIZATION, auth_header())
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(create_body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let bytes = body::to_bytes(response.into_body(), 64 * 1024)
        .await
        .unwrap();
    let created: GroupResponse = serde_json::from_slice(&bytes).unwrap();
    let _g_group = RowGuard::group(created.id);
    assert_eq!(created.name, name);

    let response = app
        .clone()
        .oneshot(
            Request::get(&format!("/api/v3/groups/{}", created.id))
                .header(header::AUTHORIZATION, auth_header())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let response = app
        .oneshot(
            Request::delete(&format!("/api/v3/groups/{}", created.id))
                .header(header::AUTHORIZATION, auth_header())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
#[ignore = "Requires running test database - run with: ./scripts/db-manager.sh mysql"]
async fn test_api_groups_permissions_create_delete() {
    let db = get_test_db()
        .await
        .expect("Failed to connect to test database");
    let app = build_app(db);

    let username = format!("{}gp_user", test_prefix());
    let user_body = serde_json::to_vec(&CreateUserRequest {
        username: username.clone(),
        password: "testpass".to_string(),
        email: format!("{}@example.com", username),
        name: Some("Test User".to_string()),
        phone: None,
        enabled: Some(1),
    })
    .expect("serialize user");

    let response = app
        .clone()
        .oneshot(
            Request::post("/api/v3/users")
                .header(header::AUTHORIZATION, auth_header())
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(user_body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let bytes = body::to_bytes(response.into_body(), 64 * 1024)
        .await
        .unwrap();
    let user: UserResponse = serde_json::from_slice(&bytes).unwrap();
    let _g_user = RowGuard::user(user.id);

    let group_name = format!("{}gp_group", test_prefix());
    let group_body = serde_json::to_vec(&CreateGroupRequest {
        name: group_name,
        parent_id: None,
    })
    .expect("serialize group");

    let response = app
        .clone()
        .oneshot(
            Request::post("/api/v3/groups")
                .header(header::AUTHORIZATION, auth_header())
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(group_body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let bytes = body::to_bytes(response.into_body(), 64 * 1024)
        .await
        .unwrap();
    let group: GroupResponse = serde_json::from_slice(&bytes).unwrap();
    let _g_group = RowGuard::group(group.id);

    let perm_body = serde_json::to_vec(&CreateGroupPermissionRequest {
        group_id: group.id,
        user_id: user.id,
        permission: "View".to_string(),
    })
    .expect("serialize group permission");

    let response = app
        .clone()
        .oneshot(
            Request::post("/api/v3/groups-permissions")
                .header(header::AUTHORIZATION, auth_header())
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(perm_body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let bytes = body::to_bytes(response.into_body(), 64 * 1024)
        .await
        .unwrap();
    let perm: GroupPermissionResponse = serde_json::from_slice(&bytes).unwrap();
    let perm_id = perm.id;
    let _g_perm = RowGuard::new(
        format!("Groups_Permissions#{perm_id}"),
        move |db| async move {
            let _ = groups_permissions::Entity::delete_by_id(perm_id)
                .exec(&db)
                .await;
        },
    );
    assert_eq!(perm.user_id, user.id);

    let response = app
        .clone()
        .oneshot(
            Request::delete(&format!("/api/v3/groups-permissions/{}", perm.id))
                .header(header::AUTHORIZATION, auth_header())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    let response = app
        .clone()
        .oneshot(
            Request::delete(&format!("/api/v3/groups/{}", group.id))
                .header(header::AUTHORIZATION, auth_header())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    let response = app
        .oneshot(
            Request::delete(&format!("/api/v3/users/{}", user.id))
                .header(header::AUTHORIZATION, auth_header())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
#[ignore = "Requires running test database - run with: ./scripts/db-manager.sh mysql"]
async fn test_api_monitors_permissions_create_delete() {
    let db = get_test_db()
        .await
        .expect("Failed to connect to test database");
    let monitor = create_monitor_db(&db)
        .await
        .expect("Failed to create monitor");
    let _g_monitor = RowGuard::monitor(monitor.id);
    let app = build_app(db);

    let username = format!("{}mp_user", test_prefix());
    let user_body = serde_json::to_vec(&CreateUserRequest {
        username: username.clone(),
        password: "testpass".to_string(),
        email: format!("{}@example.com", username),
        name: Some("Test User".to_string()),
        phone: None,
        enabled: Some(1),
    })
    .expect("serialize user");

    let response = app
        .clone()
        .oneshot(
            Request::post("/api/v3/users")
                .header(header::AUTHORIZATION, auth_header())
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(user_body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let bytes = body::to_bytes(response.into_body(), 64 * 1024)
        .await
        .unwrap();
    let user: UserResponse = serde_json::from_slice(&bytes).unwrap();
    let _g_user = RowGuard::user(user.id);

    let perm_body = serde_json::to_vec(&CreateMonitorPermissionRequest {
        monitor_id: monitor.id,
        user_id: user.id,
        permission: "View".to_string(),
    })
    .expect("serialize monitor permission");

    let response = app
        .clone()
        .oneshot(
            Request::post("/api/v3/monitors-permissions")
                .header(header::AUTHORIZATION, auth_header())
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(perm_body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let bytes = body::to_bytes(response.into_body(), 64 * 1024)
        .await
        .unwrap();
    let perm: MonitorPermissionResponse = serde_json::from_slice(&bytes).unwrap();
    let perm_id = perm.id;
    let _g_perm = RowGuard::new(
        format!("Monitors_Permissions#{perm_id}"),
        move |db| async move {
            let _ = monitors_permissions::Entity::delete_by_id(perm_id)
                .exec(&db)
                .await;
        },
    );
    assert_eq!(perm.monitor_id, monitor.id);

    let response = app
        .clone()
        .oneshot(
            Request::delete(&format!("/api/v3/monitors-permissions/{}", perm.id))
                .header(header::AUTHORIZATION, auth_header())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    let response = app
        .clone()
        .oneshot(
            Request::delete(&format!("/api/v3/users/{}", user.id))
                .header(header::AUTHORIZATION, auth_header())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
#[ignore = "Requires running test database - run with: ./scripts/db-manager.sh mysql"]
async fn test_api_sessions_create_update_delete() {
    let db = get_test_db()
        .await
        .expect("Failed to connect to test database");
    let app = build_app(db);

    let prefix = test_prefix();
    let short = if prefix.len() > 16 {
        &prefix[prefix.len() - 16..]
    } else {
        prefix.as_str()
    };
    let session_id = format!("s_{short}");
    let create_body = serde_json::to_vec(&CreateSessionRequest {
        id: session_id.clone(),
        access: Some(1),
        data: Some("test".to_string()),
    })
    .expect("serialize session");

    let response = app
        .clone()
        .oneshot(
            Request::post("/api/v3/sessions")
                .header(header::AUTHORIZATION, auth_header())
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(create_body))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let bytes = body::to_bytes(response.into_body(), 64 * 1024)
        .await
        .unwrap();
    if status != StatusCode::CREATED {
        panic!(
            "Unexpected status {}: {}",
            status,
            String::from_utf8_lossy(&bytes)
        );
    }
    let created: SessionResponse = serde_json::from_slice(&bytes).unwrap();
    let guard_session_id = created.id.clone();
    let _g_session = RowGuard::new(
        format!("Sessions#{guard_session_id}"),
        move |db| async move {
            let _ = sessions::Entity::delete_by_id(guard_session_id)
                .exec(&db)
                .await;
        },
    );
    assert_eq!(created.id, session_id);

    let update_body = serde_json::to_vec(&UpdateSessionRequest {
        access: Some(2),
        data: Some("updated".to_string()),
    })
    .expect("serialize session update");

    let response = app
        .clone()
        .oneshot(
            Request::patch(&format!("/api/v3/sessions/{}", session_id))
                .header(header::AUTHORIZATION, auth_header())
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(update_body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = body::to_bytes(response.into_body(), 64 * 1024)
        .await
        .unwrap();
    let updated: SessionResponse = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(updated.access, Some(2));

    let response = app
        .oneshot(
            Request::delete(&format!("/api/v3/sessions/{}", session_id))
                .header(header::AUTHORIZATION, auth_header())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}
