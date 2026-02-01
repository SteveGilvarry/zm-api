//! Response DTOs for daemon controller API.

use serde::Serialize;
use utoipa::ToSchema;

use crate::daemon::ipc::{ProcessStatus, SystemStats, SystemStatus};
use crate::daemon::ProcessState;

/// Response containing a single daemon's status.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct DaemonStatusResponse {
    /// Process identifier
    pub id: String,
    /// Display name
    pub name: String,
    /// Current state
    pub state: ProcessState,
    /// Process ID if running
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
    /// Uptime in seconds if running
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uptime_seconds: Option<u64>,
    /// Number of restart attempts
    pub restart_count: u32,
    /// Associated monitor ID if applicable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monitor_id: Option<u32>,
}

impl From<ProcessStatus> for DaemonStatusResponse {
    fn from(status: ProcessStatus) -> Self {
        Self {
            id: status.id,
            name: status.name,
            state: status.state,
            pid: status.pid,
            uptime_seconds: status.uptime_seconds,
            restart_count: status.restart_count,
            monitor_id: status.monitor_id,
        }
    }
}

/// Response containing list of all daemons.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct DaemonListResponse {
    /// List of daemon statuses
    pub daemons: Vec<DaemonStatusResponse>,
}

/// Response containing system-wide status.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct SystemStatusResponse {
    /// Whether the daemon controller is running
    pub running: bool,
    /// Status of all managed daemons
    pub daemons: Vec<DaemonStatusResponse>,
    /// System statistics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stats: Option<SystemStatsResponse>,
}

impl From<SystemStatus> for SystemStatusResponse {
    fn from(status: SystemStatus) -> Self {
        Self {
            running: status.running,
            daemons: status.daemons.into_iter().map(Into::into).collect(),
            stats: status.stats.map(Into::into),
        }
    }
}

/// Response containing system statistics.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct SystemStatsResponse {
    /// CPU load average (1 minute)
    pub cpu_load: f64,
    /// CPU usage percentage
    pub cpu_usage_percent: f64,
    /// Total memory in bytes
    pub total_mem: u64,
    /// Free memory in bytes
    pub free_mem: u64,
    /// Total swap in bytes
    pub total_swap: u64,
    /// Free swap in bytes
    pub free_swap: u64,
    /// Total disk space in bytes (root filesystem)
    pub total_disk: u64,
    /// Used disk space in bytes
    pub used_disk: u64,
    /// Free disk space in bytes
    pub free_disk: u64,
    /// Disk usage percentage
    pub disk_usage_percent: f64,
}

impl From<SystemStats> for SystemStatsResponse {
    fn from(stats: SystemStats) -> Self {
        Self {
            cpu_load: stats.cpu_load,
            cpu_usage_percent: stats.cpu_usage_percent,
            total_mem: stats.total_mem,
            free_mem: stats.free_mem,
            total_swap: stats.total_swap,
            free_swap: stats.free_swap,
            total_disk: stats.total_disk,
            used_disk: stats.used_disk,
            free_disk: stats.free_disk,
            disk_usage_percent: stats.disk_usage_percent,
        }
    }
}

/// Response for daemon control actions.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct DaemonActionResponse {
    /// Whether the action succeeded
    pub success: bool,
    /// Human-readable message
    pub message: String,
}

impl DaemonActionResponse {
    /// Create a success response.
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
        }
    }

    /// Create an error response.
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
        }
    }
}
