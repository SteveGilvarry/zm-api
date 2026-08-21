//! Server-side token revocation coverage.
//!
//! Runs against the mock-database harness: the revocation check in the RBAC
//! layer is purely in-memory, and logout's single `UPDATE Users` is served by
//! a mock exec result — no real database needed.

mod common;

use axum::http::{Method, StatusCode};
use common::harness::{token_for, TestApp};
use sea_orm::MockExecResult;
use zm_api::util::authz::UserPermissions;

/// Distinct uids per test: the revocation registry is per-state, but distinct
/// ids keep the tests independent of each other's minting time.
const REVOKED_UID: u32 = 991_001_001;
const LOGOUT_UID: u32 = 991_001_002;

#[tokio::test]
async fn revoked_token_is_rejected_on_protected_routes() {
    let state = common::create_test_state();
    // Mint first, then raise the floor above the token's iat.
    let token = token_for(REVOKED_UID, UserPermissions::superuser());
    state
        .revocations
        .revoke(REVOKED_UID, chrono::Utc::now().timestamp() + 1);
    let app = TestApp::from_state(state);

    let resp = app.get("/api/v3/monitors", &token).await;
    assert_eq!(
        resp.status,
        StatusCode::UNAUTHORIZED,
        "revoked token must be rejected, body: {}",
        resp.text()
    );

    // A different user with no revocation floor still clears the auth/RBAC
    // layer (the mock DB may then fail the query — anything but 401/403).
    let other = token_for(REVOKED_UID + 1, UserPermissions::superuser());
    let resp = app.get("/api/v3/monitors", &other).await;
    assert_ne!(resp.status, StatusCode::UNAUTHORIZED);
    assert_ne!(resp.status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn logout_revokes_the_current_token() {
    // One exec result for logout's `UPDATE Users SET TokenMinExpiry`.
    let state = common::create_test_state_with_exec(vec![MockExecResult {
        last_insert_id: 0,
        rows_affected: 1,
    }]);
    let app = TestApp::from_state(state);
    let token = token_for(LOGOUT_UID, UserPermissions::superuser());

    // The token authenticates its own logout...
    let resp = app.get("/api/v3/auth/logout", &token).await;
    assert_eq!(resp.status, StatusCode::OK, "body: {}", resp.text());

    // ...and is dead immediately afterwards, on every protected surface.
    let resp = app.get("/api/v3/monitors", &token).await;
    assert_eq!(
        resp.status,
        StatusCode::UNAUTHORIZED,
        "post-logout use of the token must be rejected, body: {}",
        resp.text()
    );

    // Media-style query-parameter auth is rejected too.
    let resp = app
        .request(
            Method::GET,
            &format!("/api/v3/monitors/1/snapshot?token={token}"),
        )
        .send()
        .await;
    assert_eq!(resp.status, StatusCode::UNAUTHORIZED);
}
