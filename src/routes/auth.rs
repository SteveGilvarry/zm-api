use axum::middleware;
use axum::routing::{get, post, put};

use crate::handlers::auth;
use crate::server::state::AppState;
use crate::util::middleware::authenticated_middleware;
use tracing::info;

pub fn add_routers(router: axum::Router<AppState>, state: AppState) -> axum::Router<AppState> {
    info!("Registering routes for auth...");

    // These routes carry their own authentication (not `authz::protect`, which
    // requires a permission feature). `authenticated_middleware` is state-aware
    // so it enforces token revocation and populates `UserClaims`.
    let authed = || middleware::from_fn_with_state(state.clone(), authenticated_middleware);

    router
        // Login and issue a JWT token
        .route("/api/v3/auth/login", post(auth::login))
        // Refresh an expired or expiring token using a refresh token
        .route("/api/v3/auth/refresh", post(auth::refresh_token))
        // Logout to invalidate the current session
        .route("/api/v3/auth/logout", get(auth::logout).layer(authed()))
        // Self-service account endpoints: any authenticated user may read
        // their own account and rotate their own password.
        .route("/api/v3/me", get(auth::me).layer(authed()))
        .route(
            "/api/v3/me/password",
            put(auth::change_password).layer(authed()),
        )
}
