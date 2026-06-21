//! Native PTZ protocol implementations.
//!
//! Each submodule implements [`crate::ptz::traits::PtzControl`] for a specific
//! camera control protocol. Currently: ONVIF. These are registered as native
//! factories in the [`crate::ptz::registry::PtzRegistry`] at server startup;
//! protocols without a native implementation fall back to the Perl bridge.

pub mod onvif;
