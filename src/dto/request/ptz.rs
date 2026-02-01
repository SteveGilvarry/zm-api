//! PTZ request DTOs

use garde::Validate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::ptz::traits::{FocusParams, MoveParams, ZoomParams};

/// Request for continuous movement
#[derive(Debug, Clone, Default, Deserialize, Serialize, ToSchema, Validate)]
pub struct PtzMoveRequest {
    /// Pan speed (0-100 percent)
    #[garde(range(max = 100))]
    #[serde(default)]
    pub pan_speed: Option<u8>,

    /// Tilt speed (0-100 percent)
    #[garde(range(max = 100))]
    #[serde(default)]
    pub tilt_speed: Option<u8>,

    /// Duration in milliseconds (optional, for timed movements)
    #[garde(skip)]
    #[serde(default)]
    pub duration_ms: Option<u32>,
}

impl From<PtzMoveRequest> for MoveParams {
    fn from(req: PtzMoveRequest) -> Self {
        Self {
            pan_speed: req.pan_speed,
            tilt_speed: req.tilt_speed,
            duration_ms: req.duration_ms,
            auto_stop: true,
        }
    }
}

/// Request for zoom operations
#[derive(Debug, Clone, Default, Deserialize, Serialize, ToSchema, Validate)]
pub struct PtzZoomRequest {
    /// Zoom speed (0-100 percent)
    #[garde(range(max = 100))]
    #[serde(default)]
    pub speed: Option<u8>,

    /// Duration in milliseconds
    #[garde(skip)]
    #[serde(default)]
    pub duration_ms: Option<u32>,
}

impl From<PtzZoomRequest> for ZoomParams {
    fn from(req: PtzZoomRequest) -> Self {
        Self {
            speed: req.speed,
            duration_ms: req.duration_ms,
        }
    }
}

/// Request for focus operations
#[derive(Debug, Clone, Default, Deserialize, Serialize, ToSchema, Validate)]
pub struct PtzFocusRequest {
    /// Focus speed (0-100 percent)
    #[garde(range(max = 100))]
    #[serde(default)]
    pub speed: Option<u8>,

    /// Duration in milliseconds
    #[garde(skip)]
    #[serde(default)]
    pub duration_ms: Option<u32>,
}

impl From<PtzFocusRequest> for FocusParams {
    fn from(req: PtzFocusRequest) -> Self {
        Self {
            speed: req.speed,
            duration_ms: req.duration_ms,
        }
    }
}

/// Request for preset operations
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, Validate)]
pub struct PtzPresetRequest {
    /// Optional name for the preset (when setting)
    #[garde(length(max = 64))]
    #[serde(default)]
    pub name: Option<String>,
}

/// Request for absolute positioning
#[derive(Debug, Clone, Default, Deserialize, Serialize, ToSchema, Validate)]
pub struct PtzAbsoluteRequest {
    /// Pan position
    #[garde(skip)]
    #[serde(default)]
    pub pan: Option<f64>,

    /// Tilt position
    #[garde(skip)]
    #[serde(default)]
    pub tilt: Option<f64>,

    /// Zoom position
    #[garde(skip)]
    #[serde(default)]
    pub zoom: Option<f64>,
}

/// Request for relative movement
#[derive(Debug, Clone, Default, Deserialize, Serialize, ToSchema, Validate)]
pub struct PtzRelativeRequest {
    /// Pan delta
    #[garde(skip)]
    #[serde(default)]
    pub pan_delta: Option<f64>,

    /// Tilt delta
    #[garde(skip)]
    #[serde(default)]
    pub tilt_delta: Option<f64>,

    /// Zoom delta
    #[garde(skip)]
    #[serde(default)]
    pub zoom_delta: Option<f64>,
}

/// Generic PTZ command request
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, Validate)]
pub struct PtzGenericCommandRequest {
    /// Command name (e.g., "moveConUp", "zoomIn")
    #[garde(length(min = 1, max = 64))]
    pub command: String,

    /// Optional parameters as key-value pairs
    #[garde(skip)]
    #[serde(default)]
    pub params: Option<serde_json::Value>,
}
