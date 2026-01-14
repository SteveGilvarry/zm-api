//! Routes for daemon controller API.

use axum::{
    routing::{get, post},
    Router,
};

use crate::handlers::daemon;
use crate::server::state::AppState;

/// Add daemon routes to a router.
pub fn add_daemon_routes(router: Router<AppState>) -> Router<AppState> {
    router
        // Daemon management
        .route("/api/v3/daemons", get(daemon::list_daemons))
        .route("/api/v3/daemons/{id}", get(daemon::get_daemon))
        .route("/api/v3/daemons/{id}/start", post(daemon::start_daemon))
        .route("/api/v3/daemons/{id}/stop", post(daemon::stop_daemon))
        .route("/api/v3/daemons/{id}/restart", post(daemon::restart_daemon))
        .route("/api/v3/daemons/{id}/reload", post(daemon::reload_daemon))
        // System management
        .route("/api/v3/system/status", get(daemon::get_system_status))
        .route("/api/v3/system/startup", post(daemon::system_startup))
        .route("/api/v3/system/shutdown", post(daemon::system_shutdown))
        .route("/api/v3/system/restart", post(daemon::system_restart))
        .route("/api/v3/system/logrot", post(daemon::system_logrot))
        .route("/api/v3/system/state", post(daemon::apply_state))
}
