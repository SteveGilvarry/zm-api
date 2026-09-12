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
    /// Status of every daemon, or of one (`zmdc.pl status zmc -m 1`).
    Status { target: Option<String> },
    /// running / pending / stopped / unknown for one daemon, or whether the
    /// controller itself is up when no target is given.
    Check { target: Option<String> },
    /// Rotate log files
    LogRot,
    /// Get version info
    Version,

    // Individual daemon commands
    /// Start a specific daemon
    Start { daemon: String, args: Vec<String> },
    /// Stop a specific daemon
    Stop { daemon: String, args: Vec<String> },
    /// Restart a specific daemon
    Restart { daemon: String, args: Vec<String> },
    /// Send SIGHUP to reload configuration
    Reload { daemon: String, args: Vec<String> },

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

/// The one key a daemon is tracked under: `"<command> <args>"`, the same
/// string zmdc.pl builds with `join(' ', $daemon, @args)`. `zmdc.pl stop zmc
/// -m 1` arrives as daemon "zmc" + args ["-m", "1"] and has to find the entry
/// `start_all_daemons` created as "zmc -m 1", not a second one under "zmc"
/// (#84).
pub fn canonical_daemon_id<S: AsRef<str>>(daemon: &str, args: &[S]) -> String {
    let mut id = daemon.trim().to_string();
    for a in args {
        let a = a.as_ref().trim();
        if !a.is_empty() {
            id.push(' ');
            id.push_str(a);
        }
    }
    id
}

/// `status;zmc;-m;1` → `Some("zmc -m 1")`; bare `status` → `None`.
fn legacy_target(parts: &[&str]) -> Option<String> {
    match parts.get(1) {
        Some(daemon) if !daemon.trim().is_empty() => Some(canonical_daemon_id(daemon, &parts[2..])),
        _ => None,
    }
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
            "status" => Ok(DaemonCommand::Status {
                target: legacy_target(&parts),
            }),
            "check" => Ok(DaemonCommand::Check {
                target: legacy_target(&parts),
            }),
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
                    args: parts[2..].iter().map(|s| s.to_string()).collect(),
                })
            }
            "restart" => {
                if parts.len() < 2 {
                    return Err("restart requires daemon name".to_string());
                }
                Ok(DaemonCommand::Restart {
                    daemon: parts[1].to_string(),
                    args: parts[2..].iter().map(|s| s.to_string()).collect(),
                })
            }
            "reload" => {
                if parts.len() < 2 {
                    return Err("reload requires daemon name".to_string());
                }
                Ok(DaemonCommand::Reload {
                    daemon: parts[1].to_string(),
                    args: parts[2..].iter().map(|s| s.to_string()).collect(),
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
            DaemonCommand::Status { target } => with_target("status", target.as_deref()),
            DaemonCommand::Check { target } => with_target("check", target.as_deref()),
            DaemonCommand::LogRot => "logrot".to_string(),
            DaemonCommand::Version => "version".to_string(),
            DaemonCommand::Start { daemon, args } => {
                if args.is_empty() {
                    format!("start;{}", daemon)
                } else {
                    format!("start;{};{}", daemon, args.join(";"))
                }
            }
            DaemonCommand::Stop { daemon, args } => {
                if args.is_empty() {
                    format!("stop;{}", daemon)
                } else {
                    format!("stop;{};{}", daemon, args.join(";"))
                }
            }
            DaemonCommand::Restart { daemon, args } => {
                if args.is_empty() {
                    format!("restart;{}", daemon)
                } else {
                    format!("restart;{};{}", daemon, args.join(";"))
                }
            }
            DaemonCommand::Reload { daemon, args } => {
                if args.is_empty() {
                    format!("reload;{}", daemon)
                } else {
                    format!("reload;{};{}", daemon, args.join(";"))
                }
            }
            DaemonCommand::PackageStart => "pkg_start".to_string(),
            DaemonCommand::PackageStop => "pkg_stop".to_string(),
            DaemonCommand::PackageRestart => "pkg_restart".to_string(),
            DaemonCommand::ApplyState { state_name } => format!("state;{}", state_name),
        }
    }
}

fn with_target(verb: &str, target: Option<&str>) -> String {
    match target {
        Some(t) => format!(
            "{verb};{}",
            t.split_whitespace().collect::<Vec<_>>().join(";")
        ),
        None => verb.to_string(),
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
    /// What a text-protocol client is sent instead of `OK;<message>`. zmdc.pl
    /// clients — ZoneMinder's web console included — parse `check` and
    /// `status` output by its exact words and line shape (#83).
    #[serde(skip)]
    #[schema(ignore)]
    pub legacy_text: Option<String>,
}

impl DaemonResponse {
    /// Create a success response.
    pub fn ok(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
            data: None,
            legacy_text: None,
        }
    }

    /// What text-protocol clients receive verbatim (JSON clients are unaffected).
    pub fn with_legacy_text(mut self, text: impl Into<String>) -> Self {
        self.legacy_text = Some(text.into());
        self
    }

    /// Create a success response with data.
    pub fn ok_with_data(message: impl Into<String>, data: impl Serialize) -> Self {
        Self {
            success: true,
            message: message.into(),
            data: serde_json::to_value(data).ok(),
            legacy_text: None,
        }
    }

    /// Create an error response.
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
            data: None,
            legacy_text: None,
        }
    }

    /// Format for legacy text protocol.
    pub fn to_legacy(&self) -> String {
        if let Some(text) = &self.legacy_text {
            return text.clone();
        }
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
            legacy_text: None,
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
    /// CPU usage percentage (overall)
    pub cpu_usage_percent: f64,
    /// CPU user percentage
    pub cpu_user_percent: f64,
    /// CPU nice percentage
    pub cpu_nice_percent: f64,
    /// CPU system percentage
    pub cpu_system_percent: f64,
    /// CPU idle percentage
    pub cpu_idle_percent: f64,
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
            DaemonCommand::Status { target: None }
        );
        assert_eq!(
            DaemonCommand::parse_legacy("check").unwrap(),
            DaemonCommand::Check { target: None }
        );
    }

    /// zmdc.pl sends `check;zmc;-m;1` and `status;zmc;-m;1` for one daemon;
    /// the target must be the daemon's map key (#83).
    #[test]
    fn status_and_check_take_a_daemon_target() {
        assert_eq!(
            DaemonCommand::parse_legacy("check;zmc;-m;1").unwrap(),
            DaemonCommand::Check {
                target: Some("zmc -m 1".to_string())
            }
        );
        assert_eq!(
            DaemonCommand::parse_legacy("status;zmfilter.pl").unwrap(),
            DaemonCommand::Status {
                target: Some("zmfilter.pl".to_string())
            }
        );
    }

    /// `zmdc.pl stop zmc -m 1` carries the args; dropping them left the
    /// daemon name alone, which matched no tracked entry (#84).
    #[test]
    fn stop_and_reload_keep_their_args() {
        assert_eq!(
            DaemonCommand::parse_legacy("stop;zmc;-m;1").unwrap(),
            DaemonCommand::Stop {
                daemon: "zmc".to_string(),
                args: vec!["-m".to_string(), "1".to_string()],
            }
        );
        assert_eq!(canonical_daemon_id("zmc", &["-m", "1"]), "zmc -m 1");
        assert_eq!(
            canonical_daemon_id("zmfilter.pl", &[] as &[&str]),
            "zmfilter.pl"
        );
        assert_eq!(canonical_daemon_id(" zma ", &["-m", " 7 "]), "zma -m 7");
    }

    #[test]
    fn legacy_text_overrides_the_ok_prefix() {
        let resp = DaemonResponse::ok("running").with_legacy_text("running");
        assert_eq!(resp.to_legacy(), "running");
        assert_eq!(DaemonResponse::ok("running").to_legacy(), "OK;running");
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
                args: vec![],
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
            DaemonCommand::Status { target: None },
            DaemonCommand::Status {
                target: Some("zmc -m 5".to_string()),
            },
            DaemonCommand::Check {
                target: Some("zma -m 5".to_string()),
            },
            DaemonCommand::Start {
                daemon: "zmc".to_string(),
                args: vec!["-m".to_string(), "5".to_string()],
            },
            DaemonCommand::Stop {
                daemon: "zmfilter.pl".to_string(),
                args: vec![],
            },
            DaemonCommand::Stop {
                daemon: "zmc".to_string(),
                args: vec!["-m".to_string(), "5".to_string()],
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
