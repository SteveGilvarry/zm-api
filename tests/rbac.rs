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
