//! ONVIF client subsystem.
//!
//! A reusable, client-only ONVIF library: SOAP-over-HTTP transport,
//! WS-Security UsernameToken authentication, and the five WSDL service
//! clients (Device, Media, PTZ, Events, WS-Discovery). See
//! `docs/ONVIF_TASKS.md` for the architecture and phased plan.
//!
//! This module is the Phase 1 foundation: `error`, `types`, `transport`,
//! and `security` are implemented; the service submodules are stubs filled
//! in by later phases.

pub mod error;
pub mod security;
pub mod transport;
pub mod types;

pub mod device;
pub mod discovery;
pub mod events;
pub mod media;
pub mod ptz;

pub use error::{OnvifError, OnvifResult};
pub use security::{password_digest, wsse_username_token};
pub use transport::OnvifTransport;
pub use types::{Credentials, ServiceUrls};
