//! PTZ (Pan-Tilt-Zoom) Control System
//!
//! This module provides a unified interface for controlling PTZ cameras through
//! various protocols. It supports both native Rust implementations and a Perl
//! bridge for legacy protocol support.

pub mod bridge;
pub mod capabilities;
pub mod error;
pub mod manager;
#[cfg(feature = "onvif-ptz")]
pub mod protocols;
pub mod registry;
pub mod traits;

// Re-export commonly used types
pub use capabilities::PtzCapabilities;
pub use error::PtzError;
pub use manager::PtzManager;
#[cfg(feature = "onvif-ptz")]
pub use protocols::onvif::{OnvifControl, OnvifControlFactory};
pub use registry::PtzRegistry;
pub use traits::{PtzCommand, PtzControl};
