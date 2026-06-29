//! ONVIF camera discovery routes.
//!
//! Wires the two discovery handlers under the `/api/v3/discovery` prefix:
//!
//! - `POST /api/v3/discovery/probe`   — WS-Discovery probe of the local network.
//! - `POST /api/v3/discovery/inspect` — directed Device + Media query of one device.
//!
//! Like the monitor and PTZ route modules, this applies `auth_middleware` so the
//! `MonitorScope` extractor in the handlers has a populated auth context. The
//! handlers additionally require unrestricted monitor access (discovery is a
//! whole-network operation and a stepping stone to minting monitors), and the
//! main session wraps this router with feature-level RBAC via `protect`.

use crate::handlers::discovery;
use crate::server::state::AppState;
use crate::util::middleware::auth_middleware;
use axum::{middleware, routing::post, Router};
use tracing::info;

pub fn add_discovery_routes(router: Router<AppState>) -> Router<AppState> {
    info!("Registering routes for ONVIF discovery...");

    let api_prefix = "/api/v3";

    let protected_routes = Router::new()
        .route(
            &format!("{}/discovery/probe", api_prefix),
            post(discovery::probe),
        )
        .route(
            &format!("{}/discovery/inspect", api_prefix),
            post(discovery::inspect),
        )
        .route(
            &format!("{}/discovery/onboard", api_prefix),
            post(discovery::onboard),
        )
        .layer(middleware::from_fn(auth_middleware));

    router.merge(protected_routes)
}
