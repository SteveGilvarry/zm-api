//! Command parsing and handling for the daemon controller.

use crate::daemon::ipc::{DaemonCommand, DaemonResponse};

/// Parse a command from raw input (auto-detects format).
///
/// Supports both legacy text format (semicolon-separated) and JSON.
pub fn parse_command(input: &str) -> Result<DaemonCommand, String> {
    let input = input.trim();

    // Try JSON first if it looks like JSON
    if input.starts_with('{') {
        parse_json_command(input)
    } else {
        DaemonCommand::parse_legacy(input)
    }
}

/// Parse a command from JSON format.
fn parse_json_command(input: &str) -> Result<DaemonCommand, String> {
    #[derive(serde::Deserialize)]
    struct JsonCommand {
        command: String,
        #[serde(default)]
        daemon: Option<String>,
        #[serde(default)]
        args: Vec<String>,
        #[serde(default)]
        state_name: Option<String>,
    }

    let json: JsonCommand =
        serde_json::from_str(input).map_err(|e| format!("Invalid JSON: {}", e))?;

    match json.command.to_lowercase().as_str() {
        "startup" => Ok(DaemonCommand::Startup),
        "shutdown" => Ok(DaemonCommand::Shutdown),
        "status" => Ok(DaemonCommand::Status),
        "check" => Ok(DaemonCommand::Check),
        "logrot" => Ok(DaemonCommand::LogRot),
        "version" => Ok(DaemonCommand::Version),
        "start" => {
            let daemon = json.daemon.ok_or("start requires daemon field")?;
            Ok(DaemonCommand::Start {
                daemon,
                args: json.args,
            })
        }
        "stop" => {
            let daemon = json.daemon.ok_or("stop requires daemon field")?;
            Ok(DaemonCommand::Stop { daemon })
        }
        "restart" => {
            let daemon = json.daemon.ok_or("restart requires daemon field")?;
            Ok(DaemonCommand::Restart { daemon })
        }
        "reload" => {
            let daemon = json.daemon.ok_or("reload requires daemon field")?;
            Ok(DaemonCommand::Reload { daemon })
        }
        "pkg_start" | "package_start" => Ok(DaemonCommand::PackageStart),
        "pkg_stop" | "package_stop" => Ok(DaemonCommand::PackageStop),
        "pkg_restart" | "package_restart" => Ok(DaemonCommand::PackageRestart),
        "state" | "apply_state" => {
            let state_name = json.state_name.ok_or("state requires state_name field")?;
            Ok(DaemonCommand::ApplyState { state_name })
        }
        _ => Err(format!("Unknown command: {}", json.command)),
    }
}

/// Format a response for output (auto-detects preferred format based on request).
pub fn format_response(response: &DaemonResponse, json_format: bool) -> String {
    if json_format {
        serde_json::to_string(response).unwrap_or_else(|_| response.to_legacy())
    } else {
        response.to_legacy()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_legacy_format() {
        let cmd = parse_command("start;zmc;-m;1").unwrap();
        assert_eq!(
            cmd,
            DaemonCommand::Start {
                daemon: "zmc".to_string(),
                args: vec!["-m".to_string(), "1".to_string()],
            }
        );
    }

    #[test]
    fn test_parse_json_format() {
        let json = r#"{"command": "start", "daemon": "zmc", "args": ["-m", "1"]}"#;
        let cmd = parse_command(json).unwrap();
        assert_eq!(
            cmd,
            DaemonCommand::Start {
                daemon: "zmc".to_string(),
                args: vec!["-m".to_string(), "1".to_string()],
            }
        );
    }

    #[test]
    fn test_parse_json_simple() {
        let json = r#"{"command": "status"}"#;
        let cmd = parse_command(json).unwrap();
        assert_eq!(cmd, DaemonCommand::Status);
    }

    #[test]
    fn test_parse_json_state() {
        let json = r#"{"command": "apply_state", "state_name": "default"}"#;
        let cmd = parse_command(json).unwrap();
        assert_eq!(
            cmd,
            DaemonCommand::ApplyState {
                state_name: "default".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_json_error_missing_daemon() {
        let json = r#"{"command": "start"}"#;
        assert!(parse_command(json).is_err());
    }

    #[test]
    fn test_format_response_legacy() {
        let resp = DaemonResponse::ok("Success");
        let formatted = format_response(&resp, false);
        assert_eq!(formatted, "OK;Success");
    }

    #[test]
    fn test_format_response_json() {
        let resp = DaemonResponse::ok("Success");
        let formatted = format_response(&resp, true);
        assert!(formatted.contains("\"success\":true"));
        assert!(formatted.contains("\"message\":\"Success\""));
    }
}
