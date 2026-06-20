//! Shared types for the ONVIF client subsystem.
//!
//! Service-specific request/response types live in their respective service
//! modules (`device`, `media`, `ptz`, `events`, `discovery`); only types
//! shared across services belong here.

/// WS-Security UsernameToken credentials for an ONVIF device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Credentials {
    /// ONVIF username.
    pub username: String,
    /// ONVIF password (used to derive the digest; never sent in clear).
    pub password: String,
}

impl Credentials {
    /// Construct credentials from any string-like values.
    pub fn new(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            username: username.into(),
            password: password.into(),
        }
    }
}

/// Resolved endpoint URLs for a device's ONVIF services.
///
/// The Device service URL is the entry point; the rest are discovered via
/// `GetCapabilities` and may be absent if the device does not advertise the
/// service.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ServiceUrls {
    /// Device service endpoint (the well-known entry point, e.g.
    /// `http://host/onvif/device_service`).
    pub device: String,
    /// Media service endpoint, if advertised.
    pub media: Option<String>,
    /// PTZ service endpoint, if advertised.
    pub ptz: Option<String>,
    /// Events service endpoint, if advertised.
    pub events: Option<String>,
}

impl ServiceUrls {
    /// Construct with only the device entry-point known; other services are
    /// filled in after `GetCapabilities`.
    pub fn from_device(device: impl Into<String>) -> Self {
        Self {
            device: device.into(),
            ..Default::default()
        }
    }
}
