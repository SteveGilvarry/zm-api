// src/dto/request/discovery.rs
//! Request DTOs for the ONVIF camera discovery endpoints.
//!
//! `ProbeRequest` drives a WS-Discovery multicast probe (`POST
//! /api/v3/discovery/probe`); `InspectRequest` drives a directed inspection of a
//! single device's ONVIF service endpoint (`POST /api/v3/discovery/inspect`).
//!
//! The `xaddr` field flows into outbound HTTP requests made by the API process,
//! so it is hardened with the same SSRF guard used for monitor `onvif_url`
//! (`is_safe_onvif_url`): scheme is restricted to http/https/rtsp/rtsps, and
//! control/CR/LF characters are rejected.
use garde::Validate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::dto::request::monitor::is_safe_onvif_url;

/// Request to run a WS-Discovery multicast probe for ONVIF devices on the local
/// network.
#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct ProbeRequest {
    /// How long, in milliseconds, to wait for `ProbeMatch` responses before
    /// returning the collected candidates. Bounded to keep the request from
    /// holding a connection open indefinitely.
    #[garde(range(min = 100, max = 30_000))]
    pub timeout_ms: u64,
}

/// Request to inspect a single ONVIF device by its service-endpoint URL,
/// retrieving device information and media profiles/stream URIs.
#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct InspectRequest {
    /// ONVIF device-service endpoint URL (the `XAddr` surfaced by a probe, e.g.
    /// `http://192.168.1.50/onvif/device_service`). Restricted to camera URL
    /// schemes to prevent SSRF.
    // ONVIF XAddr values are full URLs; mirror the monitor `onvif_url` cap.
    #[garde(length(max = 255))]
    #[garde(custom(is_safe_onvif_url))]
    pub xaddr: String,

    /// ONVIF username for WS-Security authentication. Empty string means the
    /// device is queried unauthenticated.
    #[garde(length(min = 0))]
    pub username: String,

    /// ONVIF password for WS-Security authentication.
    #[garde(length(min = 0))]
    pub password: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn probe_request_accepts_in_range_timeout() {
        let req = ProbeRequest { timeout_ms: 2000 };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn probe_request_rejects_zero_and_oversized_timeout() {
        assert!(ProbeRequest { timeout_ms: 0 }.validate().is_err());
        assert!(ProbeRequest { timeout_ms: 99 }.validate().is_err());
        assert!(ProbeRequest { timeout_ms: 30_001 }.validate().is_err());
    }

    #[test]
    fn inspect_request_accepts_camera_xaddr() {
        let req = InspectRequest {
            xaddr: "http://192.168.1.50/onvif/device_service".to_string(),
            username: "admin".to_string(),
            password: "secret".to_string(),
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn inspect_request_accepts_empty_credentials() {
        let req = InspectRequest {
            xaddr: "http://192.168.1.50/onvif/device_service".to_string(),
            username: String::new(),
            password: String::new(),
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn inspect_request_rejects_ssrf_schemes() {
        let req = InspectRequest {
            xaddr: "file:///etc/passwd".to_string(),
            username: String::new(),
            password: String::new(),
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn inspect_request_rejects_control_chars_in_xaddr() {
        let req = InspectRequest {
            xaddr: "http://host/onvif\r\nInjected: 1".to_string(),
            username: String::new(),
            password: String::new(),
        };
        assert!(req.validate().is_err());
    }
}
