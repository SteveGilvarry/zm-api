//! Live streaming coordination module
//!
//! Coordinates the flow of video data from per-monitor stream sockets to
//! various output protocols (HLS, WebRTC). This module bridges the
//! SourceRouter (which reads zmc's stream sockets) to the protocol-specific
//! output handlers.

pub mod audio;
pub mod coordinator;
pub mod hls;
pub mod webrtc;

pub use coordinator::*;
