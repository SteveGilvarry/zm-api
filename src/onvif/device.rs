//! ONVIF **Device** service client.
//!
//! Implements the Device-management operations needed to onboard a camera:
//!
//! - [`DeviceClient::get_device_information`] — `GetDeviceInformation`
//!   (manufacturer / model / firmware / serial / hardware id).
//! - [`DeviceClient::get_capabilities`] — `GetCapabilities` (per-service XAddr
//!   URLs for Device, Media, PTZ, Events).
//! - [`DeviceClient::get_services`] — `GetServices` (the newer, namespace-keyed
//!   service list; also yields per-service XAddr URLs).
//!
//! Each method builds the SOAP body, dispatches it through
//! [`OnvifTransport::call`] against the Device service XAddr, and parses the
//! response with quick-xml.
//!
//! ## Parsing philosophy
//!
//! ONVIF responses come from a wide variety of camera firmwares that bind the
//! ONVIF namespaces to arbitrary prefixes (`tds:`, `trt:`, `tptz:`, `tev:`,
//! `wsdl:`, or even a default namespace with no prefix). The parsers here match
//! purely on the **local name** of each element (never the prefix) and treat
//! every field as optional — a missing element yields `None`/empty rather than
//! an error or panic.

use std::time::Duration;

use quick_xml::events::Event;
use quick_xml::reader::Reader;

use crate::onvif::error::{OnvifError, OnvifResult};
use crate::onvif::transport::OnvifTransport;
use crate::onvif::types::{Credentials, ServiceUrls};

/// WSDL namespace for the ONVIF Device service.
const DEVICE_WSDL_NS: &str = "http://www.onvif.org/ver10/device/wsdl";

/// Default per-request timeout when the caller does not specify one.
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);

/// Identity / firmware metadata returned by `GetDeviceInformation`.
///
/// Every field is optional: ONVIF marks Manufacturer/Model/FirmwareVersion/
/// SerialNumber/HardwareId as mandatory in the response, but real cameras
/// occasionally omit some, so we tolerate their absence rather than fail the
/// whole onboarding flow.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DeviceInformation {
    /// Device manufacturer (e.g. `ACME`).
    pub manufacturer: Option<String>,
    /// Device model (e.g. `IPC-1000`).
    pub model: Option<String>,
    /// Firmware version string.
    pub firmware_version: Option<String>,
    /// Hardware serial number.
    pub serial_number: Option<String>,
    /// Hardware identifier.
    pub hardware_id: Option<String>,
}

/// Device capabilities reduced to the per-service endpoint URLs we care about.
///
/// `GetCapabilities` returns a large tree; we extract only the `XAddr` of each
/// service category. Services the device does not advertise are `None`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Capabilities {
    /// Device service XAddr.
    pub device: Option<String>,
    /// Media service XAddr (Profile S).
    pub media: Option<String>,
    /// PTZ service XAddr.
    pub ptz: Option<String>,
    /// Events service XAddr.
    pub events: Option<String>,
}

impl Capabilities {
    /// Fold the discovered service XAddrs into a [`ServiceUrls`], using
    /// `device_fallback` for the Device entry-point when the capabilities tree
    /// omits a Device XAddr (common — the device URL is already known).
    pub fn into_service_urls(self, device_fallback: &str) -> ServiceUrls {
        ServiceUrls {
            device: self.device.unwrap_or_else(|| device_fallback.to_string()),
            media: self.media,
            ptz: self.ptz,
            events: self.events,
        }
    }
}

/// A single entry from `GetServices`: the service namespace plus its XAddr.
///
/// `GetServices` is the modern replacement for `GetCapabilities`; it keys each
/// service by its WSDL namespace (e.g. `http://www.onvif.org/ver10/media/wsdl`)
/// rather than by a fixed element name, which lets us classify services by
/// matching on the namespace substring.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Service {
    /// The service WSDL namespace.
    pub namespace: Option<String>,
    /// The service endpoint address.
    pub xaddr: Option<String>,
}

/// Device service client bound to a single device's Device-service XAddr.
#[derive(Debug, Clone)]
pub struct DeviceClient {
    transport: OnvifTransport,
    /// Device service endpoint URL (the entry point).
    xaddr: String,
    creds: Option<Credentials>,
    timeout: Duration,
}

impl DeviceClient {
    /// Create a Device client for `xaddr` (the device-service endpoint URL),
    /// optionally with WS-Security credentials. Uses the default timeout.
    pub fn new(
        transport: OnvifTransport,
        xaddr: impl Into<String>,
        creds: Option<Credentials>,
    ) -> Self {
        Self {
            transport,
            xaddr: xaddr.into(),
            creds,
            timeout: DEFAULT_TIMEOUT,
        }
    }

    /// Override the per-request timeout.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// The device-service XAddr this client targets.
    pub fn xaddr(&self) -> &str {
        &self.xaddr
    }

    /// `GetDeviceInformation` — manufacturer / model / firmware / serial /
    /// hardware id.
    pub async fn get_device_information(&self) -> OnvifResult<DeviceInformation> {
        let body = format!(
            "<tds:GetDeviceInformation xmlns:tds=\"{ns}\"/>",
            ns = DEVICE_WSDL_NS
        );
        let action = format!("{DEVICE_WSDL_NS}/GetDeviceInformation");
        let xml = self
            .transport
            .call(
                &self.xaddr,
                &action,
                &body,
                self.creds.as_ref(),
                self.timeout,
            )
            .await?;
        parse_device_information(&xml)
    }

    /// `GetCapabilities` — per-service endpoint XAddr URLs. Requests the `All`
    /// category so every service the device advertises is returned.
    pub async fn get_capabilities(&self) -> OnvifResult<Capabilities> {
        let body = format!(
            concat!(
                "<tds:GetCapabilities xmlns:tds=\"{ns}\">",
                "<tds:Category>All</tds:Category>",
                "</tds:GetCapabilities>",
            ),
            ns = DEVICE_WSDL_NS
        );
        let action = format!("{DEVICE_WSDL_NS}/GetCapabilities");
        let xml = self
            .transport
            .call(
                &self.xaddr,
                &action,
                &body,
                self.creds.as_ref(),
                self.timeout,
            )
            .await?;
        parse_capabilities(&xml)
    }

    /// `GetServices` — the namespace-keyed service list. `include_capability`
    /// controls whether the device also embeds each service's capability blob
    /// (we don't parse it, so callers usually pass `false`).
    pub async fn get_services(&self, include_capability: bool) -> OnvifResult<Vec<Service>> {
        let body = format!(
            concat!(
                "<tds:GetServices xmlns:tds=\"{ns}\">",
                "<tds:IncludeCapability>{inc}</tds:IncludeCapability>",
                "</tds:GetServices>",
            ),
            ns = DEVICE_WSDL_NS,
            inc = include_capability,
        );
        let action = format!("{DEVICE_WSDL_NS}/GetServices");
        let xml = self
            .transport
            .call(
                &self.xaddr,
                &action,
                &body,
                self.creds.as_ref(),
                self.timeout,
            )
            .await?;
        parse_services(&xml)
    }

    /// Convenience: classify the [`GetServices`](Self::get_services) result into
    /// [`ServiceUrls`], matching each entry's namespace against the known ONVIF
    /// WSDL namespaces. Falls back to this client's XAddr for the Device entry.
    pub async fn resolve_service_urls(&self) -> OnvifResult<ServiceUrls> {
        let services = self.get_services(false).await?;
        Ok(services_to_urls(&services, &self.xaddr))
    }
}

/// Classify `GetServices` entries into a [`ServiceUrls`] by matching each
/// service namespace against the canonical ONVIF WSDL namespace substrings.
fn services_to_urls(services: &[Service], device_fallback: &str) -> ServiceUrls {
    let mut urls = ServiceUrls::from_device(device_fallback.to_string());
    for s in services {
        let (Some(ns), Some(addr)) = (s.namespace.as_deref(), s.xaddr.clone()) else {
            continue;
        };
        // Namespaces look like `http://www.onvif.org/ver10/media/wsdl`.
        if ns.contains("/device/wsdl") {
            urls.device = addr;
        } else if ns.contains("/media/wsdl") || ns.contains("/media2/wsdl") {
            urls.media = Some(addr);
        } else if ns.contains("/ptz/wsdl") {
            urls.ptz = Some(addr);
        } else if ns.contains("/events/wsdl") {
            urls.events = Some(addr);
        }
    }
    urls
}

/// Strip an XML namespace prefix, returning the local name.
fn local_name(qname: &[u8]) -> String {
    let s = String::from_utf8_lossy(qname);
    match s.rsplit_once(':') {
        Some((_, local)) => local.to_string(),
        None => s.into_owned(),
    }
}

/// Read the text content of the element currently open in `reader` (the parser
/// is positioned just after a `Start` event). Returns the accumulated, escaped
/// text, stopping at the matching `End`. Tolerant of empty elements.
fn read_text(reader: &mut Reader<&[u8]>) -> String {
    let mut depth = 0usize;
    let mut out = String::new();
    loop {
        match reader.read_event() {
            Ok(Event::Text(t)) => {
                if depth == 0 {
                    out.push_str(&t.unescape().unwrap_or_default());
                }
            }
            Ok(Event::CData(t)) => {
                if depth == 0 {
                    out.push_str(&String::from_utf8_lossy(t.as_ref()));
                }
            }
            Ok(Event::Start(_)) => depth += 1,
            Ok(Event::End(_)) => {
                if depth == 0 {
                    break;
                }
                depth -= 1;
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
    }
    out
}

/// Parse a `GetDeviceInformationResponse` into [`DeviceInformation`].
///
/// Matches the five identity elements by local name regardless of prefix and
/// leaves any absent field as `None`.
fn parse_device_information(xml: &str) -> OnvifResult<DeviceInformation> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut info = DeviceInformation::default();
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let local = local_name(e.name().as_ref());
                match local.as_str() {
                    "Manufacturer" => info.manufacturer = non_empty(read_text(&mut reader)),
                    "Model" => info.model = non_empty(read_text(&mut reader)),
                    "FirmwareVersion" => info.firmware_version = non_empty(read_text(&mut reader)),
                    "SerialNumber" => info.serial_number = non_empty(read_text(&mut reader)),
                    "HardwareId" => info.hardware_id = non_empty(read_text(&mut reader)),
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(OnvifError::Parse(format!("device info xml: {e}"))),
            _ => {}
        }
    }
    Ok(info)
}

/// Parse a `GetCapabilitiesResponse`, extracting the `XAddr` of each service
/// category (Device, Media, PTZ, Events).
///
/// The response groups capabilities under `<Capabilities>` with child elements
/// named `Device`, `Media`, `PTZ`, `Events`, each containing an `XAddr`. We
/// track which category element we are inside and capture the next `XAddr`
/// encountered within it. Matching is by local name, so any prefix works.
fn parse_capabilities(xml: &str) -> OnvifResult<Capabilities> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut caps = Capabilities::default();
    // Stack of local element names, used to know which service category an
    // `XAddr` belongs to.
    let mut stack: Vec<String> = Vec::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let local = local_name(e.name().as_ref());
                if local == "XAddr" {
                    let addr = non_empty(read_text(&mut reader));
                    // The XAddr's category is the nearest enclosing
                    // service-category element on the stack.
                    if let Some(cat) = stack.iter().rev().find(|n| is_service_category(n)) {
                        assign_xaddr(&mut caps, cat, addr);
                    }
                    // read_text consumed through the matching End; do not push.
                } else {
                    stack.push(local);
                }
            }
            Ok(Event::End(_)) => {
                stack.pop();
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(OnvifError::Parse(format!("capabilities xml: {e}"))),
            _ => {}
        }
    }
    Ok(caps)
}

/// Whether a local element name names one of the service categories we extract.
fn is_service_category(local: &str) -> bool {
    matches!(local, "Device" | "Media" | "PTZ" | "Events")
}

/// Store an `XAddr` under the matching capability category.
fn assign_xaddr(caps: &mut Capabilities, category: &str, addr: Option<String>) {
    match category {
        "Device" => caps.device = addr,
        "Media" => caps.media = addr,
        "PTZ" => caps.ptz = addr,
        "Events" => caps.events = addr,
        _ => {}
    }
}

/// Parse a `GetServicesResponse` into a list of [`Service`] entries.
///
/// The response contains repeated `<Service>` elements, each with a
/// `<Namespace>` and an `<XAddr>` child. We collect one [`Service`] per
/// `<Service>` element; missing children stay `None`.
fn parse_services(xml: &str) -> OnvifResult<Vec<Service>> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut services = Vec::new();
    let mut current: Option<Service> = None;

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let local = local_name(e.name().as_ref());
                match local.as_str() {
                    "Service" => current = Some(Service::default()),
                    "Namespace" => {
                        let v = non_empty(read_text(&mut reader));
                        if let Some(s) = current.as_mut() {
                            s.namespace = v;
                        }
                    }
                    "XAddr" => {
                        let v = non_empty(read_text(&mut reader));
                        if let Some(s) = current.as_mut() {
                            s.xaddr = v;
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::End(e)) => {
                if local_name(e.name().as_ref()) == "Service" {
                    if let Some(s) = current.take() {
                        services.push(s);
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(OnvifError::Parse(format!("services xml: {e}"))),
            _ => {}
        }
    }
    Ok(services)
}

/// Normalize an empty/whitespace string to `None`.
fn non_empty(s: String) -> Option<String> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- GetDeviceInformation ------------------------------------------

    /// A normal, well-formed response with the conventional `tds:` prefix.
    const DEVICE_INFO_NORMAL: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope">
  <s:Body>
    <tds:GetDeviceInformationResponse xmlns:tds="http://www.onvif.org/ver10/device/wsdl">
      <tds:Manufacturer>ACME</tds:Manufacturer>
      <tds:Model>IPC-1000</tds:Model>
      <tds:FirmwareVersion>V5.3.0 build 200101</tds:FirmwareVersion>
      <tds:SerialNumber>SN-ABC-12345</tds:SerialNumber>
      <tds:HardwareId>HW-7788</tds:HardwareId>
    </tds:GetDeviceInformationResponse>
  </s:Body>
</s:Envelope>"#;

    /// Same payload but with arbitrary, unconventional prefixes and a
    /// default-namespaced SOAP envelope. The parser must key on local names.
    const DEVICE_INFO_PREFIX_VARIED: &str = r#"<?xml version="1.0"?>
<Envelope xmlns="http://www.w3.org/2003/05/soap-envelope"
          xmlns:wsdl="http://www.onvif.org/ver10/device/wsdl">
  <Body>
    <wsdl:GetDeviceInformationResponse>
      <wsdl:Manufacturer>Hikvision</wsdl:Manufacturer>
      <wsdl:Model>DS-2CD2042WD</wsdl:Model>
      <wsdl:FirmwareVersion>V5.4.5</wsdl:FirmwareVersion>
      <wsdl:SerialNumber>DS-XYZ</wsdl:SerialNumber>
      <wsdl:HardwareId>88</wsdl:HardwareId>
    </wsdl:GetDeviceInformationResponse>
  </Body>
</Envelope>"#;

    /// Optional fields omitted: only Manufacturer and Model present. Must not
    /// panic; the missing fields stay `None`.
    const DEVICE_INFO_MISSING: &str = r#"<s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope">
  <s:Body>
    <tds:GetDeviceInformationResponse xmlns:tds="http://www.onvif.org/ver10/device/wsdl">
      <tds:Manufacturer>Axis</tds:Manufacturer>
      <tds:Model>M3045</tds:Model>
      <tds:SerialNumber></tds:SerialNumber>
    </tds:GetDeviceInformationResponse>
  </s:Body>
</s:Envelope>"#;

    #[test]
    fn device_info_normal() {
        let info = parse_device_information(DEVICE_INFO_NORMAL).expect("parse");
        assert_eq!(info.manufacturer.as_deref(), Some("ACME"));
        assert_eq!(info.model.as_deref(), Some("IPC-1000"));
        assert_eq!(
            info.firmware_version.as_deref(),
            Some("V5.3.0 build 200101")
        );
        assert_eq!(info.serial_number.as_deref(), Some("SN-ABC-12345"));
        assert_eq!(info.hardware_id.as_deref(), Some("HW-7788"));
    }

    #[test]
    fn device_info_prefix_varied() {
        // Different prefixes (wsdl:) and a default-namespaced envelope must
        // yield identical parsing — we match on local name only.
        let info = parse_device_information(DEVICE_INFO_PREFIX_VARIED).expect("parse");
        assert_eq!(info.manufacturer.as_deref(), Some("Hikvision"));
        assert_eq!(info.model.as_deref(), Some("DS-2CD2042WD"));
        assert_eq!(info.firmware_version.as_deref(), Some("V5.4.5"));
        assert_eq!(info.serial_number.as_deref(), Some("DS-XYZ"));
        assert_eq!(info.hardware_id.as_deref(), Some("88"));
    }

    #[test]
    fn device_info_missing_optionals() {
        let info = parse_device_information(DEVICE_INFO_MISSING).expect("parse");
        assert_eq!(info.manufacturer.as_deref(), Some("Axis"));
        assert_eq!(info.model.as_deref(), Some("M3045"));
        // Absent and empty elements normalize to None.
        assert_eq!(info.firmware_version, None);
        assert_eq!(info.serial_number, None);
        assert_eq!(info.hardware_id, None);
    }

    // ---- GetCapabilities -----------------------------------------------

    /// Normal capabilities with the four service categories we extract.
    const CAPS_NORMAL: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope">
  <s:Body>
    <tds:GetCapabilitiesResponse xmlns:tds="http://www.onvif.org/ver10/device/wsdl"
                                 xmlns:tt="http://www.onvif.org/ver10/schema">
      <tds:Capabilities>
        <tt:Device>
          <tt:XAddr>http://192.168.1.10/onvif/device_service</tt:XAddr>
          <tt:Network><tt:IPFilter>false</tt:IPFilter></tt:Network>
        </tt:Device>
        <tt:Media>
          <tt:XAddr>http://192.168.1.10/onvif/media_service</tt:XAddr>
          <tt:StreamingCapabilities><tt:RTPMulticast>true</tt:RTPMulticast></tt:StreamingCapabilities>
        </tt:Media>
        <tt:PTZ>
          <tt:XAddr>http://192.168.1.10/onvif/ptz_service</tt:XAddr>
        </tt:PTZ>
        <tt:Events>
          <tt:XAddr>http://192.168.1.10/onvif/event_service</tt:XAddr>
          <tt:WSSubscriptionPolicySupport>true</tt:WSSubscriptionPolicySupport>
        </tt:Events>
      </tds:Capabilities>
    </tds:GetCapabilitiesResponse>
  </s:Body>
</s:Envelope>"#;

    /// Prefix-varied capabilities: default-namespaced SOAP, ONVIF schema bound
    /// to `onvif:` instead of `tt:`, device service to `dev:`.
    const CAPS_PREFIX_VARIED: &str = r#"<Envelope xmlns="http://www.w3.org/2003/05/soap-envelope">
  <Body>
    <dev:GetCapabilitiesResponse xmlns:dev="http://www.onvif.org/ver10/device/wsdl"
                                 xmlns:onvif="http://www.onvif.org/ver10/schema">
      <dev:Capabilities>
        <onvif:Media>
          <onvif:XAddr>http://cam.local/onvif/Media</onvif:XAddr>
        </onvif:Media>
        <onvif:PTZ>
          <onvif:XAddr>http://cam.local/onvif/PTZ</onvif:XAddr>
        </onvif:PTZ>
      </dev:Capabilities>
    </dev:GetCapabilitiesResponse>
  </Body>
</Envelope>"#;

    /// A device that advertises only Media — PTZ/Events/Device XAddrs absent.
    const CAPS_MISSING: &str = r#"<s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope">
  <s:Body>
    <tds:GetCapabilitiesResponse xmlns:tds="http://www.onvif.org/ver10/device/wsdl"
                                 xmlns:tt="http://www.onvif.org/ver10/schema">
      <tds:Capabilities>
        <tt:Media>
          <tt:XAddr>http://10.0.0.5/onvif/media</tt:XAddr>
        </tt:Media>
      </tds:Capabilities>
    </tds:GetCapabilitiesResponse>
  </s:Body>
</s:Envelope>"#;

    #[test]
    fn capabilities_normal() {
        let caps = parse_capabilities(CAPS_NORMAL).expect("parse");
        assert_eq!(
            caps.device.as_deref(),
            Some("http://192.168.1.10/onvif/device_service")
        );
        assert_eq!(
            caps.media.as_deref(),
            Some("http://192.168.1.10/onvif/media_service")
        );
        assert_eq!(
            caps.ptz.as_deref(),
            Some("http://192.168.1.10/onvif/ptz_service")
        );
        assert_eq!(
            caps.events.as_deref(),
            Some("http://192.168.1.10/onvif/event_service")
        );
    }

    #[test]
    fn capabilities_into_service_urls() {
        let caps = parse_capabilities(CAPS_NORMAL).expect("parse");
        let urls = caps.into_service_urls("http://fallback/device");
        assert_eq!(urls.device, "http://192.168.1.10/onvif/device_service");
        assert_eq!(
            urls.media.as_deref(),
            Some("http://192.168.1.10/onvif/media_service")
        );
        assert_eq!(
            urls.ptz.as_deref(),
            Some("http://192.168.1.10/onvif/ptz_service")
        );
        assert_eq!(
            urls.events.as_deref(),
            Some("http://192.168.1.10/onvif/event_service")
        );
    }

    #[test]
    fn capabilities_prefix_varied() {
        let caps = parse_capabilities(CAPS_PREFIX_VARIED).expect("parse");
        assert_eq!(caps.media.as_deref(), Some("http://cam.local/onvif/Media"));
        assert_eq!(caps.ptz.as_deref(), Some("http://cam.local/onvif/PTZ"));
        assert_eq!(caps.device, None);
        assert_eq!(caps.events, None);
    }

    #[test]
    fn capabilities_missing_services() {
        let caps = parse_capabilities(CAPS_MISSING).expect("parse");
        assert_eq!(caps.media.as_deref(), Some("http://10.0.0.5/onvif/media"));
        assert_eq!(caps.device, None);
        assert_eq!(caps.ptz, None);
        assert_eq!(caps.events, None);
        // Fallback fills the device entry-point.
        let urls = caps.into_service_urls("http://10.0.0.5/onvif/device_service");
        assert_eq!(urls.device, "http://10.0.0.5/onvif/device_service");
    }

    // ---- GetServices ----------------------------------------------------

    /// Normal GetServices response with the four core services.
    const SERVICES_NORMAL: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope">
  <s:Body>
    <tds:GetServicesResponse xmlns:tds="http://www.onvif.org/ver10/device/wsdl">
      <tds:Service>
        <tds:Namespace>http://www.onvif.org/ver10/device/wsdl</tds:Namespace>
        <tds:XAddr>http://192.168.1.10/onvif/device_service</tds:XAddr>
        <tds:Version><tds:Major>2</tds:Major><tds:Minor>6</tds:Minor></tds:Version>
      </tds:Service>
      <tds:Service>
        <tds:Namespace>http://www.onvif.org/ver10/media/wsdl</tds:Namespace>
        <tds:XAddr>http://192.168.1.10/onvif/media_service</tds:XAddr>
      </tds:Service>
      <tds:Service>
        <tds:Namespace>http://www.onvif.org/ver20/ptz/wsdl</tds:Namespace>
        <tds:XAddr>http://192.168.1.10/onvif/ptz_service</tds:XAddr>
      </tds:Service>
      <tds:Service>
        <tds:Namespace>http://www.onvif.org/ver10/events/wsdl</tds:Namespace>
        <tds:XAddr>http://192.168.1.10/onvif/event_service</tds:XAddr>
      </tds:Service>
    </tds:GetServicesResponse>
  </s:Body>
</s:Envelope>"#;

    /// Prefix-varied: arbitrary `q1:` prefix and default-namespaced envelope.
    const SERVICES_PREFIX_VARIED: &str = r#"<Envelope xmlns="http://www.w3.org/2003/05/soap-envelope">
  <Body>
    <q1:GetServicesResponse xmlns:q1="http://www.onvif.org/ver10/device/wsdl">
      <q1:Service>
        <q1:Namespace>http://www.onvif.org/ver10/media/wsdl</q1:Namespace>
        <q1:XAddr>http://cam/onvif/media</q1:XAddr>
      </q1:Service>
      <q1:Service>
        <q1:Namespace>http://www.onvif.org/ver10/device/wsdl</q1:Namespace>
        <q1:XAddr>http://cam/onvif/device_service</q1:XAddr>
      </q1:Service>
    </q1:GetServicesResponse>
  </Body>
</Envelope>"#;

    /// A Service entry missing its XAddr (optional-field tolerance).
    const SERVICES_MISSING: &str = r#"<s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope">
  <s:Body>
    <tds:GetServicesResponse xmlns:tds="http://www.onvif.org/ver10/device/wsdl">
      <tds:Service>
        <tds:Namespace>http://www.onvif.org/ver10/media/wsdl</tds:Namespace>
      </tds:Service>
      <tds:Service>
        <tds:XAddr>http://cam/onvif/orphan</tds:XAddr>
      </tds:Service>
    </tds:GetServicesResponse>
  </s:Body>
</s:Envelope>"#;

    #[test]
    fn services_normal() {
        let services = parse_services(SERVICES_NORMAL).expect("parse");
        assert_eq!(services.len(), 4);
        assert_eq!(
            services[0].namespace.as_deref(),
            Some("http://www.onvif.org/ver10/device/wsdl")
        );
        assert_eq!(
            services[0].xaddr.as_deref(),
            Some("http://192.168.1.10/onvif/device_service")
        );

        // Classification into ServiceUrls.
        let urls = services_to_urls(&services, "http://fallback");
        assert_eq!(urls.device, "http://192.168.1.10/onvif/device_service");
        assert_eq!(
            urls.media.as_deref(),
            Some("http://192.168.1.10/onvif/media_service")
        );
        assert_eq!(
            urls.ptz.as_deref(),
            Some("http://192.168.1.10/onvif/ptz_service")
        );
        assert_eq!(
            urls.events.as_deref(),
            Some("http://192.168.1.10/onvif/event_service")
        );
    }

    #[test]
    fn services_prefix_varied() {
        let services = parse_services(SERVICES_PREFIX_VARIED).expect("parse");
        assert_eq!(services.len(), 2);
        let urls = services_to_urls(&services, "http://fallback");
        assert_eq!(urls.device, "http://cam/onvif/device_service");
        assert_eq!(urls.media.as_deref(), Some("http://cam/onvif/media"));
        // Not advertised.
        assert_eq!(urls.ptz, None);
        assert_eq!(urls.events, None);
    }

    #[test]
    fn services_missing_fields() {
        let services = parse_services(SERVICES_MISSING).expect("parse");
        assert_eq!(services.len(), 2);
        // First has namespace but no XAddr.
        assert_eq!(
            services[0].namespace.as_deref(),
            Some("http://www.onvif.org/ver10/media/wsdl")
        );
        assert_eq!(services[0].xaddr, None);
        // Second has XAddr but no namespace.
        assert_eq!(services[1].namespace, None);
        assert_eq!(
            services[1].xaddr.as_deref(),
            Some("http://cam/onvif/orphan")
        );

        // A namespace-less entry cannot be classified; a XAddr-less entry is
        // skipped. So no service URL is resolved here beyond the fallback.
        let urls = services_to_urls(&services, "http://fallback");
        assert_eq!(urls.device, "http://fallback");
        assert_eq!(urls.media, None);
    }

    #[test]
    fn garbage_input_does_not_panic() {
        // Robustness: malformed but non-empty XML must not panic; it yields an
        // empty/parse-error result, never a crash.
        let _ = parse_device_information("<not><closed>");
        let _ = parse_capabilities("plain text, not xml at all");
        let _ = parse_services("<Service><Service>");
    }
}
