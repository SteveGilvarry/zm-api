//! PTZ-specific error types

use std::fmt;

/// PTZ operation errors
#[derive(Debug)]
pub enum PtzError {
    /// Camera is offline or unreachable
    CameraOffline(String),

    /// Authentication failed (invalid credentials)
    AuthenticationFailed(String),

    /// Command not supported by this camera/protocol
    CommandNotSupported(String),

    /// Invalid parameter value
    InvalidParameter(String),

    /// Command timed out
    CommandTimeout(String),

    /// Protocol-level error (SOAP fault, HTTP error, etc.)
    ProtocolError(String),

    /// Perl bridge execution error
    PerlBridgeError(String),

    /// Monitor not found in database
    MonitorNotFound(u32),

    /// Monitor has no PTZ control configured
    NoControlConfigured(u32),

    /// Control configuration not found
    ControlNotFound(u32),

    /// Internal error
    InternalError(String),
}

impl fmt::Display for PtzError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CameraOffline(msg) => write!(f, "Camera offline: {}", msg),
            Self::AuthenticationFailed(msg) => write!(f, "Authentication failed: {}", msg),
            Self::CommandNotSupported(cmd) => write!(f, "Command not supported: {}", cmd),
            Self::InvalidParameter(msg) => write!(f, "Invalid parameter: {}", msg),
            Self::CommandTimeout(msg) => write!(f, "Command timeout: {}", msg),
            Self::ProtocolError(msg) => write!(f, "Protocol error: {}", msg),
            Self::PerlBridgeError(msg) => write!(f, "Perl bridge error: {}", msg),
            Self::MonitorNotFound(id) => write!(f, "Monitor {} not found", id),
            Self::NoControlConfigured(id) => {
                write!(f, "Monitor {} has no PTZ control configured", id)
            }
            Self::ControlNotFound(id) => write!(f, "Control configuration {} not found", id),
            Self::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for PtzError {}

/// Result type for PTZ operations
pub type PtzResult<T> = Result<T, PtzError>;
