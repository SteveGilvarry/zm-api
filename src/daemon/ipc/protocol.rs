//! IPC protocol definitions for daemon controller communication.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::daemon::ProcessState;

/// Commands that can be sent to the daemon controller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DaemonCommand {
    // System commands
    /// Start the daemon controller server
    Startup,
    /// Stop all daemons and shut down
    Shutdown,
    /// Get status of all daemons
    Status,
    /// Simple health check
    Check,
    /// Rotate log files
    LogRot,
    /// Get version info
    Version,

    // Individual daemon commands
    /// Start a specific daemon
    Start { daemon: String, args: Vec<String> },
    /// Stop a specific daemon
    Stop { daemon: String },
    /// Restart a specific daemon
    Restart { daemon: String },
    /// Send SIGHUP to reload configuration
    Reload { daemon: String },

    // Package-level commands (zmpkg.pl compatibility)
    /// Full system startup (verify folders, start zmdc, start all daemons)
    PackageStart,
    /// Full system shutdown
    PackageStop,
    /// Full system restart
    PackageRestart,
    /// Apply a named system state
    ApplyState { state_name: String },
}

impl DaemonCommand {
    /// Parse a command from the legacy text protocol.
    ///
    /// Format: "command;arg1;arg2;..."
    pub fn parse_legacy(input: &str) -> Result<Self, String> {
        let parts: Vec<&str> = input.trim().split(';').collect();
        if parts.is_empty() {
            return Err("Empty command".to_string());
        }

        let cmd = parts[0].to_lowercase();
        match cmd.as_str() {
            "startup" => Ok(DaemonCommand::Startup),
            "shutdown" => Ok(DaemonCommand::Shutdown),
            "status" => Ok(DaemonCommand::Status),
            "check" => Ok(DaemonCommand::Check),
            "logrot" => Ok(DaemonCommand::LogRot),
            "version" => Ok(DaemonCommand::Version),
            "start" => {
                if parts.len() < 2 {
                    return Err("start requires daemon name".to_string());
                }
                Ok(DaemonCommand::Start {
                    daemon: parts[1].to_string(),
                    args: parts[2..].iter().map(|s| s.to_string()).collect(),
                })
            }
            "stop" => {
                if parts.len() < 2 {
                    return Err("stop requires daemon name".to_string());
                }
                Ok(DaemonCommand::Stop {
                    daemon: parts[1].to_string(),
                })
            }
            "restart" => {
                if parts.len() < 2 {
                    return Err("restart requires daemon name".to_string());
                }
                Ok(DaemonCommand::Restart {
                    daemon: parts[1].to_string(),
                })
            }
            "reload" => {
                if parts.len() < 2 {
                    return Err("reload requires daemon name".to_string());
                }
                Ok(DaemonCommand::Reload {
                    daemon: parts[1].to_string(),
                })
            }
            "pkg_start" => Ok(DaemonCommand::PackageStart),
            "pkg_stop" => Ok(DaemonCommand::PackageStop),
            "pkg_restart" => Ok(DaemonCommand::PackageRestart),
            "state" => {
                if parts.len() < 2 {
                    return Err("state requires state name".to_string());
                }
                Ok(DaemonCommand::ApplyState {
                    state_name: parts[1].to_string(),
                })
            }
            _ => Err(format!("Unknown command: {}", cmd)),
        }
    }

    /// Format the command for the legacy text protocol.
    pub fn to_legacy(&self) -> String {
        match self {
            DaemonCommand::Startup => "startup".to_string(),
            DaemonCommand::Shutdown => "shutdown".to_string(),
            DaemonCommand::Status => "status".to_string(),
            DaemonCommand::Check => "check".to_string(),
            DaemonCommand::LogRot => "logrot".to_string(),
            DaemonCommand::Version => "version".to_string(),
            DaemonCommand::Start { daemon, args } => {
                if args.is_empty() {
                    format!("start;{}", daemon)
                } else {
                    format!("start;{};{}", daemon, args.join(";"))
                }
            }
            DaemonCommand::Stop { daemon } => format!("stop;{}", daemon),
            DaemonCommand::Restart { daemon } => format!("restart;{}", daemon),
            DaemonCommand::Reload { daemon } => format!("reload;{}", daemon),
            DaemonCommand::PackageStart => "pkg_start".to_string(),
            DaemonCommand::PackageStop => "pkg_stop".to_string(),
            DaemonCommand::PackageRestart => "pkg_restart".to_string(),
            DaemonCommand::ApplyState { state_name } => format!("state;{}", state_name),
        }
    }
}

/// Response from the daemon controller.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DaemonResponse {
    /// Whether the command succeeded
    pub success: bool,
    /// Human-readable message
    pub message: String,
    /// Optional additional data (JSON)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl DaemonResponse {
    /// Create a success response.
    pub fn ok(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
            data: None,
        }
    }

    /// Create a success response with data.
    pub fn ok_with_data(message: impl Into<String>, data: impl Serialize) -> Self {
        Self {
            success: true,
            message: message.into(),
            data: serde_json::to_value(data).ok(),
        }
    }

    /// Create an error response.
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
            data: None,
        }
    }

    /// Format for legacy text protocol.
    pub fn to_legacy(&self) -> String {
        if self.success {
            format!("OK;{}", self.message)
        } else {
            format!("ERR;{}", self.message)
        }
    }

    /// Parse from legacy text protocol.
    pub fn parse_legacy(input: &str) -> Self {
        let parts: Vec<&str> = input.trim().splitn(2, ';').collect();
        if parts.is_empty() {
            return Self::error("Empty response");
        }

        let success = parts[0].eq_ignore_ascii_case("ok");
        let message = parts.get(1).unwrap_or(&"").to_string();

        Self {
            success,
            message,
            data: None,
        }
    }
}

/// Status information for a single process.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ProcessStatus {
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

/// System-wide status.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SystemStatus {
    /// Whether the daemon controller is running
    pub running: bool,
    /// Status of all managed daemons
    pub daemons: Vec<ProcessStatus>,
    /// System statistics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stats: Option<SystemStats>,
}

/// System statistics.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SystemStats {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_legacy_simple_commands() {
        assert_eq!(
            DaemonCommand::parse_legacy("startup").unwrap(),
            DaemonCommand::Startup
        );
        assert_eq!(
            DaemonCommand::parse_legacy("SHUTDOWN").unwrap(),
            DaemonCommand::Shutdown
        );
        assert_eq!(
            DaemonCommand::parse_legacy("status").unwrap(),
            DaemonCommand::Status
        );
        assert_eq!(
            DaemonCommand::parse_legacy("check").unwrap(),
            DaemonCommand::Check
        );
    }

    #[test]
    fn test_parse_legacy_start_command() {
        let cmd = DaemonCommand::parse_legacy("start;zmc;-m;1").unwrap();
        assert_eq!(
            cmd,
            DaemonCommand::Start {
                daemon: "zmc".to_string(),
                args: vec!["-m".to_string(), "1".to_string()],
            }
        );
    }

    #[test]
    fn test_parse_legacy_stop_command() {
        let cmd = DaemonCommand::parse_legacy("stop;zmfilter.pl").unwrap();
        assert_eq!(
            cmd,
            DaemonCommand::Stop {
                daemon: "zmfilter.pl".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_legacy_state_command() {
        let cmd = DaemonCommand::parse_legacy("state;default").unwrap();
        assert_eq!(
            cmd,
            DaemonCommand::ApplyState {
                state_name: "default".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_legacy_error() {
        assert!(DaemonCommand::parse_legacy("").is_err());
        assert!(DaemonCommand::parse_legacy("unknown").is_err());
        assert!(DaemonCommand::parse_legacy("start").is_err()); // Missing daemon
    }

    #[test]
    fn test_command_roundtrip() {
        let commands = vec![
            DaemonCommand::Startup,
            DaemonCommand::Status,
            DaemonCommand::Start {
                daemon: "zmc".to_string(),
                args: vec!["-m".to_string(), "5".to_string()],
            },
            DaemonCommand::Stop {
                daemon: "zmfilter.pl".to_string(),
            },
            DaemonCommand::ApplyState {
                state_name: "default".to_string(),
            },
        ];

        for cmd in commands {
            let legacy = cmd.to_legacy();
            let parsed = DaemonCommand::parse_legacy(&legacy).unwrap();
            assert_eq!(cmd, parsed);
        }
    }

    #[test]
    fn test_response_ok() {
        let resp = DaemonResponse::ok("Started successfully");
        assert!(resp.success);
        assert_eq!(resp.message, "Started successfully");
        assert_eq!(resp.to_legacy(), "OK;Started successfully");
    }

    #[test]
    fn test_response_error() {
        let resp = DaemonResponse::error("Process not found");
        assert!(!resp.success);
        assert_eq!(resp.to_legacy(), "ERR;Process not found");
    }

    #[test]
    fn test_response_roundtrip() {
        let resp = DaemonResponse::ok("Test message");
        let legacy = resp.to_legacy();
        let parsed = DaemonResponse::parse_legacy(&legacy);
        assert_eq!(resp.success, parsed.success);
        assert_eq!(resp.message, parsed.message);
    }
}
