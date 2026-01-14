//! Request DTOs for daemon controller API.

use serde::Deserialize;
use utoipa::ToSchema;

/// Request to start a daemon.
#[derive(Debug, Deserialize, ToSchema)]
pub struct StartDaemonRequest {
    /// Optional additional arguments
    #[serde(default)]
    pub args: Vec<String>,
}

/// Request to apply a system state.
#[derive(Debug, Deserialize, ToSchema)]
pub struct ApplyStateRequest {
    /// Name of the state to apply
    pub state_name: String,
}
