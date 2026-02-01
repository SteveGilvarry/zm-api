//! Perl bridge for PTZ control via zmcontrol.pl
//!
//! This module provides PTZ control by communicating with ZoneMinder's zmcontrol.pl daemon.
//! It uses a two-tier approach matching ZoneMinder's PHP implementation:
//!
//! 1. **Primary**: Unix socket connection to the zmcontrol daemon (faster, no process spawn)
//! 2. **Fallback**: Direct execution of zmcontrol.pl if socket connection fails

use async_trait::async_trait;
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::process::Command;
use tracing::{debug, error, info, instrument, warn};

use super::capabilities::PtzCapabilities;
use super::error::{PtzError, PtzResult};
use super::traits::{
    AbsolutePosition, MoveParams, PtzCommand, PtzCommandResult, PtzConnectionConfig, PtzControl,
    PtzControlFactory, ZoomParams,
};

/// Default path to zmcontrol.pl
const ZMCONTROL_PATH: &str = "/usr/bin/zmcontrol.pl";

/// Default path for zmcontrol sockets
const ZM_PATH_SOCKS: &str = "/run/zm";

/// Timeout for command execution in seconds
const COMMAND_TIMEOUT_SECS: u64 = 30;

/// Timeout for socket connection in milliseconds
const SOCKET_CONNECT_TIMEOUT_MS: u64 = 1000;

/// Perl-based PTZ control proxy that executes zmcontrol.pl
///
/// Uses Unix socket communication as the primary method (matching ZoneMinder's PHP),
/// with fallback to direct process execution if the socket is unavailable.
pub struct PerlControlProxy {
    config: PtzConnectionConfig,
    capabilities: PtzCapabilities,
    zmcontrol_path: String,
    socket_path: PathBuf,
}

impl PerlControlProxy {
    /// Create a new Perl control proxy
    pub fn new(
        config: PtzConnectionConfig,
        capabilities: PtzCapabilities,
        zmcontrol_path: Option<String>,
        socket_dir: Option<String>,
    ) -> Self {
        let monitor_id = config.monitor_id;
        let sock_dir = socket_dir.unwrap_or_else(|| ZM_PATH_SOCKS.to_string());
        let socket_path = PathBuf::from(&sock_dir).join(format!("zmcontrol-{}.sock", monitor_id));

        Self {
            config,
            capabilities,
            zmcontrol_path: zmcontrol_path.unwrap_or_else(|| ZMCONTROL_PATH.to_string()),
            socket_path,
        }
    }

    /// Build JSON options for socket communication
    /// This matches the format expected by zmcontrol.pl daemon
    fn build_json_options(&self, command: &PtzCommand) -> serde_json::Value {
        let mut options: HashMap<String, serde_json::Value> = HashMap::new();

        // Add command name
        options.insert("command".to_string(), json!(command.zmcontrol_command()));

        // Add parameters based on command type
        match command {
            PtzCommand::MoveUp(params)
            | PtzCommand::MoveDown(params)
            | PtzCommand::MoveLeft(params)
            | PtzCommand::MoveRight(params)
            | PtzCommand::MoveUpLeft(params)
            | PtzCommand::MoveUpRight(params)
            | PtzCommand::MoveDownLeft(params)
            | PtzCommand::MoveDownRight(params) => {
                if let Some(pan_speed) = params.pan_speed {
                    options.insert(
                        "panspeed".to_string(),
                        json!(self.scale_speed(pan_speed, true)),
                    );
                }
                if let Some(tilt_speed) = params.tilt_speed {
                    options.insert(
                        "tiltspeed".to_string(),
                        json!(self.scale_speed(tilt_speed, false)),
                    );
                }
            }
            PtzCommand::ZoomIn(params) | PtzCommand::ZoomOut(params) => {
                if let Some(speed) = params.speed {
                    options.insert("speed".to_string(), json!(self.scale_zoom_speed(speed)));
                }
            }
            PtzCommand::GotoPreset { preset_id } => {
                options.insert("preset".to_string(), json!(preset_id));
            }
            PtzCommand::SetPreset { preset_id, name } => {
                options.insert("preset".to_string(), json!(preset_id));
                if let Some(n) = name {
                    options.insert("name".to_string(), json!(n));
                }
            }
            PtzCommand::ClearPreset { preset_id } => {
                options.insert("preset".to_string(), json!(preset_id));
            }
            PtzCommand::MoveAbsolute(pos) => {
                if let Some(pan) = pos.pan {
                    options.insert("panpos".to_string(), json!(pan));
                }
                if let Some(tilt) = pos.tilt {
                    options.insert("tiltpos".to_string(), json!(tilt));
                }
                if let Some(zoom) = pos.zoom {
                    options.insert("zoompos".to_string(), json!(zoom));
                }
            }
            PtzCommand::MoveRelative(pos) => {
                if let Some(pan) = pos.pan_delta {
                    options.insert("panstep".to_string(), json!(pan));
                }
                if let Some(tilt) = pos.tilt_delta {
                    options.insert("tiltstep".to_string(), json!(tilt));
                }
                if let Some(zoom) = pos.zoom_delta {
                    options.insert("zoomstep".to_string(), json!(zoom));
                }
            }
            _ => {}
        }

        // Add auto-stop if configured
        // PHP sets this to 1 (numeric) when flag is present without value
        if let Some(timeout) = self.config.auto_stop_timeout {
            if timeout > 0.0 {
                options.insert("autostop".to_string(), json!(1));
            }
        }

        json!(options)
    }

    /// Execute command via Unix socket (primary method)
    #[instrument(skip(self), fields(monitor_id = self.config.monitor_id, socket = %self.socket_path.display()))]
    async fn execute_via_socket(&self, command: &PtzCommand) -> PtzResult<PtzCommandResult> {
        let json_options = self.build_json_options(command);
        let json_string = json_options.to_string();

        debug!(json = %json_string, "Connecting to zmcontrol socket");

        // Try to connect with timeout
        let connect_result = tokio::time::timeout(
            std::time::Duration::from_millis(SOCKET_CONNECT_TIMEOUT_MS),
            UnixStream::connect(&self.socket_path),
        )
        .await;

        let mut stream = match connect_result {
            Ok(Ok(stream)) => stream,
            Ok(Err(e)) => {
                debug!(error = %e, "Socket connection failed");
                return Err(PtzError::PerlBridgeError(format!(
                    "Socket connection failed: {}",
                    e
                )));
            }
            Err(_) => {
                debug!("Socket connection timed out");
                return Err(PtzError::PerlBridgeError(
                    "Socket connection timed out".to_string(),
                ));
            }
        };

        // Write the JSON command
        if let Err(e) = stream.write_all(json_string.as_bytes()).await {
            return Err(PtzError::PerlBridgeError(format!(
                "Failed to write to socket: {}",
                e
            )));
        }

        // Shutdown write side to signal we're done sending
        if let Err(e) = stream.shutdown().await {
            warn!(error = %e, "Failed to shutdown socket write side");
        }

        // Read response (zmcontrol.pl may or may not send a response)
        let mut response = String::new();
        let read_result = tokio::time::timeout(
            std::time::Duration::from_secs(COMMAND_TIMEOUT_SECS),
            stream.read_to_string(&mut response),
        )
        .await;

        match read_result {
            Ok(Ok(_)) => {
                debug!(response = %response, "Received socket response");
                Ok(PtzCommandResult::success(if response.is_empty() {
                    "Command sent via socket"
                } else {
                    response.trim()
                }))
            }
            Ok(Err(e)) => {
                // Read error, but command may have been sent successfully
                warn!(error = %e, "Error reading socket response");
                Ok(PtzCommandResult::success("Command sent (no response)"))
            }
            Err(_) => {
                // Timeout reading response, but command was likely sent
                warn!("Timeout reading socket response");
                Ok(PtzCommandResult::success("Command sent (response timeout)"))
            }
        }
    }

    /// Build command-line arguments for zmcontrol.pl
    fn build_args(&self, command: &PtzCommand) -> Vec<String> {
        let mut args = vec![
            "--id".to_string(),
            self.config.monitor_id.to_string(),
            "--command".to_string(),
            command.zmcontrol_command().to_string(),
        ];

        // Add speed parameters based on command type
        match command {
            PtzCommand::MoveUp(params)
            | PtzCommand::MoveDown(params)
            | PtzCommand::MoveLeft(params)
            | PtzCommand::MoveRight(params)
            | PtzCommand::MoveUpLeft(params)
            | PtzCommand::MoveUpRight(params)
            | PtzCommand::MoveDownLeft(params)
            | PtzCommand::MoveDownRight(params) => {
                self.add_move_params(&mut args, params);
            }
            PtzCommand::ZoomIn(params) | PtzCommand::ZoomOut(params) => {
                self.add_zoom_params(&mut args, params);
            }
            PtzCommand::GotoPreset { preset_id } => {
                args.push("--preset".to_string());
                args.push(preset_id.to_string());
            }
            PtzCommand::SetPreset { preset_id, name } => {
                args.push("--preset".to_string());
                args.push(preset_id.to_string());
                if let Some(n) = name {
                    args.push("--name".to_string());
                    args.push(n.clone());
                }
            }
            PtzCommand::ClearPreset { preset_id } => {
                args.push("--preset".to_string());
                args.push(preset_id.to_string());
            }
            PtzCommand::MoveAbsolute(pos) => {
                self.add_absolute_params(&mut args, pos);
            }
            PtzCommand::MoveRelative(pos) => {
                if let Some(pan) = pos.pan_delta {
                    args.push("--panstep".to_string());
                    args.push(pan.to_string());
                }
                if let Some(tilt) = pos.tilt_delta {
                    args.push("--tiltstep".to_string());
                    args.push(tilt.to_string());
                }
                if let Some(zoom) = pos.zoom_delta {
                    args.push("--zoomstep".to_string());
                    args.push(zoom.to_string());
                }
            }
            _ => {}
        }

        // Add auto-stop if configured
        if let Some(timeout) = self.config.auto_stop_timeout {
            if timeout > 0.0 {
                args.push("--autostop".to_string());
            }
        }

        args
    }

    fn add_move_params(&self, args: &mut Vec<String>, params: &MoveParams) {
        if let Some(pan_speed) = params.pan_speed {
            args.push("--panspeed".to_string());
            args.push(self.scale_speed(pan_speed, true).to_string());
        }
        if let Some(tilt_speed) = params.tilt_speed {
            args.push("--tiltspeed".to_string());
            args.push(self.scale_speed(tilt_speed, false).to_string());
        }
    }

    fn add_zoom_params(&self, args: &mut Vec<String>, params: &ZoomParams) {
        if let Some(speed) = params.speed {
            args.push("--speed".to_string());
            args.push(self.scale_zoom_speed(speed).to_string());
        }
    }

    fn add_absolute_params(&self, args: &mut Vec<String>, pos: &AbsolutePosition) {
        if let Some(pan) = pos.pan {
            args.push("--panpos".to_string());
            args.push(pan.to_string());
        }
        if let Some(tilt) = pos.tilt {
            args.push("--tiltpos".to_string());
            args.push(tilt.to_string());
        }
        if let Some(zoom) = pos.zoom {
            args.push("--zoompos".to_string());
            args.push(zoom.to_string());
        }
    }

    /// Scale a percentage speed (0-100) to the camera's speed range
    fn scale_speed(&self, percent: u8, is_pan: bool) -> i32 {
        let (min, max) = if is_pan {
            (
                self.capabilities.pan_tilt.pan_speed.min.unwrap_or(0),
                self.capabilities.pan_tilt.pan_speed.max.unwrap_or(100),
            )
        } else {
            (
                self.capabilities.pan_tilt.tilt_speed.min.unwrap_or(0),
                self.capabilities.pan_tilt.tilt_speed.max.unwrap_or(100),
            )
        };

        let range = max - min;
        min + (range * percent as i32 / 100)
    }

    fn scale_zoom_speed(&self, percent: u8) -> i32 {
        let min = self.capabilities.zoom.speed.min.unwrap_or(0);
        let max = self.capabilities.zoom.speed.max.unwrap_or(100);
        let range = max - min;
        min + (range * percent as i32 / 100)
    }

    /// Execute zmcontrol.pl with the given arguments
    #[instrument(skip(self), fields(monitor_id = self.config.monitor_id))]
    async fn execute_zmcontrol(&self, args: Vec<String>) -> PtzResult<PtzCommandResult> {
        debug!(
            path = %self.zmcontrol_path,
            args = ?args,
            "Executing zmcontrol.pl"
        );

        let result = tokio::time::timeout(
            std::time::Duration::from_secs(COMMAND_TIMEOUT_SECS),
            Command::new(&self.zmcontrol_path)
                .args(&args)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .map_err(|e| PtzError::PerlBridgeError(format!("Failed to spawn process: {}", e)))?
                .wait_with_output(),
        )
        .await
        .map_err(|_| {
            PtzError::CommandTimeout(format!(
                "zmcontrol.pl timed out after {} seconds",
                COMMAND_TIMEOUT_SECS
            ))
        })?
        .map_err(|e| PtzError::PerlBridgeError(format!("Process error: {}", e)))?;

        let stdout = String::from_utf8_lossy(&result.stdout);
        let stderr = String::from_utf8_lossy(&result.stderr);

        if !result.status.success() {
            let exit_code = result.status.code().unwrap_or(-1);
            error!(
                exit_code = exit_code,
                stderr = %stderr,
                "zmcontrol.pl failed"
            );

            // Parse common error conditions
            let error_msg = if stderr.contains("Authentication") {
                return Err(PtzError::AuthenticationFailed(stderr.to_string()));
            } else if stderr.contains("not supported") {
                return Err(PtzError::CommandNotSupported(stderr.to_string()));
            } else if stderr.contains("Connection") || stderr.contains("timeout") {
                return Err(PtzError::CameraOffline(stderr.to_string()));
            } else {
                format!("Exit code {}: {}", exit_code, stderr)
            };

            return Err(PtzError::PerlBridgeError(error_msg));
        }

        if !stderr.is_empty() {
            warn!(stderr = %stderr, "zmcontrol.pl warning");
        }

        debug!(stdout = %stdout, "zmcontrol.pl succeeded");

        Ok(PtzCommandResult::success(if stdout.is_empty() {
            "Command executed successfully"
        } else {
            stdout.trim()
        }))
    }
}

#[async_trait]
impl PtzControl for PerlControlProxy {
    fn capabilities(&self) -> &PtzCapabilities {
        &self.capabilities
    }

    fn protocol_name(&self) -> &str {
        self.capabilities.protocol.as_deref().unwrap_or("unknown")
    }

    fn is_native(&self) -> bool {
        false
    }

    #[instrument(skip(self), fields(monitor_id = self.config.monitor_id, command = ?command))]
    async fn execute(&self, command: PtzCommand) -> PtzResult<PtzCommandResult> {
        // Try socket communication first (matches ZoneMinder PHP behavior)
        match self.execute_via_socket(&command).await {
            Ok(result) => {
                info!(
                    monitor_id = self.config.monitor_id,
                    method = "socket",
                    "PTZ command executed via socket"
                );
                return Ok(result);
            }
            Err(e) => {
                debug!(
                    error = %e,
                    "Socket execution failed, falling back to process"
                );
            }
        }

        // Fallback to direct process execution
        info!(
            monitor_id = self.config.monitor_id,
            method = "process",
            "Falling back to zmcontrol.pl process execution"
        );
        let args = self.build_args(&command);
        self.execute_zmcontrol(args).await
    }
}

/// Factory for creating Perl bridge control instances
pub struct PerlControlFactory {
    zmcontrol_path: Option<String>,
    socket_dir: Option<String>,
}

impl PerlControlFactory {
    pub fn new(zmcontrol_path: Option<String>, socket_dir: Option<String>) -> Self {
        Self {
            zmcontrol_path,
            socket_dir,
        }
    }
}

impl Default for PerlControlFactory {
    fn default() -> Self {
        Self::new(None, None)
    }
}

impl PtzControlFactory for PerlControlFactory {
    fn protocol_name(&self) -> &str {
        "perl"
    }

    fn is_native(&self) -> bool {
        false
    }

    fn create(
        &self,
        config: PtzConnectionConfig,
        capabilities: PtzCapabilities,
    ) -> Box<dyn PtzControl> {
        Box::new(PerlControlProxy::new(
            config,
            capabilities,
            self.zmcontrol_path.clone(),
            self.socket_dir.clone(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> PtzConnectionConfig {
        PtzConnectionConfig {
            monitor_id: 1,
            address: "192.168.1.100".to_string(),
            username: Some("admin".to_string()),
            password: Some("password".to_string()),
            protocol: "onvif".to_string(),
            auto_stop_timeout: Some(5.0),
        }
    }

    fn test_capabilities() -> PtzCapabilities {
        let mut caps = PtzCapabilities::default();
        caps.pan_tilt.can_move = true;
        caps.pan_tilt.can_move_con = true;
        caps.pan_tilt.pan_speed.has_speed = true;
        caps.pan_tilt.pan_speed.min = Some(0);
        caps.pan_tilt.pan_speed.max = Some(100);
        caps
    }

    #[test]
    fn test_build_args_move_up() {
        let proxy = PerlControlProxy::new(test_config(), test_capabilities(), None, None);

        let args = proxy.build_args(&PtzCommand::MoveUp(MoveParams {
            pan_speed: Some(50),
            tilt_speed: Some(50),
            ..Default::default()
        }));

        assert!(args.contains(&"--id".to_string()));
        assert!(args.contains(&"1".to_string()));
        assert!(args.contains(&"--command".to_string()));
        assert!(args.contains(&"moveConUp".to_string()));
        assert!(args.contains(&"--tiltspeed".to_string()));
        assert!(args.contains(&"--autostop".to_string()));
    }

    #[test]
    fn test_build_args_goto_preset() {
        let proxy = PerlControlProxy::new(test_config(), test_capabilities(), None, None);

        let args = proxy.build_args(&PtzCommand::GotoPreset { preset_id: 5 });

        assert!(args.contains(&"--command".to_string()));
        assert!(args.contains(&"presetGoto".to_string()));
        assert!(args.contains(&"--preset".to_string()));
        assert!(args.contains(&"5".to_string()));
    }

    #[test]
    fn test_scale_speed() {
        let proxy = PerlControlProxy::new(test_config(), test_capabilities(), None, None);

        assert_eq!(proxy.scale_speed(0, true), 0);
        assert_eq!(proxy.scale_speed(100, true), 100);
        assert_eq!(proxy.scale_speed(50, true), 50);
    }

    #[test]
    fn test_build_json_options_move_up() {
        let proxy = PerlControlProxy::new(test_config(), test_capabilities(), None, None);

        let json = proxy.build_json_options(&PtzCommand::MoveUp(MoveParams {
            pan_speed: Some(50),
            tilt_speed: Some(50),
            ..Default::default()
        }));

        assert_eq!(json["command"], "moveConUp");
        assert_eq!(json["panspeed"], 50);
        assert_eq!(json["tiltspeed"], 50);
        assert_eq!(json["autostop"], 1); // PHP uses 1 for flags without values
    }

    #[test]
    fn test_build_json_options_goto_preset() {
        let proxy = PerlControlProxy::new(test_config(), test_capabilities(), None, None);

        let json = proxy.build_json_options(&PtzCommand::GotoPreset { preset_id: 5 });

        assert_eq!(json["command"], "presetGoto");
        assert_eq!(json["preset"], 5);
    }

    #[test]
    fn test_socket_path() {
        let proxy = PerlControlProxy::new(test_config(), test_capabilities(), None, None);
        assert_eq!(proxy.socket_path, PathBuf::from("/run/zm/zmcontrol-1.sock"));

        let proxy_custom = PerlControlProxy::new(
            test_config(),
            test_capabilities(),
            None,
            Some("/tmp/zm".to_string()),
        );
        assert_eq!(
            proxy_custom.socket_path,
            PathBuf::from("/tmp/zm/zmcontrol-1.sock")
        );
    }
}
