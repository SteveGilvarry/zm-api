//! Parameterised authentication & authorization failure coverage.
//!
//! `PROTECTED_ROUTES` is the single source of truth for the API's security
//! model: every protected route group appears here. The tests below sweep the
//! whole table, so adding a route module without adding a row will leave a
//! visible coverage gap.
//!
//! Runs against the mock-database harness — every request is rejected by the
//! auth or RBAC layer before any query, so no real database is needed.

mod common;

use axum::http::{Method, StatusCode};
use common::assertions::assert_error;
use common::harness::{superuser_token, token_with, TestApp};
use zm_api::util::authz::UserPermissions;
use zm_api::util::claim::UserClaims;

/// A route under RBAC protection. `method`/`path` identify it; `feature` is
/// the ZoneMinder permission category it is gated on (documentation only —
/// the assertions do not need it since a no-permission token is denied
/// regardless of feature).
struct ProtectedRoute {
    method: Method,
    path: &'static str,
}

const fn route(method: Method, path: &'static str) -> ProtectedRoute {
    ProtectedRoute { method, path }
}

/// Every protected route group in the API. One representative endpoint per
/// route module (read endpoints — `GET` requires `View`).
fn protected_routes() -> Vec<ProtectedRoute> {
    vec![
        // --- System ---
        route(Method::GET, "/api/v3/configs"),
        route(Method::GET, "/api/v3/logs"),
        route(Method::GET, "/api/v3/storage"),
        route(Method::GET, "/api/v3/servers"),
        route(Method::GET, "/api/v3/server-stats"),
        route(Method::GET, "/api/v3/stats"),
        route(Method::GET, "/api/v3/states"),
        route(Method::GET, "/api/v3/reports"),
        route(Method::GET, "/api/v3/daemons"),
        route(Method::GET, "/api/v3/users"),
        route(Method::GET, "/api/v3/user_preferences"),
        route(Method::GET, "/api/v3/sessions"),
        route(Method::GET, "/api/v3/montage_layouts"),
        route(Method::POST, "/api/v3/system/shutdown"),
        // --- Monitors ---
        route(Method::GET, "/api/v3/monitors"),
        route(Method::GET, "/api/v3/monitor_presets"),
        route(Method::GET, "/api/v3/monitor-status"),
        route(Method::GET, "/api/v3/monitors-permissions"),
        route(Method::GET, "/api/v3/zone-presets"),
        // --- Events ---
        route(Method::GET, "/api/v3/events"),
        route(Method::GET, "/api/v3/event-data"),
        route(Method::GET, "/api/v3/event-summaries"),
        route(Method::GET, "/api/v3/events-tags"),
        route(Method::GET, "/api/v3/frames"),
        route(Method::GET, "/api/v3/filters"),
        route(Method::GET, "/api/v3/tags"),
        route(Method::GET, "/api/v3/object-types"),
        // --- Control ---
        route(Method::GET, "/api/v3/controls"),
        route(Method::GET, "/api/v3/control_presets"),
        route(Method::GET, "/api/v3/triggers_x10"),
        route(Method::GET, "/api/v3/ptz/protocols"),
        // --- Groups ---
        route(Method::GET, "/api/v3/groups"),
        route(Method::GET, "/api/v3/groups-monitors"),
        route(Method::GET, "/api/v3/groups-permissions"),
        // --- Devices ---
        route(Method::GET, "/api/v3/devices"),
        route(Method::GET, "/api/v3/manufacturers"),
        route(Method::GET, "/api/v3/models"),
        // --- Snapshots ---
        route(Method::GET, "/api/v3/snapshots"),
        route(Method::GET, "/api/v3/snapshots-events"),
        // --- Stream ---
        route(Method::GET, "/api/v3/live/sessions"),
    ]
}

/// An access token whose signature is valid but which expired in 1970.
fn expired_token() -> String {
    UserClaims {
        iat: 0,
        exp: 100,
        user: "expired".to_string(),
        uid: 1,
        perms: UserPermissions::superuser(),
    }
    .encode(&zm_api::constant::ACCESS_TOKEN_ENCODE_KEY)
    .expect("encode expired token")
}

#[tokio::test]
async fn protected_routes_reject_missing_token() {
    let app = TestApp::mock();
    for r in protected_routes() {
        let resp = app.request(r.method.clone(), r.path).send().await;
        assert_error(&resp, StatusCode::UNAUTHORIZED, "UNAUTHORIZED_ERROR");
    }
}

#[tokio::test]
async fn protected_routes_reject_malformed_token() {
    let app = TestApp::mock();
    for r in protected_routes() {
        let resp = app
            .request(r.method.clone(), r.path)
            .bearer("this-is-not-a-jwt")
            .send()
            .await;
        assert_eq!(
            resp.status(),
            StatusCode::UNAUTHORIZED,
            "{} {} should reject a malformed token",
            r.method,
            r.path
        );
    }
}

#[tokio::test]
async fn protected_routes_reject_non_bearer_scheme() {
    let app = TestApp::mock();
    for r in protected_routes() {
        let resp = app
            .request(r.method.clone(), r.path)
            .raw_auth("Basic dXNlcjpwYXNz")
            .send()
            .await;
        assert_eq!(
            resp.status(),
            StatusCode::UNAUTHORIZED,
            "{} {} should reject a non-Bearer Authorization scheme",
            r.method,
            r.path
        );
    }
}

#[tokio::test]
async fn protected_routes_reject_expired_token() {
    let app = TestApp::mock();
    let token = expired_token();
    for r in protected_routes() {
        let resp = app
            .request(r.method.clone(), r.path)
            .bearer(&token)
            .send()
            .await;
        assert_eq!(
            resp.status(),
            StatusCode::UNAUTHORIZED,
            "{} {} should reject an expired token",
            r.method,
            r.path
        );
    }
}

#[tokio::test]
async fn protected_routes_reject_insufficient_permissions() {
    // A valid token carrying no permissions: authenticated, but RBAC must
    // still deny every protected route with 403.
    let app = TestApp::mock();
    let token = token_with(UserPermissions::default());
    for r in protected_routes() {
        let resp = app
            .request(r.method.clone(), r.path)
            .bearer(&token)
            .send()
            .await;
        assert_error(&resp, StatusCode::FORBIDDEN, "PERMISSION_DENIED_ERROR");
    }
}

#[tokio::test]
async fn public_routes_need_no_token() {
    // Counter-check: login, health check and version are intentionally public.
    let app = TestApp::mock();
    for path in ["/api/v3/server/health_check", "/api/v3/host/getVersion"] {
        let resp = app.request(Method::GET, path).send().await;
        assert_ne!(
            resp.status(),
            StatusCode::UNAUTHORIZED,
            "{path} is meant to be public"
        );
        assert_ne!(
            resp.status(),
            StatusCode::FORBIDDEN,
            "{path} is meant to be public"
        );
    }
}

#[tokio::test]
async fn superuser_token_clears_auth_and_rbac() {
    // Sanity check the positive path: a fully-privileged token is never
    // rejected by the auth or RBAC layers.
    let app = TestApp::mock();
    let token = superuser_token();
    for r in protected_routes() {
        // Skip the destructive system endpoint in the positive sweep.
        if r.path == "/api/v3/system/shutdown" {
            continue;
        }
        let resp = app
            .request(r.method.clone(), r.path)
            .bearer(&token)
            .send()
            .await;
        assert_ne!(
            resp.status(),
            StatusCode::UNAUTHORIZED,
            "{} {} rejected a superuser token",
            r.method,
            r.path
        );
        assert_ne!(
            resp.status(),
            StatusCode::FORBIDDEN,
            "{} {} rejected a superuser token",
            r.method,
            r.path
        );
    }
}
