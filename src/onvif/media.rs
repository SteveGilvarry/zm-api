//! ONVIF Media service client (Profile S, `ver10/media/wsdl`).
//!
//! Implements the subset of the Media service needed to drive the existing
//! Retina-based `MonitorSource`:
//!
//! - **GetProfiles** — enumerate media profiles (token, name, video resolution
//!   and encoding).
//! - **GetStreamUri** — resolve an RTP-Unicast/RTSP stream URI for a profile.
//! - **GetSnapshotUri** — resolve a JPEG snapshot URI for a profile.
//!
//! Each method builds the SOAP body for the operation, hands it to
//! [`OnvifTransport::call`] against the Media service XAddr, and parses the
//! response with `quick-xml`.
//!
//! ## Parsing strategy
//!
//! ONVIF devices bind WSDL namespaces to arbitrary prefixes (`trt:`, `tt:`,
//! `tds:`, …) and frequently omit optional elements. The parsers here therefore
//! match on the **local name** of each element (prefix-agnostic) and treat every
//! field except the structural minimum as optional, never panicking on a missing
//! element.

use std::time::Duration;

use quick_xml::events::Event;
use quick_xml::reader::Reader;

use crate::onvif::error::{OnvifError, OnvifResult};
use crate::onvif::transport::OnvifTransport;
use crate::onvif::types::Credentials;

/// ONVIF Media service WSDL namespace (`trt:` by convention).
const MEDIA_NS: &str = "http://www.onvif.org/ver10/media/wsdl";
/// ONVIF schema namespace for common types (`tt:` by convention).
const SCHEMA_NS: &str = "http://www.onvif.org/ver10/schema";
/// SOAPAction base for the Media service operations.
const ACTION_BASE: &str = "http://www.onvif.org/ver10/media/wsdl";

/// A media profile as reported by `GetProfiles`.
///
/// Only the fields zm_api consumes are surfaced. `token` is the stable handle
/// used in subsequent `GetStreamUri`/`GetSnapshotUri` calls; the video encoder
/// fields are optional because a profile may lack a video source configuration.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MediaProfile {
    /// Profile token (the `token` attribute on `<Profiles>`), e.g. `Profile_1`.
    pub token: String,
    /// Human-readable profile name (`<Name>`), if present.
    pub name: Option<String>,
    /// Video encoding (`H264`, `H265`, `JPEG`, …) from the video encoder config.
    pub encoding: Option<String>,
    /// Encoded video width in pixels.
    pub width: Option<u32>,
    /// Encoded video height in pixels.
    pub height: Option<u32>,
}

/// A resolved media URI (`GetStreamUri` / `GetSnapshotUri`).
///
/// The transport-level metadata fields are optional; only `uri` is required.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MediaUri {
    /// The stream or snapshot URI (e.g. `rtsp://host:554/Streaming/...`).
    pub uri: String,
    /// Whether the device requires re-fetching the URI before it expires.
    pub invalid_after_connect: Option<bool>,
    /// Whether the URI is invalidated after a reboot.
    pub invalid_after_reboot: Option<bool>,
    /// ISO-8601 duration the URI is valid for (e.g. `PT0S` = no timeout).
    pub timeout: Option<String>,
}

/// Transport protocol requested in a `StreamSetup`.
///
/// For the Retina-backed `MonitorSource` we always want unicast RTP delivered
/// over an RTSP-negotiated session, i.e. [`StreamTransport::RtspUnicast`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamTransport {
    /// `Stream=RTP-Unicast`, `Transport/Protocol=RTSP` — the standard "give me
    /// an `rtsp://` URL" setup.
    RtspUnicast,
    /// `Stream=RTP-Unicast`, `Transport/Protocol=UDP`.
    UdpUnicast,
    /// `Stream=RTP-Multicast`, `Transport/Protocol=UDP`.
    UdpMulticast,
}

impl StreamTransport {
    /// `<tt:Stream>` value for this setup.
    fn stream(self) -> &'static str {
        match self {
            StreamTransport::RtspUnicast | StreamTransport::UdpUnicast => "RTP-Unicast",
            StreamTransport::UdpMulticast => "RTP-Multicast",
        }
    }

    /// `<tt:Transport><tt:Protocol>` value for this setup.
    fn protocol(self) -> &'static str {
        match self {
            StreamTransport::RtspUnicast => "RTSP",
            StreamTransport::UdpUnicast | StreamTransport::UdpMulticast => "UDP",
        }
    }
}

/// Media service client bound to a single device's Media XAddr.
///
/// Cheap to clone (shares the underlying `reqwest` client via the transport).
#[derive(Debug, Clone)]
pub struct MediaClient {
    transport: OnvifTransport,
    /// Media service endpoint (the XAddr discovered via `GetCapabilities`).
    service_url: String,
    /// WS-Security credentials; `None` for devices with auth disabled.
    creds: Option<Credentials>,
    /// Per-call timeout.
    timeout: Duration,
}

impl MediaClient {
    /// Default per-operation timeout.
    pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);

    /// Construct a Media client for the given Media service XAddr.
    pub fn new(
        transport: OnvifTransport,
        service_url: impl Into<String>,
        creds: Option<Credentials>,
    ) -> Self {
        Self {
            transport,
            service_url: service_url.into(),
            creds,
            timeout: Self::DEFAULT_TIMEOUT,
        }
    }

    /// Override the per-operation timeout.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// `GetProfiles` — enumerate the device's media profiles.
    pub async fn get_profiles(&self) -> OnvifResult<Vec<MediaProfile>> {
        let body = format!("<trt:GetProfiles xmlns:trt=\"{MEDIA_NS}\"/>");
        let xml = self
            .transport
            .call(
                &self.service_url,
                &format!("{ACTION_BASE}/GetProfiles"),
                &body,
                self.creds.as_ref(),
                self.timeout,
            )
            .await?;
        parse_profiles(&xml)
    }

    /// `GetStreamUri` — resolve the streaming URI for a profile.
    ///
    /// For `MonitorSource` use [`StreamTransport::RtspUnicast`], which yields an
    /// `rtsp://` URI that drops straight into the Retina source.
    pub async fn get_stream_uri(
        &self,
        profile_token: &str,
        transport: StreamTransport,
    ) -> OnvifResult<MediaUri> {
        let body = format!(
            concat!(
                "<trt:GetStreamUri xmlns:trt=\"{media}\" xmlns:tt=\"{schema}\">",
                "<trt:StreamSetup>",
                "<tt:Stream>{stream}</tt:Stream>",
                "<tt:Transport><tt:Protocol>{proto}</tt:Protocol></tt:Transport>",
                "</trt:StreamSetup>",
                "<trt:ProfileToken>{token}</trt:ProfileToken>",
                "</trt:GetStreamUri>",
            ),
            media = MEDIA_NS,
            schema = SCHEMA_NS,
            stream = transport.stream(),
            proto = transport.protocol(),
            token = xml_escape(profile_token),
        );
        let xml = self
            .transport
            .call(
                &self.service_url,
                &format!("{ACTION_BASE}/GetStreamUri"),
                &body,
                self.creds.as_ref(),
                self.timeout,
            )
            .await?;
        parse_media_uri(&xml)
    }

    /// `GetSnapshotUri` — resolve the JPEG snapshot URI for a profile.
    pub async fn get_snapshot_uri(&self, profile_token: &str) -> OnvifResult<MediaUri> {
        let body = format!(
            concat!(
                "<trt:GetSnapshotUri xmlns:trt=\"{media}\">",
                "<trt:ProfileToken>{token}</trt:ProfileToken>",
                "</trt:GetSnapshotUri>",
            ),
            media = MEDIA_NS,
            token = xml_escape(profile_token),
        );
        let xml = self
            .transport
            .call(
                &self.service_url,
                &format!("{ACTION_BASE}/GetSnapshotUri"),
                &body,
                self.creds.as_ref(),
                self.timeout,
            )
            .await?;
        parse_media_uri(&xml)
    }
}

/// Minimal XML escaping for element/attribute text we emit.
fn xml_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(c),
        }
    }
    out
}

/// Strip an XML namespace prefix, returning the local name.
fn local_name(qname: &[u8]) -> String {
    let s = String::from_utf8_lossy(qname);
    match s.rsplit_once(':') {
        Some((_, local)) => local.to_string(),
        None => s.into_owned(),
    }
}

/// Find an attribute by local name (prefix-agnostic) on a start tag.
fn attr_local<'a>(e: &'a quick_xml::events::BytesStart<'a>, want: &str) -> Option<String> {
    for attr in e.attributes().flatten() {
        if local_name(attr.key.as_ref()) == want {
            return Some(String::from_utf8_lossy(&attr.value).into_owned());
        }
    }
    None
}

/// Parse a `GetProfilesResponse` into a list of [`MediaProfile`].
///
/// Prefix-agnostic and tolerant of missing optional fields. Each `<Profiles>`
/// (or `<Profile>`) element with a `token` attribute becomes one profile; its
/// `Name` and nested `VideoEncoderConfiguration` (encoding + resolution) are
/// captured when present.
fn parse_profiles(xml: &str) -> OnvifResult<Vec<MediaProfile>> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut profiles: Vec<MediaProfile> = Vec::new();
    // Local-name stack used to scope where text belongs.
    let mut stack: Vec<String> = Vec::new();
    // The profile currently being assembled (between Profiles start/end).
    let mut current: Option<MediaProfile> = None;

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let local = local_name(e.name().as_ref());
                if local == "Profiles" || local == "Profile" {
                    // A new profile begins. Its token is an attribute.
                    let mut p = MediaProfile::default();
                    if let Some(tok) = attr_local(&e, "token") {
                        p.token = tok;
                    }
                    current = Some(p);
                }
                stack.push(local);
            }
            Ok(Event::Empty(e)) => {
                // Self-closing element: may carry the token attribute (rare, but
                // a profile could be empty) — handle it without a matching End.
                let local = local_name(e.name().as_ref());
                if local == "Profiles" || local == "Profile" {
                    let mut p = MediaProfile::default();
                    if let Some(tok) = attr_local(&e, "token") {
                        p.token = tok;
                    }
                    if !p.token.is_empty() {
                        profiles.push(p);
                    }
                }
            }
            Ok(Event::Text(t)) => {
                if let Some(p) = current.as_mut() {
                    let text = t.unescape().unwrap_or_default().into_owned();
                    if text.is_empty() {
                        // Skip whitespace-only nodes.
                    } else if let Some(local) = stack.last() {
                        assign_profile_field(p, &stack, local, &text);
                    }
                }
            }
            Ok(Event::End(e)) => {
                let local = local_name(e.name().as_ref());
                if local == "Profiles" || local == "Profile" {
                    if let Some(p) = current.take() {
                        if !p.token.is_empty() {
                            profiles.push(p);
                        }
                    }
                }
                stack.pop();
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(OnvifError::Parse(format!("GetProfiles XML: {e}")));
            }
            _ => {}
        }
    }

    Ok(profiles)
}

/// Assign a text node to the right field of the in-progress profile, based on
/// the local-name stack context. Tolerates fields appearing in any order and
/// silently ignores elements we don't model.
fn assign_profile_field(p: &mut MediaProfile, stack: &[String], local: &str, text: &str) {
    match local {
        // `<Name>` directly inside the profile (not inside a nested config's
        // own Name). We only take it when its parent is the profile element.
        "Name" if parent_is(stack, &["Profiles", "Profile"]) => {
            if p.name.is_none() {
                p.name = Some(text.to_string());
            }
        }
        // `<Encoding>` inside a VideoEncoderConfiguration.
        "Encoding" if ancestor_is(stack, "VideoEncoderConfiguration") => {
            if p.encoding.is_none() {
                p.encoding = Some(text.to_string());
            }
        }
        // `<Width>` / `<Height>` inside a Resolution (which itself is inside the
        // VideoEncoderConfiguration). Guard on the Resolution ancestor so we
        // don't pick up rate-control or other numeric fields.
        "Width" if ancestor_is(stack, "Resolution") && p.width.is_none() => {
            p.width = text.parse().ok();
        }
        "Height" if ancestor_is(stack, "Resolution") && p.height.is_none() => {
            p.height = text.parse().ok();
        }
        _ => {}
    }
}

/// True when the element enclosing the current text (the stack top) has one of
/// the given local names. The stack top is the element whose text we're in.
fn parent_is(stack: &[String], any_of: &[&str]) -> bool {
    // stack.last() is the element currently open (whose text this is); its
    // parent is the element before it.
    if stack.len() < 2 {
        return false;
    }
    let parent = &stack[stack.len() - 2];
    any_of.iter().any(|n| n == parent)
}

/// True when any ancestor on the stack (excluding the current element) has the
/// given local name.
fn ancestor_is(stack: &[String], name: &str) -> bool {
    if stack.is_empty() {
        return false;
    }
    stack[..stack.len() - 1].iter().any(|s| s == name)
}

/// Parse a `GetStreamUriResponse` / `GetSnapshotUriResponse` into a
/// [`MediaUri`]. Both responses wrap a `<MediaUri>`/`<Uri>` payload with the
/// same shape: a `<Uri>` plus optional validity metadata.
fn parse_media_uri(xml: &str) -> OnvifResult<MediaUri> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut out = MediaUri::default();
    let mut found_uri = false;
    let mut stack: Vec<String> = Vec::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                stack.push(local_name(e.name().as_ref()));
            }
            Ok(Event::Text(t)) => {
                let text = t.unescape().unwrap_or_default().into_owned();
                if text.is_empty() {
                    // ignore whitespace
                } else if let Some(local) = stack.last() {
                    match local.as_str() {
                        "Uri" => {
                            if !found_uri {
                                out.uri = text;
                                found_uri = true;
                            }
                        }
                        "InvalidAfterConnect" => {
                            out.invalid_after_connect = parse_bool(&text);
                        }
                        "InvalidAfterReboot" => {
                            out.invalid_after_reboot = parse_bool(&text);
                        }
                        "Timeout" => {
                            out.timeout = Some(text);
                        }
                        _ => {}
                    }
                }
            }
            Ok(Event::End(_)) => {
                stack.pop();
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(OnvifError::Parse(format!("GetUri XML: {e}")));
            }
            _ => {}
        }
    }

    if !found_uri || out.uri.is_empty() {
        return Err(OnvifError::Parse(
            "response did not contain a <Uri> element".to_string(),
        ));
    }
    Ok(out)
}

/// Parse an XML-schema boolean (`true`/`false`/`1`/`0`).
fn parse_bool(s: &str) -> Option<bool> {
    match s.trim() {
        "true" | "1" => Some(true),
        "false" | "0" => Some(false),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ----- GetProfiles -------------------------------------------------------

    /// Representative two-profile `GetProfilesResponse` with the conventional
    /// `trt:`/`tt:` prefixes.
    const PROFILES_NORMAL: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
    <SOAP-ENV:Envelope xmlns:SOAP-ENV="http://www.w3.org/2003/05/soap-envelope"
                       xmlns:trt="http://www.onvif.org/ver10/media/wsdl"
                       xmlns:tt="http://www.onvif.org/ver10/schema">
      <SOAP-ENV:Body>
        <trt:GetProfilesResponse>
          <trt:Profiles token="Profile_1" fixed="true">
            <tt:Name>MainStream</tt:Name>
            <tt:VideoSourceConfiguration token="VSC_1">
              <tt:Name>VideoSource</tt:Name>
            </tt:VideoSourceConfiguration>
            <tt:VideoEncoderConfiguration token="VEC_1">
              <tt:Name>VideoEncoder</tt:Name>
              <tt:Encoding>H264</tt:Encoding>
              <tt:Resolution>
                <tt:Width>1920</tt:Width>
                <tt:Height>1080</tt:Height>
              </tt:Resolution>
              <tt:RateControl>
                <tt:FrameRateLimit>30</tt:FrameRateLimit>
                <tt:EncodingInterval>1</tt:EncodingInterval>
                <tt:BitrateLimit>4096</tt:BitrateLimit>
              </tt:RateControl>
            </tt:VideoEncoderConfiguration>
          </trt:Profiles>
          <trt:Profiles token="Profile_2">
            <tt:Name>SubStream</tt:Name>
            <tt:VideoEncoderConfiguration token="VEC_2">
              <tt:Encoding>H265</tt:Encoding>
              <tt:Resolution>
                <tt:Width>640</tt:Width>
                <tt:Height>360</tt:Height>
              </tt:Resolution>
            </tt:VideoEncoderConfiguration>
          </trt:Profiles>
        </trt:GetProfilesResponse>
      </SOAP-ENV:Body>
    </SOAP-ENV:Envelope>"#;

    #[test]
    fn parses_normal_profiles() {
        let profiles = parse_profiles(PROFILES_NORMAL).expect("parse");
        assert_eq!(profiles.len(), 2);

        let p1 = &profiles[0];
        assert_eq!(p1.token, "Profile_1");
        assert_eq!(p1.name.as_deref(), Some("MainStream"));
        assert_eq!(p1.encoding.as_deref(), Some("H264"));
        assert_eq!(p1.width, Some(1920));
        assert_eq!(p1.height, Some(1080));

        let p2 = &profiles[1];
        assert_eq!(p2.token, "Profile_2");
        assert_eq!(p2.name.as_deref(), Some("SubStream"));
        assert_eq!(p2.encoding.as_deref(), Some("H265"));
        assert_eq!(p2.width, Some(640));
        assert_eq!(p2.height, Some(360));
    }

    #[test]
    fn rate_control_numbers_do_not_leak_into_resolution() {
        // Regression: Width/Height must only come from <Resolution>, not from
        // RateControl/Bitrate numeric fields.
        let profiles = parse_profiles(PROFILES_NORMAL).expect("parse");
        assert_eq!(profiles[0].width, Some(1920));
        assert_eq!(profiles[0].height, Some(1080));
    }

    /// Same content but with *different, arbitrary* namespace prefixes and a
    /// default namespace on the schema elements — exercises prefix tolerance.
    /// Here the profile element is the singular `Profile` with a `media:`
    /// prefix and schema elements use `s:` instead of `tt:`.
    const PROFILES_PREFIX_VARIED: &str = r#"<?xml version="1.0"?>
    <env:Envelope xmlns:env="http://www.w3.org/2003/05/soap-envelope"
                  xmlns:media="http://www.onvif.org/ver10/media/wsdl"
                  xmlns:s="http://www.onvif.org/ver10/schema">
      <env:Body>
        <media:GetProfilesResponse>
          <media:Profiles token="MyProfile">
            <s:Name>Cam Main</s:Name>
            <s:VideoEncoderConfiguration token="enc">
              <s:Name>EncCfg</s:Name>
              <s:Encoding>JPEG</s:Encoding>
              <s:Resolution>
                <s:Width>1280</s:Width>
                <s:Height>720</s:Height>
              </s:Resolution>
            </s:VideoEncoderConfiguration>
          </media:Profiles>
        </media:GetProfilesResponse>
      </env:Body>
    </env:Envelope>"#;

    #[test]
    fn parses_prefix_varied_profiles() {
        let profiles = parse_profiles(PROFILES_PREFIX_VARIED).expect("parse");
        assert_eq!(profiles.len(), 1);
        let p = &profiles[0];
        assert_eq!(p.token, "MyProfile");
        assert_eq!(p.name.as_deref(), Some("Cam Main"));
        assert_eq!(p.encoding.as_deref(), Some("JPEG"));
        assert_eq!(p.width, Some(1280));
        assert_eq!(p.height, Some(720));
    }

    /// A profile with no video encoder configuration and no name: only the
    /// mandatory `token` attribute is present. Must parse without panicking and
    /// leave the optional fields `None`.
    const PROFILES_MISSING_OPTIONAL: &str = r#"<?xml version="1.0"?>
    <s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope"
                xmlns:trt="http://www.onvif.org/ver10/media/wsdl">
      <s:Body>
        <trt:GetProfilesResponse>
          <trt:Profiles token="BareProfile"/>
          <trt:Profiles token="AudioOnly">
            <tt:AudioEncoderConfiguration xmlns:tt="http://www.onvif.org/ver10/schema" token="aec">
              <tt:Name>Audio</tt:Name>
              <tt:Encoding>AAC</tt:Encoding>
            </tt:AudioEncoderConfiguration>
          </trt:Profiles>
        </trt:GetProfilesResponse>
      </s:Body>
    </s:Envelope>"#;

    #[test]
    fn parses_profiles_with_missing_optional_fields() {
        let profiles = parse_profiles(PROFILES_MISSING_OPTIONAL).expect("parse");
        assert_eq!(profiles.len(), 2);

        // Self-closing profile: token only.
        let bare = &profiles[0];
        assert_eq!(bare.token, "BareProfile");
        assert_eq!(bare.name, None);
        assert_eq!(bare.encoding, None);
        assert_eq!(bare.width, None);
        assert_eq!(bare.height, None);

        // Audio-only profile: must NOT pick up audio Encoding as video encoding
        // (it lives under AudioEncoderConfiguration, not VideoEncoderConfiguration).
        let audio = &profiles[1];
        assert_eq!(audio.token, "AudioOnly");
        assert_eq!(audio.encoding, None);
        assert_eq!(audio.width, None);
        assert_eq!(audio.height, None);
    }

    #[test]
    fn empty_profiles_response_yields_empty_vec() {
        let xml = r#"<s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope">
          <s:Body><trt:GetProfilesResponse
            xmlns:trt="http://www.onvif.org/ver10/media/wsdl"/></s:Body></s:Envelope>"#;
        let profiles = parse_profiles(xml).expect("parse");
        assert!(profiles.is_empty());
    }

    // ----- GetStreamUri ------------------------------------------------------

    const STREAM_URI_NORMAL: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
    <SOAP-ENV:Envelope xmlns:SOAP-ENV="http://www.w3.org/2003/05/soap-envelope"
                       xmlns:trt="http://www.onvif.org/ver10/media/wsdl"
                       xmlns:tt="http://www.onvif.org/ver10/schema">
      <SOAP-ENV:Body>
        <trt:GetStreamUriResponse>
          <trt:MediaUri>
            <tt:Uri>rtsp://192.168.1.10:554/Streaming/Channels/101</tt:Uri>
            <tt:InvalidAfterConnect>false</tt:InvalidAfterConnect>
            <tt:InvalidAfterReboot>false</tt:InvalidAfterReboot>
            <tt:Timeout>PT0S</tt:Timeout>
          </trt:MediaUri>
        </trt:GetStreamUriResponse>
      </SOAP-ENV:Body>
    </SOAP-ENV:Envelope>"#;

    #[test]
    fn parses_normal_stream_uri() {
        let uri = parse_media_uri(STREAM_URI_NORMAL).expect("parse");
        assert_eq!(uri.uri, "rtsp://192.168.1.10:554/Streaming/Channels/101");
        assert_eq!(uri.invalid_after_connect, Some(false));
        assert_eq!(uri.invalid_after_reboot, Some(false));
        assert_eq!(uri.timeout.as_deref(), Some("PT0S"));
    }

    /// Prefix-varied stream URI: arbitrary prefixes, schema-as-default ns, and
    /// boolean given as `1`/`0`.
    const STREAM_URI_PREFIX_VARIED: &str = r#"<?xml version="1.0"?>
    <e:Envelope xmlns:e="http://www.w3.org/2003/05/soap-envelope"
                xmlns:m="http://www.onvif.org/ver10/media/wsdl">
      <e:Body>
        <m:GetStreamUriResponse>
          <m:MediaUri xmlns="http://www.onvif.org/ver10/schema">
            <Uri>rtsp://cam.local/stream1</Uri>
            <InvalidAfterConnect>1</InvalidAfterConnect>
            <InvalidAfterReboot>0</InvalidAfterReboot>
            <Timeout>PT60S</Timeout>
          </m:MediaUri>
        </m:GetStreamUriResponse>
      </e:Body>
    </e:Envelope>"#;

    #[test]
    fn parses_prefix_varied_stream_uri() {
        let uri = parse_media_uri(STREAM_URI_PREFIX_VARIED).expect("parse");
        assert_eq!(uri.uri, "rtsp://cam.local/stream1");
        assert_eq!(uri.invalid_after_connect, Some(true));
        assert_eq!(uri.invalid_after_reboot, Some(false));
        assert_eq!(uri.timeout.as_deref(), Some("PT60S"));
    }

    /// Stream URI response missing all optional validity metadata: only `<Uri>`.
    const STREAM_URI_MISSING_OPTIONAL: &str = r#"<?xml version="1.0"?>
    <s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope">
      <s:Body>
        <trt:GetStreamUriResponse xmlns:trt="http://www.onvif.org/ver10/media/wsdl"
                                  xmlns:tt="http://www.onvif.org/ver10/schema">
          <trt:MediaUri>
            <tt:Uri>rtsp://10.0.0.5/onvif1</tt:Uri>
          </trt:MediaUri>
        </trt:GetStreamUriResponse>
      </s:Body>
    </s:Envelope>"#;

    #[test]
    fn parses_stream_uri_with_missing_optional_fields() {
        let uri = parse_media_uri(STREAM_URI_MISSING_OPTIONAL).expect("parse");
        assert_eq!(uri.uri, "rtsp://10.0.0.5/onvif1");
        assert_eq!(uri.invalid_after_connect, None);
        assert_eq!(uri.invalid_after_reboot, None);
        assert_eq!(uri.timeout, None);
    }

    #[test]
    fn stream_uri_without_uri_element_is_parse_error() {
        let xml = r#"<s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope">
          <s:Body><trt:GetStreamUriResponse
            xmlns:trt="http://www.onvif.org/ver10/media/wsdl">
            <trt:MediaUri/></trt:GetStreamUriResponse></s:Body></s:Envelope>"#;
        let err = parse_media_uri(xml).expect_err("must error");
        assert!(matches!(err, OnvifError::Parse(_)));
    }

    // ----- GetSnapshotUri ----------------------------------------------------

    #[test]
    fn parses_snapshot_uri() {
        // GetSnapshotUriResponse shares the MediaUri shape.
        let xml = r#"<?xml version="1.0"?>
        <s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope"
                    xmlns:trt="http://www.onvif.org/ver10/media/wsdl"
                    xmlns:tt="http://www.onvif.org/ver10/schema">
          <s:Body>
            <trt:GetSnapshotUriResponse>
              <trt:MediaUri>
                <tt:Uri>http://192.168.1.10/onvif-http/snapshot?Profile_1</tt:Uri>
                <tt:InvalidAfterConnect>true</tt:InvalidAfterConnect>
              </trt:MediaUri>
            </trt:GetSnapshotUriResponse>
          </s:Body>
        </s:Envelope>"#;
        let uri = parse_media_uri(xml).expect("parse");
        assert_eq!(uri.uri, "http://192.168.1.10/onvif-http/snapshot?Profile_1");
        assert_eq!(uri.invalid_after_connect, Some(true));
    }

    // ----- request-body builders --------------------------------------------

    #[test]
    fn stream_setup_rtsp_unicast_emits_rtp_unicast_and_rtsp() {
        assert_eq!(StreamTransport::RtspUnicast.stream(), "RTP-Unicast");
        assert_eq!(StreamTransport::RtspUnicast.protocol(), "RTSP");
    }

    #[test]
    fn stream_setup_variants() {
        assert_eq!(StreamTransport::UdpUnicast.stream(), "RTP-Unicast");
        assert_eq!(StreamTransport::UdpUnicast.protocol(), "UDP");
        assert_eq!(StreamTransport::UdpMulticast.stream(), "RTP-Multicast");
        assert_eq!(StreamTransport::UdpMulticast.protocol(), "UDP");
    }

    #[test]
    fn profile_token_is_xml_escaped_in_request() {
        // Tokens with XML-special characters must be escaped to keep the body
        // well-formed. We can verify via the escape helper directly.
        assert_eq!(xml_escape("a&b<c>\""), "a&amp;b&lt;c&gt;&quot;");
    }

    #[test]
    fn fault_response_propagates_as_soap_error_shape() {
        // A response containing a SOAP fault is handled by the transport layer,
        // not the media parsers; the media URI parser itself just won't find a
        // <Uri> and returns a Parse error. Confirm it doesn't panic on a fault
        // body fed directly.
        let fault = r#"<s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope">
          <s:Body><s:Fault>
            <s:Code><s:Value>s:Sender</s:Value></s:Code>
            <s:Reason><s:Text>NoProfile</s:Text></s:Reason>
          </s:Fault></s:Body></s:Envelope>"#;
        let err = parse_media_uri(fault).expect_err("no uri");
        assert!(matches!(err, OnvifError::Parse(_)));
    }
}
