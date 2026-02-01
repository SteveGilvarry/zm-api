//! PTZ response DTOs

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::ptz::capabilities::PtzCapabilities;
use crate::ptz::registry::ProtocolInfo;

/// Response for PTZ command execution
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PtzCommandResponse {
    /// Whether the command succeeded
    pub success: bool,

    /// Human-readable message
    pub message: String,
}

impl PtzCommandResponse {
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
        }
    }

    pub fn failure(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
        }
    }
}

/// Response for PTZ status query
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PtzStatusResponse {
    /// Monitor ID
    pub monitor_id: u32,

    /// Whether PTZ control is available
    pub available: bool,

    /// Protocol name
    pub protocol: Option<String>,

    /// Whether using native Rust implementation
    pub is_native: bool,

    /// Full capabilities
    pub capabilities: PtzCapabilities,

    /// Current position (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<PtzPositionResponse>,
}

/// Current PTZ position
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PtzPositionResponse {
    /// Pan position
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pan: Option<f64>,

    /// Tilt position
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tilt: Option<f64>,

    /// Zoom position
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zoom: Option<f64>,
}

/// Response for protocol list
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PtzProtocolListResponse {
    /// List of available protocols
    pub protocols: Vec<PtzProtocolInfo>,

    /// List of native (Rust) protocol names
    pub native_protocols: Vec<String>,

    /// Whether Perl fallback is enabled
    pub perl_fallback_enabled: bool,
}

/// Information about a single protocol
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PtzProtocolInfo {
    /// Protocol name
    pub name: String,

    /// Whether this is a native Rust implementation
    pub is_native: bool,

    /// Optional description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl From<ProtocolInfo> for PtzProtocolInfo {
    fn from(info: ProtocolInfo) -> Self {
        Self {
            name: info.name,
            is_native: info.is_native,
            description: info.description,
        }
    }
}

/// Response for capabilities query
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PtzCapabilitiesResponse {
    /// Monitor ID
    pub monitor_id: u32,

    /// Control ID
    pub control_id: u32,

    /// Control name
    pub name: String,

    /// Protocol name
    pub protocol: Option<String>,

    /// Full capabilities
    #[serde(flatten)]
    pub capabilities: PtzCapabilities,
}
