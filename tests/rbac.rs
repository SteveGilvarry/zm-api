//! Tests for per-resource RBAC enforcement (`zm_api::util::authz`).
//!
//! RBAC rejects requests before any handler runs, so these need no database.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use sea_orm::{DatabaseBackend, MockDatabase};
use tower::util::ServiceExt;
use zm_api::service::token::generate_tokens;
use zm_api::util::authz::{Level, UserPermissions};

fn router() -> axum::Router {
    let db = MockDatabase::new(DatabaseBackend::MySql).into_connection();
    let state = zm_api::server::state::AppState::for_test_with_db(db);
    zm_api::routes::create_router_app(state)
}

fn token(perms: UserPermissions) -> String {
    generate_tokens("rbac-tester".to_string(), 1, perms)
        .expect("token")
        .access_token
}

async fn status(method: &str, uri: &str, bearer: Option<&str>) -> StatusCode {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(t) = bearer {
        builder = builder.header("Authorization", format!("Bearer {t}"));
    }
    router()
        .oneshot(builder.body(Body::empty()).unwrap())
        .await
        .unwrap()
        .status()
}

#[tokio::test]
async fn no_token_is_unauthorized() {
    assert_eq!(
        status("GET", "/api/v3/monitors", None).await,
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn missing_feature_permission_is_forbidden() {
    // A user with no `Monitors` permission cannot read monitors.
    let t = token(UserPermissions::default());
    assert_eq!(
        status("GET", "/api/v3/monitors", Some(&t)).await,
        StatusCode::FORBIDDEN
    );
}

#[tokio::test]
async fn view_permission_cannot_perform_writes() {
    // `View` is enough to read but not to mutate — writes require `Edit`.
    let perms = UserPermissions {
        monitors: Level::View,
        ..UserPermissions::default()
    };
    let t = token(perms);
    assert_eq!(
        status("POST", "/api/v3/monitors", Some(&t)).await,
        StatusCode::FORBIDDEN
    );
}

#[tokio::test]
async fn sufficient_permission_passes_rbac() {
    // A superuser token clears RBAC; the request then reaches the handler
    // (which may itself succeed or fail), so it is neither 401 nor 403.
    let t = token(UserPermissions::superuser());
    let got = status("GET", "/api/v3/monitors", Some(&t)).await;
    assert_ne!(got, StatusCode::UNAUTHORIZED);
    assert_ne!(got, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn wrong_feature_permission_does_not_grant_access() {
    // Holding `System` permission does not grant `Monitors` access.
    let perms = UserPermissions {
        system: Level::Edit,
        ..UserPermissions::default()
    };
    let t = token(perms);
    assert_eq!(
        status("GET", "/api/v3/monitors", Some(&t)).await,
        StatusCode::FORBIDDEN
    );
}

// ---------------------------------------------------------------------------
// Granting permissions to users is an administrative operation. The
// `/groups-permissions` and `/monitors-permissions` POST/PATCH/DELETE routes
// must require `System:Edit`, not the feature they manage — otherwise a user
// with `Groups:Edit` or `Monitors:Edit` could grant themselves or others
// elevated row-level access.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn groups_edit_alone_cannot_grant_group_permissions() {
    let perms = UserPermissions {
        groups: Level::Edit,
        ..UserPermissions::default()
    };
    let t = token(perms);
    assert_eq!(
        status("POST", "/api/v3/groups-permissions", Some(&t)).await,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        status("DELETE", "/api/v3/groups-permissions/1", Some(&t)).await,
        StatusCode::FORBIDDEN
    );
}

#[tokio::test]
async fn monitors_edit_alone_cannot_grant_monitor_permissions() {
    let perms = UserPermissions {
        monitors: Level::Edit,
        ..UserPermissions::default()
    };
    let t = token(perms);
    assert_eq!(
        status("POST", "/api/v3/monitors-permissions", Some(&t)).await,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        status("DELETE", "/api/v3/monitors-permissions/1", Some(&t)).await,
        StatusCode::FORBIDDEN
    );
}

#[tokio::test]
async fn state_change_requires_system_edit() {
    // POST /api/v3/states/change/{action} invokes `systemctl restart` on
    // zoneminder. Any token-holder must not be able to trigger it; require
    // System:Edit even though Monitors:Edit is the "biggest" feature most
    // operators carry.
    let perms = UserPermissions {
        monitors: Level::Edit,
        ..UserPermissions::default()
    };
    let t = token(perms);
    assert_eq!(
        status("POST", "/api/v3/states/change/restart", Some(&t)).await,
        StatusCode::FORBIDDEN
    );
}

#[tokio::test]
async fn system_edit_clears_rbac_for_permission_crud() {
    let perms = UserPermissions {
        system: Level::Edit,
        ..UserPermissions::default()
    };
    let t = token(perms);
    // Both routes should clear RBAC for a System:Edit caller — what happens
    // beyond that (bad JSON body, etc.) is a handler concern, not an RBAC one.
    let got = status("POST", "/api/v3/groups-permissions", Some(&t)).await;
    assert_ne!(got, StatusCode::FORBIDDEN);
    assert_ne!(got, StatusCode::UNAUTHORIZED);
    let got = status("POST", "/api/v3/monitors-permissions", Some(&t)).await;
    assert_ne!(got, StatusCode::FORBIDDEN);
    assert_ne!(got, StatusCode::UNAUTHORIZED);
}
