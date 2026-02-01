//! PTZ capability definitions mapped from the Controls database table

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::entity::controls::Model as ControlModel;

/// Range limits for a PTZ axis (pan, tilt, zoom, etc.)
#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct AxisRange {
    pub min: Option<i32>,
    pub max: Option<i32>,
}

/// Step limits for incremental movements
#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct AxisStep {
    pub min: Option<i32>,
    pub max: Option<i32>,
}

/// Speed limits for an axis
#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct AxisSpeed {
    pub has_speed: bool,
    pub min: Option<i32>,
    pub max: Option<i32>,
}

/// Turbo speed configuration for pan/tilt
#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct TurboSpeed {
    pub has_turbo: bool,
    pub speed: Option<i32>,
}

/// Capability flags for an axis (zoom, focus, iris, gain, white balance)
#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct AxisCapabilities {
    /// Can perform this operation at all
    pub can: bool,
    /// Has auto mode
    pub can_auto: bool,
    /// Supports absolute positioning
    pub can_abs: bool,
    /// Supports relative movements
    pub can_rel: bool,
    /// Supports continuous movement
    pub can_con: bool,
    /// Range limits
    pub range: AxisRange,
    /// Step limits
    pub step: AxisStep,
    /// Speed limits
    pub speed: AxisSpeed,
}

/// Pan/Tilt specific capabilities
#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct PanTiltCapabilities {
    /// Can pan (horizontal movement)
    pub can_pan: bool,
    /// Can tilt (vertical movement)
    pub can_tilt: bool,
    /// Can move at all
    pub can_move: bool,
    /// Supports diagonal movement
    pub can_move_diag: bool,
    /// Supports map-based movement (click-to-move)
    pub can_move_map: bool,
    /// Supports absolute positioning
    pub can_move_abs: bool,
    /// Supports relative movements
    pub can_move_rel: bool,
    /// Supports continuous movement
    pub can_move_con: bool,
    /// Pan range limits
    pub pan_range: AxisRange,
    /// Pan step limits
    pub pan_step: AxisStep,
    /// Pan speed limits
    pub pan_speed: AxisSpeed,
    /// Turbo pan speed
    pub pan_turbo: TurboSpeed,
    /// Tilt range limits
    pub tilt_range: AxisRange,
    /// Tilt step limits
    pub tilt_step: AxisStep,
    /// Tilt speed limits
    pub tilt_speed: AxisSpeed,
    /// Turbo tilt speed
    pub tilt_turbo: TurboSpeed,
}

/// Preset capabilities
#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct PresetCapabilities {
    /// Has preset support
    pub has_presets: bool,
    /// Number of presets supported
    pub num_presets: u8,
    /// Has a home preset
    pub has_home_preset: bool,
    /// Can set/save presets
    pub can_set_presets: bool,
}

/// Power control capabilities
#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct PowerCapabilities {
    pub can_wake: bool,
    pub can_sleep: bool,
    pub can_reset: bool,
    pub can_reboot: bool,
}

/// Auto-scan capabilities
#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct ScanCapabilities {
    pub can_auto_scan: bool,
    pub num_scan_paths: u8,
}

/// Complete PTZ capabilities for a camera
#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct PtzCapabilities {
    /// Control ID from database
    pub control_id: u32,
    /// Control name
    pub name: String,
    /// Protocol name (e.g., "onvif", "Dahua", etc.)
    pub protocol: Option<String>,
    /// Power control
    pub power: PowerCapabilities,
    /// Pan/Tilt capabilities
    pub pan_tilt: PanTiltCapabilities,
    /// Zoom capabilities
    pub zoom: AxisCapabilities,
    /// Focus capabilities
    pub focus: AxisCapabilities,
    /// Iris capabilities
    pub iris: AxisCapabilities,
    /// Gain capabilities
    pub gain: AxisCapabilities,
    /// White balance capabilities
    pub white_balance: AxisCapabilities,
    /// Preset capabilities
    pub presets: PresetCapabilities,
    /// Scan capabilities
    pub scan: ScanCapabilities,
}

impl PtzCapabilities {
    /// Check if camera supports any PTZ operations
    pub fn has_any_ptz(&self) -> bool {
        self.pan_tilt.can_move
            || self.pan_tilt.can_pan
            || self.pan_tilt.can_tilt
            || self.zoom.can
            || self.focus.can
            || self.presets.has_presets
    }

    /// Check if camera supports continuous movement
    pub fn supports_continuous_move(&self) -> bool {
        self.pan_tilt.can_move_con
    }

    /// Check if camera supports absolute positioning
    pub fn supports_absolute_move(&self) -> bool {
        self.pan_tilt.can_move_abs
    }
}

/// Convert from database Controls model to PtzCapabilities
impl From<&ControlModel> for PtzCapabilities {
    fn from(m: &ControlModel) -> Self {
        Self {
            control_id: m.id,
            name: m.name.clone(),
            protocol: m.protocol.clone(),
            power: PowerCapabilities {
                can_wake: m.can_wake != 0,
                can_sleep: m.can_sleep != 0,
                can_reset: m.can_reset != 0,
                can_reboot: m.can_reboot != 0,
            },
            pan_tilt: PanTiltCapabilities {
                can_pan: m.can_pan != 0,
                can_tilt: m.can_tilt != 0,
                can_move: m.can_move != 0,
                can_move_diag: m.can_move_diag != 0,
                can_move_map: m.can_move_map != 0,
                can_move_abs: m.can_move_abs != 0,
                can_move_rel: m.can_move_rel != 0,
                can_move_con: m.can_move_con != 0,
                pan_range: AxisRange {
                    min: m.min_pan_range,
                    max: m.max_pan_range,
                },
                pan_step: AxisStep {
                    min: m.min_pan_step,
                    max: m.max_pan_step,
                },
                pan_speed: AxisSpeed {
                    has_speed: m.has_pan_speed != 0,
                    min: m.min_pan_speed,
                    max: m.max_pan_speed,
                },
                pan_turbo: TurboSpeed {
                    has_turbo: m.has_turbo_pan != 0,
                    speed: m.turbo_pan_speed,
                },
                tilt_range: AxisRange {
                    min: m.min_tilt_range,
                    max: m.max_tilt_range,
                },
                tilt_step: AxisStep {
                    min: m.min_tilt_step,
                    max: m.max_tilt_step,
                },
                tilt_speed: AxisSpeed {
                    has_speed: m.has_tilt_speed != 0,
                    min: m.min_tilt_speed,
                    max: m.max_tilt_speed,
                },
                tilt_turbo: TurboSpeed {
                    has_turbo: m.has_turbo_tilt != 0,
                    speed: m.turbo_tilt_speed,
                },
            },
            zoom: AxisCapabilities {
                can: m.can_zoom != 0,
                can_auto: m.can_auto_zoom != 0,
                can_abs: m.can_zoom_abs != 0,
                can_rel: m.can_zoom_rel != 0,
                can_con: m.can_zoom_con != 0,
                range: AxisRange {
                    min: m.min_zoom_range.map(|v| v as i32),
                    max: m.max_zoom_range.map(|v| v as i32),
                },
                step: AxisStep {
                    min: m.min_zoom_step.map(|v| v as i32),
                    max: m.max_zoom_step.map(|v| v as i32),
                },
                speed: AxisSpeed {
                    has_speed: m.has_zoom_speed != 0,
                    min: m.min_zoom_speed.map(|v| v as i32),
                    max: m.max_zoom_speed.map(|v| v as i32),
                },
            },
            focus: AxisCapabilities {
                can: m.can_focus != 0,
                can_auto: m.can_auto_focus != 0,
                can_abs: m.can_focus_abs != 0,
                can_rel: m.can_focus_rel != 0,
                can_con: m.can_focus_con != 0,
                range: AxisRange {
                    min: m.min_focus_range.map(|v| v as i32),
                    max: m.max_focus_range.map(|v| v as i32),
                },
                step: AxisStep {
                    min: m.min_focus_step.map(|v| v as i32),
                    max: m.max_focus_step.map(|v| v as i32),
                },
                speed: AxisSpeed {
                    has_speed: m.has_focus_speed != 0,
                    min: m.min_focus_speed.map(|v| v as i32),
                    max: m.max_focus_speed.map(|v| v as i32),
                },
            },
            iris: AxisCapabilities {
                can: m.can_iris != 0,
                can_auto: m.can_auto_iris != 0,
                can_abs: m.can_iris_abs != 0,
                can_rel: m.can_iris_rel != 0,
                can_con: m.can_iris_con != 0,
                range: AxisRange {
                    min: m.min_iris_range.map(|v| v as i32),
                    max: m.max_iris_range.map(|v| v as i32),
                },
                step: AxisStep {
                    min: m.min_iris_step.map(|v| v as i32),
                    max: m.max_iris_step.map(|v| v as i32),
                },
                speed: AxisSpeed {
                    has_speed: m.has_iris_speed != 0,
                    min: m.min_iris_speed.map(|v| v as i32),
                    max: m.max_iris_speed.map(|v| v as i32),
                },
            },
            gain: AxisCapabilities {
                can: m.can_gain != 0,
                can_auto: m.can_auto_gain != 0,
                can_abs: m.can_gain_abs != 0,
                can_rel: m.can_gain_rel != 0,
                can_con: m.can_gain_con != 0,
                range: AxisRange {
                    min: m.min_gain_range.map(|v| v as i32),
                    max: m.max_gain_range.map(|v| v as i32),
                },
                step: AxisStep {
                    min: m.min_gain_step.map(|v| v as i32),
                    max: m.max_gain_step.map(|v| v as i32),
                },
                speed: AxisSpeed {
                    has_speed: m.has_gain_speed != 0,
                    min: m.min_gain_speed.map(|v| v as i32),
                    max: m.max_gain_speed.map(|v| v as i32),
                },
            },
            white_balance: AxisCapabilities {
                can: m.can_white != 0,
                can_auto: m.can_auto_white != 0,
                can_abs: m.can_white_abs != 0,
                can_rel: m.can_white_rel != 0,
                can_con: m.can_white_con != 0,
                range: AxisRange {
                    min: m.min_white_range.map(|v| v as i32),
                    max: m.max_white_range.map(|v| v as i32),
                },
                step: AxisStep {
                    min: m.min_white_step.map(|v| v as i32),
                    max: m.max_white_step.map(|v| v as i32),
                },
                speed: AxisSpeed {
                    has_speed: m.has_white_speed != 0,
                    min: m.min_white_speed.map(|v| v as i32),
                    max: m.max_white_speed.map(|v| v as i32),
                },
            },
            presets: PresetCapabilities {
                has_presets: m.has_presets != 0,
                num_presets: m.num_presets,
                has_home_preset: m.has_home_preset != 0,
                can_set_presets: m.can_set_presets != 0,
            },
            scan: ScanCapabilities {
                can_auto_scan: m.can_auto_scan != 0,
                num_scan_paths: m.num_scan_paths,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_capabilities_has_no_ptz() {
        let caps = PtzCapabilities::default();
        assert!(!caps.has_any_ptz());
    }

    #[test]
    fn test_capabilities_with_pan_has_ptz() {
        let mut caps = PtzCapabilities::default();
        caps.pan_tilt.can_pan = true;
        assert!(caps.has_any_ptz());
    }

    #[test]
    fn test_capabilities_with_zoom_has_ptz() {
        let mut caps = PtzCapabilities::default();
        caps.zoom.can = true;
        assert!(caps.has_any_ptz());
    }
}
