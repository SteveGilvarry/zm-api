use axum::routing::{get, post};

use crate::handlers::auth;
use crate::server::state::AppState;
use tracing::info;

pub fn add_routers(router: axum::Router<AppState>) -> axum::Router<AppState> {
    info!("Registering routes for auth...");

    router
        // Login and issue a JWT token
        .route("/api/v3/auth/login", post(auth::login))
        // Refresh an expired or expiring token using a refresh token
        .route("/api/v3/auth/refresh", post(auth::refresh_token))
        // Logout to invalidate the current session
        .route("/api/v3/auth/logout", get(auth::logout))
}
