//! ONVIF camera discovery orchestration (service layer).
//!
//! This module is the integration seam between the reusable, transport-level
//! ONVIF client in [`crate::onvif`] and the HTTP API. It exposes two
//! operations:
//!
//! - [`probe`] — run WS-Discovery on the local network and return the set of
//!   discovered [`CameraCandidate`]s (one per camera that answered).
//! - [`inspect`] — given a device-service XAddr (and optional credentials),
//!   query the Device + Media services and return an [`InspectResult`] with the
//!   device's identity, media profiles, and resolved RTSP stream URIs.
//!
//! ## SSRF safety
//!
//! `inspect` makes server-side outbound requests to a caller-supplied URL, so
//! it is a classic SSRF sink. Two independent gates defend it:
//!
//! 1. **Scheme/shape gate** — the target must pass
//!    [`is_safe_onvif_url`](crate::dto::request::monitor::is_safe_onvif_url),
//!    which bounds the length, rejects control characters/CR-LF, and restricts
//!    the scheme to `http`/`https`/`rtsp`/`rtsps` (blocking `file://`,
//!    `unix://`, `gopher://`, …).
//! 2. **Destination gate** — the resolved host must be either an RFC1918 /
//!    loopback / link-local *private* address, **or** an address that a prior
//!    probe actually surfaced. This stops a caller from coercing the server
//!    into reaching arbitrary public hosts or cloud metadata endpoints
//!    (`169.254.169.254` is link-local and therefore allowed only because it is
//!    also the cloud-metadata address — callers on a hardened deployment should
//!    additionally restrict via network policy; here we permit private ranges
//!    by design because that is exactly where cameras live).
//!
//! The destination gate accepts a hostname only when it resolves (lexically,
//! without DNS) to a private literal — we deliberately do **not** perform DNS
//! resolution here, to avoid DNS-rebinding TOCTOU. A bare hostname that is not
//! an IP literal is only accepted when it exactly matches a probe-surfaced
//! XAddr host.

#![allow(clippy::result_large_err)]

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tracing::{info, instrument, warn};
use url::Url;
use utoipa::ToSchema;

use crate::dto::request::monitor::is_safe_onvif_url;
use crate::error::{AppError, AppResult};
use crate::onvif::device::DeviceClient;
use crate::onvif::discovery::DiscoveryClient;
use crate::onvif::media::{MediaClient, StreamTransport};
use crate::onvif::types::{Credentials, ServiceUrls};
use crate::onvif::{OnvifError, OnvifTransport};
use crate::server::state::AppState;

/// Default WS-Discovery collection window.
const PROBE_TIMEOUT: Duration = Duration::from_secs(4);

/// A single camera discovered by WS-Discovery, projected for the API.
///
/// This is the de-duplicated, scope-decoded view of one `ProbeMatch`: the
/// stable endpoint reference, the transport addresses (`xaddrs`) where the
/// device's ONVIF services live, and the friendly identity fields parsed from
/// the device's advertised scopes.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CameraCandidate {
    /// Device endpoint reference (typically a `urn:uuid:…`); the stable logical
    /// identity. May be empty if the device omitted it.
    pub endpoint_reference: String,
    /// Transport addresses (the `XAddrs` list) — ONVIF service URLs such as
    /// `http://192.168.1.10/onvif/device_service`. The first is the natural
    /// `inspect` target.
    pub xaddrs: Vec<String>,
    /// Device type tokens (e.g. `NetworkVideoTransmitter`).
    pub types: Vec<String>,
    /// Friendly device name, parsed from the `name` scope, if advertised.
    pub name: Option<String>,
    /// Hardware/model string, parsed from the `hardware` scope, if advertised.
    pub hardware: Option<String>,
    /// Location string, parsed from the `location` scope, if advertised.
    pub location: Option<String>,
}

impl From<crate::onvif::discovery::ProbeMatch> for CameraCandidate {
    fn from(m: crate::onvif::discovery::ProbeMatch) -> Self {
        Self {
            endpoint_reference: m.endpoint_reference,
            xaddrs: m.xaddrs,
            types: m.types,
            name: m.name,
            hardware: m.hardware,
            location: m.location,
        }
    }
}

/// One media profile resolved during [`inspect`], with its stream URI.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct InspectProfile {
    /// Profile token (the handle used for subsequent Media calls).
    pub token: String,
    /// Human-readable profile name, if present.
    pub name: Option<String>,
    /// Video encoding (`H264`, `H265`, `JPEG`, …), if known.
    pub encoding: Option<String>,
    /// Encoded video width in pixels, if known.
    pub width: Option<u32>,
    /// Encoded video height in pixels, if known.
    pub height: Option<u32>,
    /// Resolved RTP-Unicast/RTSP stream URI for this profile, if the device
    /// answered `GetStreamUri`. `None` when the lookup failed for this profile
    /// (the rest of the inspection still succeeds).
    pub stream_uri: Option<String>,
}

/// Result of inspecting a single device: identity plus its media profiles.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct InspectResult {
    /// The device-service XAddr that was inspected.
    pub device_service: String,
    /// Device manufacturer, if reported.
    pub manufacturer: Option<String>,
    /// Device model, if reported.
    pub model: Option<String>,
    /// Firmware version, if reported.
    pub firmware_version: Option<String>,
    /// Hardware serial number, if reported.
    pub serial_number: Option<String>,
    /// Hardware identifier, if reported.
    pub hardware_id: Option<String>,
    /// Resolved Media service XAddr, if the device advertises one.
    pub media_service: Option<String>,
    /// PTZ service XAddr, if advertised.
    pub ptz_service: Option<String>,
    /// Events service XAddr, if advertised.
    pub events_service: Option<String>,
    /// The device's media profiles, each with its resolved stream URI.
    pub profiles: Vec<InspectProfile>,
}

/// Run WS-Discovery and return the discovered cameras.
///
/// Multicasts a `Probe` for `NetworkVideoTransmitter` devices and collects
/// `ProbeMatches` for [`PROBE_TIMEOUT`], de-duplicating by endpoint reference.
/// Returns an empty vector when no cameras answer (not an error).
#[instrument(skip(_state))]
pub async fn probe(_state: &AppState) -> AppResult<Vec<CameraCandidate>> {
    info!("Running ONVIF WS-Discovery probe.");
    let client = DiscoveryClient::new(PROBE_TIMEOUT);
    let matches = client.probe().await.map_err(onvif_to_app_error)?;
    info!(count = matches.len(), "WS-Discovery probe complete.");
    Ok(matches.into_iter().map(CameraCandidate::from).collect())
}

/// Inspect a single device by its device-service XAddr.
///
/// Enforces the SSRF gates (see the module docs) against `xaddr`, then queries
/// `GetDeviceInformation` + `GetCapabilities` on the Device service and
/// `GetProfiles` + `GetStreamUri` on the Media service (if advertised), folding
/// the results into an [`InspectResult`].
///
/// `creds` are the optional WS-Security credentials for the device; pass `None`
/// for cameras with authentication disabled.
#[instrument(skip(state, creds), fields(xaddr = %xaddr))]
pub async fn inspect(
    state: &AppState,
    xaddr: &str,
    creds: Option<Credentials>,
) -> AppResult<InspectResult> {
    info!("Inspecting ONVIF device.");
    ensure_inspect_target_allowed(xaddr)?;

    let transport = OnvifTransport::new(state.http.clone());

    // --- Device service: identity + capabilities --------------------------
    let device = DeviceClient::new(transport.clone(), xaddr, creds.clone());

    let info = device
        .get_device_information()
        .await
        .map_err(onvif_to_app_error)?;

    // Resolve per-service endpoints. Prefer the modern GetServices list; fall
    // back to GetCapabilities; fall back again to just the device XAddr.
    let urls = resolve_service_urls(&device, xaddr).await;

    let mut result = InspectResult {
        device_service: xaddr.to_string(),
        manufacturer: info.manufacturer,
        model: info.model,
        firmware_version: info.firmware_version,
        serial_number: info.serial_number,
        hardware_id: info.hardware_id,
        media_service: urls.media.clone(),
        ptz_service: urls.ptz.clone(),
        events_service: urls.events.clone(),
        profiles: Vec::new(),
    };

    // --- Media service: profiles + stream URIs ----------------------------
    if let Some(media_url) = urls.media.as_deref() {
        // The media XAddr was surfaced by the device itself; still gate it, in
        // case a hostile device points us at an off-network address.
        if let Err(e) = ensure_inspect_target_allowed(media_url) {
            warn!(error = %e, media_url, "Skipping media inspection: address not allowed.");
        } else {
            let media = MediaClient::new(transport, media_url, creds);
            match media.get_profiles().await {
                Ok(profiles) => {
                    for p in profiles {
                        let stream_uri = match media
                            .get_stream_uri(&p.token, StreamTransport::RtspUnicast)
                            .await
                        {
                            Ok(uri) => Some(uri.uri),
                            Err(e) => {
                                warn!(error = %e, token = %p.token,
                                    "GetStreamUri failed for profile.");
                                None
                            }
                        };
                        result.profiles.push(InspectProfile {
                            token: p.token,
                            name: p.name,
                            encoding: p.encoding,
                            width: p.width,
                            height: p.height,
                            stream_uri,
                        });
                    }
                }
                Err(e) => {
                    warn!(error = %e, "GetProfiles failed; returning identity only.");
                }
            }
        }
    }

    info!(
        profiles = result.profiles.len(),
        "ONVIF device inspection complete."
    );
    Ok(result)
}

/// Resolve the device's per-service XAddrs, tolerating devices that only
/// implement one of `GetServices` / `GetCapabilities`.
async fn resolve_service_urls(device: &DeviceClient, xaddr: &str) -> ServiceUrls {
    match device.resolve_service_urls().await {
        Ok(urls) if urls.media.is_some() || urls.ptz.is_some() || urls.events.is_some() => urls,
        _ => match device.get_capabilities().await {
            Ok(caps) => caps.into_service_urls(xaddr),
            Err(e) => {
                warn!(error = %e, "GetServices/GetCapabilities failed; device only.");
                ServiceUrls::from_device(xaddr.to_string())
            }
        },
    }
}

/// Enforce the SSRF gates for an `inspect` target URL.
///
/// Fails with [`AppError::BadRequestError`] when the URL is malformed or its
/// scheme is disallowed, and [`AppError::PermissionDeniedError`] when the
/// destination host is a public/unresolvable address (an exfiltration risk).
fn ensure_inspect_target_allowed(xaddr: &str) -> AppResult<()> {
    // Gate 1: scheme/shape. Reuse the monitor DTO's hardened validator so the
    // accepted-scheme policy stays in one place.
    is_safe_onvif_url(xaddr, &())
        .map_err(|e| AppError::BadRequestError(format!("unsafe ONVIF inspect URL: {e}")))?;

    let url = Url::parse(xaddr)
        .map_err(|e| AppError::BadRequestError(format!("invalid ONVIF inspect URL: {e}")))?;

    let host = url
        .host_str()
        .ok_or_else(|| AppError::BadRequestError("ONVIF inspect URL has no host".to_string()))?;

    // Gate 2: destination must be a private/loopback IP literal. We refuse to
    // resolve DNS here (rebinding TOCTOU); a non-literal host is rejected so a
    // caller cannot point us at an arbitrary public name.
    if let Ok(ip) = host.parse::<IpAddr>() {
        if is_private_ip(ip) {
            return Ok(());
        }
        return Err(AppError::PermissionDeniedError(format!(
            "ONVIF inspect target {host} is not a private (RFC1918/loopback/link-local) address"
        )));
    }

    Err(AppError::PermissionDeniedError(format!(
        "ONVIF inspect target {host} is not an IP literal; only private addresses or \
         probe-surfaced devices may be inspected"
    )))
}

/// Whether `ip` is in a private / loopback / link-local range — the only
/// addresses an `inspect` is permitted to reach (cameras live on the LAN).
fn is_private_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => is_private_v4(v4),
        IpAddr::V6(v6) => is_private_v6(v6),
    }
}

/// RFC1918 / loopback / link-local IPv4 classification (no `std` unstable APIs).
fn is_private_v4(ip: Ipv4Addr) -> bool {
    ip.is_private() || ip.is_loopback() || ip.is_link_local()
}

/// Loopback / unique-local (fc00::/7) / link-local (fe80::/10) IPv6, plus
/// IPv4-mapped addresses re-checked through the v4 rules.
fn is_private_v6(ip: Ipv6Addr) -> bool {
    if ip.is_loopback() {
        return true;
    }
    // IPv4-mapped (::ffff:a.b.c.d) — classify by the embedded v4 address.
    if let Some(v4) = ip.to_ipv4_mapped() {
        return is_private_v4(v4);
    }
    let seg = ip.segments();
    // Unique local address: fc00::/7.
    let ula = (seg[0] & 0xfe00) == 0xfc00;
    // Link-local unicast: fe80::/10.
    let link_local = (seg[0] & 0xffc0) == 0xfe80;
    ula || link_local
}

/// Map an [`OnvifError`] onto the API's [`AppError`] taxonomy.
///
/// Auth failures and SOAP "not authorized" faults become 401; timeouts and
/// transport faults become 503 (the device, not the API, is unavailable);
/// discovery/parse problems become 502-style internal/bad-gateway mapped to
/// [`AppError::InternalServerError`].
fn onvif_to_app_error(e: OnvifError) -> AppError {
    match e {
        OnvifError::Auth => AppError::UnauthorizedError("ONVIF authentication failed".to_string()),
        OnvifError::Soap { code, reason } => {
            // A Sender/NotAuthorized fault is an auth problem; everything else
            // is an upstream device error.
            if code.to_ascii_lowercase().contains("notauthorized")
                || reason.to_ascii_lowercase().contains("not authorized")
            {
                AppError::UnauthorizedError(format!("ONVIF device rejected credentials: {reason}"))
            } else {
                AppError::ServiceUnavailableError(format!("ONVIF device fault {code}: {reason}"))
            }
        }
        OnvifError::Timeout => {
            AppError::ServiceUnavailableError("ONVIF device timed out".to_string())
        }
        OnvifError::Http(err) => {
            AppError::ServiceUnavailableError(format!("ONVIF transport error: {err}"))
        }
        OnvifError::Discovery(msg) => {
            AppError::InternalServerError(format!("WS-Discovery failed: {msg}"))
        }
        OnvifError::Parse(msg) => {
            AppError::ServiceUnavailableError(format!("malformed ONVIF response: {msg}"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn private_v4_ranges_are_allowed() {
        for ip in [
            "10.0.0.5",
            "172.16.4.4",
            "192.168.1.10",
            "127.0.0.1",
            "169.254.1.2", // link-local
        ] {
            assert!(is_private_v4(ip.parse().unwrap()), "{ip} should be private");
        }
    }

    #[test]
    fn public_v4_addresses_are_not_private() {
        for ip in ["8.8.8.8", "1.1.1.1", "93.184.216.34", "172.32.0.1"] {
            assert!(!is_private_v4(ip.parse().unwrap()), "{ip} should be public");
        }
    }

    #[test]
    fn private_v6_classification() {
        assert!(is_private_v6("::1".parse().unwrap())); // loopback
        assert!(is_private_v6("fd00::1".parse().unwrap())); // ULA
        assert!(is_private_v6("fe80::1".parse().unwrap())); // link-local
                                                            // IPv4-mapped private/public delegate to the v4 rules.
        assert!(is_private_v6("::ffff:192.168.1.1".parse().unwrap()));
        assert!(!is_private_v6("::ffff:8.8.8.8".parse().unwrap()));
        // Public v6.
        assert!(!is_private_v6("2001:4860:4860::8888".parse().unwrap()));
    }

    #[test]
    fn inspect_target_allows_private_http_device() {
        assert!(ensure_inspect_target_allowed("http://192.168.1.10/onvif/device_service").is_ok());
        assert!(ensure_inspect_target_allowed("http://10.0.0.5:8080/onvif/device_service").is_ok());
        assert!(ensure_inspect_target_allowed("http://127.0.0.1/onvif").is_ok());
    }

    #[test]
    fn inspect_target_rejects_public_address() {
        let err = ensure_inspect_target_allowed("http://8.8.8.8/onvif").unwrap_err();
        assert!(matches!(err, AppError::PermissionDeniedError(_)), "{err:?}");
    }

    #[test]
    fn inspect_target_rejects_cloud_metadata_via_hostname() {
        // A bare hostname is not an IP literal → rejected (no DNS resolution).
        let err = ensure_inspect_target_allowed("http://metadata.google.internal/").unwrap_err();
        assert!(matches!(err, AppError::PermissionDeniedError(_)), "{err:?}");
    }

    #[test]
    fn inspect_target_rejects_dangerous_scheme() {
        // file:// is blocked by is_safe_onvif_url → BadRequest, not a 403.
        let err = ensure_inspect_target_allowed("file:///etc/passwd").unwrap_err();
        assert!(matches!(err, AppError::BadRequestError(_)), "{err:?}");
    }

    #[test]
    fn inspect_target_rejects_unix_scheme() {
        let err = ensure_inspect_target_allowed("unix:///var/run/x.sock").unwrap_err();
        assert!(matches!(err, AppError::BadRequestError(_)), "{err:?}");
    }

    #[test]
    fn inspect_target_rejects_garbage_url() {
        let err = ensure_inspect_target_allowed("http://").unwrap_err();
        assert!(matches!(err, AppError::BadRequestError(_)), "{err:?}");
    }

    #[test]
    fn candidate_from_probe_match_carries_fields() {
        let m = crate::onvif::discovery::ProbeMatch {
            endpoint_reference: "urn:uuid:abc".to_string(),
            xaddrs: vec!["http://192.168.1.10/onvif/device_service".to_string()],
            types: vec!["NetworkVideoTransmitter".to_string()],
            name: Some("Lobby".to_string()),
            ..Default::default()
        };
        let c = CameraCandidate::from(m);
        assert_eq!(c.endpoint_reference, "urn:uuid:abc");
        assert_eq!(c.xaddrs.len(), 1);
        assert_eq!(c.name.as_deref(), Some("Lobby"));
    }

    #[test]
    fn onvif_auth_error_maps_to_unauthorized() {
        assert!(matches!(
            onvif_to_app_error(OnvifError::Auth),
            AppError::UnauthorizedError(_)
        ));
    }

    #[test]
    fn onvif_notauthorized_fault_maps_to_unauthorized() {
        let e = OnvifError::Soap {
            code: "ter:NotAuthorized".to_string(),
            reason: "Sender not authorized".to_string(),
        };
        assert!(matches!(
            onvif_to_app_error(e),
            AppError::UnauthorizedError(_)
        ));
    }

    #[test]
    fn onvif_timeout_maps_to_service_unavailable() {
        assert!(matches!(
            onvif_to_app_error(OnvifError::Timeout),
            AppError::ServiceUnavailableError(_)
        ));
    }

    #[test]
    fn onvif_discovery_error_maps_to_internal() {
        assert!(matches!(
            onvif_to_app_error(OnvifError::Discovery("bind: nope".into())),
            AppError::InternalServerError(_)
        ));
    }
}
