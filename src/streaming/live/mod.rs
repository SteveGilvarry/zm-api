//! Live streaming coordination module
//!
//! Coordinates the flow of video data from FIFO sources to various output
//! protocols (HLS, WebRTC, MSE). This module bridges the SourceRouter
//! (which reads from ZoneMinder FIFOs) to the protocol-specific output handlers.

pub mod coordinator;
pub mod hls;
pub mod mse;
pub mod webrtc;

pub use coordinator::*;
