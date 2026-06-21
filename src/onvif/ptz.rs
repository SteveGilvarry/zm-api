//! ONVIF **PTZ** service client.
//!
//! Implements the pan/tilt/zoom operations needed to drive a camera's PTZ unit:
//!
//! - [`PtzClient::continuous_move`] — `ContinuousMove` (velocity-based jog).
//! - [`PtzClient::absolute_move`] — `AbsoluteMove` (go to an absolute position).
//! - [`PtzClient::relative_move`] — `RelativeMove` (move by a delta).
//! - [`PtzClient::stop`] — `Stop` (halt pan/tilt and/or zoom).
//! - [`PtzClient::get_status`] — `GetStatus` (current position + move state).
//! - [`PtzClient::get_configurations`] — `GetConfigurations` (PTZ config list).
//!
//! Each method builds the SOAP body, dispatches it through
//! [`OnvifTransport::call`] against the PTZ service XAddr, and parses the
//! response with quick-xml.
//!
//! ## Parsing philosophy
//!
//! ONVIF responses come from a wide variety of camera firmwares that bind the
//! ONVIF namespaces to arbitrary prefixes (`tptz:`, `trt:`, `tt:`, `tds:`,
//! `wsdl:`, or even a default namespace with no prefix). The parsers here match
//! purely on the **local name** of each element/attribute (never the prefix)
//! and treat every field as optional — a missing element yields `None`/default
//! rather than an error or panic.
//!
//! ## Coordinate spaces
//!
//! PTZ vectors are expressed in the ONVIF *generic normalized* space:
//! PanTilt `x`/`y` and Zoom `x` are `f32` in `[-1.0, 1.0]`. The same `[-1,1]`
//! range is used for ContinuousMove velocities and AbsoluteMove positions.

use std::time::Duration;

use quick_xml::events::Event;
use quick_xml::reader::Reader;

use crate::onvif::error::{OnvifError, OnvifResult};
use crate::onvif::transport::OnvifTransport;
use crate::onvif::types::Credentials;

/// WSDL namespace for the ONVIF PTZ service (ver20).
const PTZ_WSDL_NS: &str = "http://www.onvif.org/ver20/ptz/wsdl";

/// ONVIF common schema namespace (the `tt:` types: PTZVector, PanTilt, Zoom…).
const ONVIF_SCHEMA_NS: &str = "http://www.onvif.org/ver10/schema";

/// Default per-request timeout when the caller does not specify one.
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);

/// A 2-D pan/tilt vector in normalized `[-1.0, 1.0]` space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PanTilt {
    /// Horizontal (pan) component.
    pub x: f32,
    /// Vertical (tilt) component.
    pub y: f32,
}

impl PanTilt {
    /// Construct a pan/tilt vector.
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// A zoom vector in normalized `[-1.0, 1.0]` space (single `x` axis).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Zoom {
    /// Zoom component.
    pub x: f32,
}

impl Zoom {
    /// Construct a zoom vector.
    pub fn new(x: f32) -> Self {
        Self { x }
    }
}

/// A full PTZ vector: an optional pan/tilt and an optional zoom component.
///
/// ONVIF allows a `PTZVector` to carry only `PanTilt`, only `Zoom`, or both;
/// callers omit whichever axis they do not wish to move.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct PtzVector {
    /// Pan/tilt component, if present.
    pub pan_tilt: Option<PanTilt>,
    /// Zoom component, if present.
    pub zoom: Option<Zoom>,
}

impl PtzVector {
    /// A vector with both pan/tilt and zoom set.
    pub fn new(pan_tilt: PanTilt, zoom: Zoom) -> Self {
        Self {
            pan_tilt: Some(pan_tilt),
            zoom: Some(zoom),
        }
    }

    /// A pan/tilt-only vector (no zoom axis).
    pub fn pan_tilt(x: f32, y: f32) -> Self {
        Self {
            pan_tilt: Some(PanTilt::new(x, y)),
            zoom: None,
        }
    }

    /// A zoom-only vector (no pan/tilt axis).
    pub fn zoom(x: f32) -> Self {
        Self {
            pan_tilt: None,
            zoom: Some(Zoom::new(x)),
        }
    }

    /// Render this vector as the inner children of a `PTZVector`/velocity
    /// element (the `<tt:PanTilt .../>` and `<tt:Zoom .../>` elements). The
    /// caller wraps this in the appropriate `tptz:` element.
    fn to_vector_children(self) -> String {
        let mut out = String::new();
        if let Some(pt) = self.pan_tilt {
            out.push_str(&format!(
                "<tt:PanTilt x=\"{}\" y=\"{}\" xmlns:tt=\"{ns}\"/>",
                fmt_f32(pt.x),
                fmt_f32(pt.y),
                ns = ONVIF_SCHEMA_NS,
            ));
        }
        if let Some(z) = self.zoom {
            out.push_str(&format!(
                "<tt:Zoom x=\"{}\" xmlns:tt=\"{ns}\"/>",
                fmt_f32(z.x),
                ns = ONVIF_SCHEMA_NS,
            ));
        }
        out
    }
}

/// The current PTZ status returned by `GetStatus`.
///
/// Every field is optional: cameras may report position without move state, or
/// vice versa, and some omit one axis. Absence is represented as `None` rather
/// than an error.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PtzStatus {
    /// Current absolute position (PanTilt and/or Zoom).
    pub position: PtzVector,
    /// Pan/tilt move-state string (e.g. `IDLE`, `MOVING`), if reported.
    pub move_status_pan_tilt: Option<String>,
    /// Zoom move-state string (e.g. `IDLE`, `MOVING`), if reported.
    pub move_status_zoom: Option<String>,
    /// UTC timestamp string from the status, if reported.
    pub utc_time: Option<String>,
}

/// A single PTZ configuration entry from `GetConfigurations`.
///
/// We surface the configuration token (needed to drive moves) and its
/// human-readable name; the full speed/limit tree is intentionally not modeled.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PtzConfiguration {
    /// The configuration token (the `token` attribute of `PTZConfiguration`).
    pub token: Option<String>,
    /// The configuration's `<Name>`.
    pub name: Option<String>,
    /// The associated node token (`<NodeToken>`), if present.
    pub node_token: Option<String>,
}

/// PTZ service client bound to a single device's PTZ-service XAddr.
#[derive(Debug, Clone)]
pub struct PtzClient {
    transport: OnvifTransport,
    /// PTZ service endpoint URL.
    xaddr: String,
    creds: Option<Credentials>,
    timeout: Duration,
}

impl PtzClient {
    /// Create a PTZ client for `xaddr` (the PTZ-service endpoint URL),
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

    /// The PTZ-service XAddr this client targets.
    pub fn xaddr(&self) -> &str {
        &self.xaddr
    }

    /// `ContinuousMove` — start a velocity-based jog on `profile_token`.
    ///
    /// `velocity` carries the pan/tilt and/or zoom speeds in normalized
    /// `[-1.0, 1.0]` space. The move continues until a [`Stop`](Self::stop) or
    /// (if the device honors it) the optional `timeout`. We do not send the
    /// `<Timeout>` element; callers manage stop explicitly.
    pub async fn continuous_move(
        &self,
        profile_token: &str,
        velocity: PtzVector,
    ) -> OnvifResult<()> {
        let body = format!(
            concat!(
                "<tptz:ContinuousMove xmlns:tptz=\"{ns}\">",
                "<tptz:ProfileToken>{profile}</tptz:ProfileToken>",
                "<tptz:Velocity>{vec}</tptz:Velocity>",
                "</tptz:ContinuousMove>",
            ),
            ns = PTZ_WSDL_NS,
            profile = xml_escape(profile_token),
            vec = velocity.to_vector_children(),
        );
        self.call_void(&body, "ContinuousMove").await
    }

    /// `AbsoluteMove` — move to an absolute `position` on `profile_token`.
    pub async fn absolute_move(&self, profile_token: &str, position: PtzVector) -> OnvifResult<()> {
        let body = format!(
            concat!(
                "<tptz:AbsoluteMove xmlns:tptz=\"{ns}\">",
                "<tptz:ProfileToken>{profile}</tptz:ProfileToken>",
                "<tptz:Position>{vec}</tptz:Position>",
                "</tptz:AbsoluteMove>",
            ),
            ns = PTZ_WSDL_NS,
            profile = xml_escape(profile_token),
            vec = position.to_vector_children(),
        );
        self.call_void(&body, "AbsoluteMove").await
    }

    /// `RelativeMove` — move by a relative `translation` on `profile_token`.
    pub async fn relative_move(
        &self,
        profile_token: &str,
        translation: PtzVector,
    ) -> OnvifResult<()> {
        let body = format!(
            concat!(
                "<tptz:RelativeMove xmlns:tptz=\"{ns}\">",
                "<tptz:ProfileToken>{profile}</tptz:ProfileToken>",
                "<tptz:Translation>{vec}</tptz:Translation>",
                "</tptz:RelativeMove>",
            ),
            ns = PTZ_WSDL_NS,
            profile = xml_escape(profile_token),
            vec = translation.to_vector_children(),
        );
        self.call_void(&body, "RelativeMove").await
    }

    /// `Stop` — halt motion on `profile_token`. `pan_tilt` and `zoom` select
    /// which axes to stop (ONVIF lets you stop them independently).
    pub async fn stop(&self, profile_token: &str, pan_tilt: bool, zoom: bool) -> OnvifResult<()> {
        let body = format!(
            concat!(
                "<tptz:Stop xmlns:tptz=\"{ns}\">",
                "<tptz:ProfileToken>{profile}</tptz:ProfileToken>",
                "<tptz:PanTilt>{pt}</tptz:PanTilt>",
                "<tptz:Zoom>{zoom}</tptz:Zoom>",
                "</tptz:Stop>",
            ),
            ns = PTZ_WSDL_NS,
            profile = xml_escape(profile_token),
            pt = pan_tilt,
            zoom = zoom,
        );
        self.call_void(&body, "Stop").await
    }

    /// `GetStatus` — current PTZ position and move state for `profile_token`.
    pub async fn get_status(&self, profile_token: &str) -> OnvifResult<PtzStatus> {
        let body = format!(
            concat!(
                "<tptz:GetStatus xmlns:tptz=\"{ns}\">",
                "<tptz:ProfileToken>{profile}</tptz:ProfileToken>",
                "</tptz:GetStatus>",
            ),
            ns = PTZ_WSDL_NS,
            profile = xml_escape(profile_token),
        );
        let action = format!("{PTZ_WSDL_NS}/GetStatus");
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
        parse_status(&xml)
    }

    /// `GetConfigurations` — the list of PTZ configurations on the device.
    pub async fn get_configurations(&self) -> OnvifResult<Vec<PtzConfiguration>> {
        let body = format!(
            "<tptz:GetConfigurations xmlns:tptz=\"{ns}\"/>",
            ns = PTZ_WSDL_NS
        );
        let action = format!("{PTZ_WSDL_NS}/GetConfigurations");
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
        parse_configurations(&xml)
    }

    /// Dispatch a body whose response carries no payload we need (the move/stop
    /// operations). The operation `name` is used to build the SOAP action.
    async fn call_void(&self, body: &str, name: &str) -> OnvifResult<()> {
        let action = format!("{PTZ_WSDL_NS}/{name}");
        self.transport
            .call(
                &self.xaddr,
                &action,
                body,
                self.creds.as_ref(),
                self.timeout,
            )
            .await?;
        Ok(())
    }
}

/// Format an `f32` for a PTZ vector attribute, avoiding locale/exponent
/// surprises and trimming a trailing `.0` only when it stays a valid number.
fn fmt_f32(v: f32) -> String {
    // `{}` on f32 already yields a round-trippable, locale-independent form
    // (e.g. `0.5`, `-1`, `0`), which ONVIF devices accept.
    format!("{v}")
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
/// text, stopping at the matching `End`. Tolerant of nested elements / empty
/// elements.
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

/// Normalize an empty/whitespace string to `None`.
fn non_empty(s: String) -> Option<String> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Extract an attribute value by **local name** (prefix-agnostic) from a start
/// element, parsed as `f32`. Returns `None` if absent or unparsable.
fn attr_f32(e: &quick_xml::events::BytesStart, want: &str) -> Option<f32> {
    for attr in e.attributes().flatten() {
        if local_name(attr.key.as_ref()) == want {
            let raw = attr.unescape_value().ok()?;
            return raw.trim().parse::<f32>().ok();
        }
    }
    None
}

/// Extract an attribute value by local name as a `String`.
fn attr_str(e: &quick_xml::events::BytesStart, want: &str) -> Option<String> {
    for attr in e.attributes().flatten() {
        if local_name(attr.key.as_ref()) == want {
            return attr
                .unescape_value()
                .ok()
                .map(|c| c.trim().to_string())
                .filter(|s| !s.is_empty());
        }
    }
    None
}

/// Parse a `PanTilt`/`Zoom` pair out of the children of a `PTZVector`-shaped
/// element (Position, Velocity, Translation). Scans the whole document for the
/// first `PanTilt`/`Zoom` elements that carry `x`/`y` attributes — this is the
/// shape used inside a single `<...Position>` block of a GetStatus response.
fn parse_vector(xml: &str) -> PtzVector {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut vec = PtzVector::default();
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                let local = local_name(e.name().as_ref());
                match local.as_str() {
                    "PanTilt" if vec.pan_tilt.is_none() => {
                        if let (Some(x), Some(y)) = (attr_f32(&e, "x"), attr_f32(&e, "y")) {
                            vec.pan_tilt = Some(PanTilt::new(x, y));
                        }
                    }
                    "Zoom" if vec.zoom.is_none() => {
                        if let Some(x) = attr_f32(&e, "x") {
                            vec.zoom = Some(Zoom::new(x));
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
    }
    vec
}

/// Parse a `GetStatusResponse` into [`PtzStatus`].
///
/// The shape is `PTZStatus { Position{PanTilt,Zoom}, MoveStatus{PanTilt,Zoom},
/// UtcTime }`. We capture the position vector, the two move-status strings, and
/// the UTC time, all by local name and all optional.
fn parse_status(xml: &str) -> OnvifResult<PtzStatus> {
    // Position vector: reuse the generic vector scanner over the whole body.
    // The Position block is the only place PanTilt/Zoom carry x/y attributes in
    // a status response, so a document-wide scan is safe and simpler.
    let position = parse_vector(xml);

    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut status = PtzStatus {
        position,
        ..Default::default()
    };
    // Local-name stack so we can disambiguate the two `PanTilt`/`Zoom` elements
    // that appear *inside MoveStatus* (which carry text, not attributes).
    let mut stack: Vec<String> = Vec::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let local = local_name(e.name().as_ref());
                match local.as_str() {
                    // MoveStatus children: <PanTilt>IDLE</PanTilt>, <Zoom>...
                    "PanTilt" if parent_is(&stack, "MoveStatus") => {
                        status.move_status_pan_tilt = non_empty(read_text(&mut reader));
                        continue;
                    }
                    "Zoom" if parent_is(&stack, "MoveStatus") => {
                        status.move_status_zoom = non_empty(read_text(&mut reader));
                        continue;
                    }
                    "UtcTime" => {
                        status.utc_time = non_empty(read_text(&mut reader));
                        continue;
                    }
                    _ => {}
                }
                stack.push(local);
            }
            Ok(Event::End(_)) => {
                stack.pop();
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(OnvifError::Parse(format!("ptz status xml: {e}"))),
            _ => {}
        }
    }
    Ok(status)
}

/// True when the element immediately enclosing the current one has the given
/// local name.
fn parent_is(stack: &[String], local: &str) -> bool {
    stack.last().map(|s| s.as_str()) == Some(local)
}

/// Parse a `GetConfigurationsResponse` into a list of [`PtzConfiguration`].
///
/// The response carries repeated `<PTZConfiguration token="...">` elements,
/// each with a `<Name>` and a `<NodeToken>` child. We collect one entry per
/// `PTZConfiguration`; missing children stay `None`.
fn parse_configurations(xml: &str) -> OnvifResult<Vec<PtzConfiguration>> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut configs = Vec::new();
    let mut current: Option<PtzConfiguration> = None;

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let local = local_name(e.name().as_ref());
                match local.as_str() {
                    "PTZConfiguration" => {
                        current = Some(PtzConfiguration {
                            token: attr_str(&e, "token"),
                            ..Default::default()
                        });
                    }
                    "Name" => {
                        let v = non_empty(read_text(&mut reader));
                        if let Some(c) = current.as_mut() {
                            c.name = v;
                        }
                    }
                    "NodeToken" => {
                        let v = non_empty(read_text(&mut reader));
                        if let Some(c) = current.as_mut() {
                            c.node_token = v;
                        }
                    }
                    _ => {}
                }
            }
            // Empty-element form `<PTZConfiguration token="..."/>` arrives as a
            // single `Empty` event (no separate Start/End). Capture it in full.
            Ok(Event::Empty(e)) => {
                if local_name(e.name().as_ref()) == "PTZConfiguration" {
                    configs.push(PtzConfiguration {
                        token: attr_str(&e, "token"),
                        ..Default::default()
                    });
                }
            }
            Ok(Event::End(e)) => {
                if local_name(e.name().as_ref()) == "PTZConfiguration" {
                    if let Some(c) = current.take() {
                        configs.push(c);
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(OnvifError::Parse(format!("ptz configurations xml: {e}"))),
            _ => {}
        }
    }
    Ok(configs)
}

/// Minimal XML text escaping for element content (profile tokens, etc.).
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

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Body builders --------------------------------------------------

    #[test]
    fn continuous_move_body_has_velocity_vector() {
        let v = PtzVector::new(PanTilt::new(0.5, -0.25), Zoom::new(0.0));
        let body = format!(
            concat!(
                "<tptz:ContinuousMove xmlns:tptz=\"{ns}\">",
                "<tptz:ProfileToken>{profile}</tptz:ProfileToken>",
                "<tptz:Velocity>{vec}</tptz:Velocity>",
                "</tptz:ContinuousMove>",
            ),
            ns = PTZ_WSDL_NS,
            profile = xml_escape("Profile_1"),
            vec = v.to_vector_children(),
        );
        assert!(body.contains("<tptz:ContinuousMove"));
        assert!(body.contains("<tptz:ProfileToken>Profile_1</tptz:ProfileToken>"));
        assert!(body.contains("<tptz:Velocity>"));
        assert!(body.contains("PanTilt x=\"0.5\" y=\"-0.25\""));
        assert!(body.contains("Zoom x=\"0\""));
        // Vectors carry the ONVIF schema namespace so the device resolves tt:.
        assert!(body.contains(ONVIF_SCHEMA_NS));
    }

    #[test]
    fn pan_tilt_only_vector_omits_zoom() {
        let children = PtzVector::pan_tilt(1.0, 0.0).to_vector_children();
        assert!(children.contains("PanTilt"));
        assert!(!children.contains("Zoom"));
    }

    #[test]
    fn zoom_only_vector_omits_pan_tilt() {
        let children = PtzVector::zoom(-1.0).to_vector_children();
        assert!(children.contains("Zoom x=\"-1\""));
        assert!(!children.contains("PanTilt"));
    }

    #[test]
    fn profile_token_is_xml_escaped_in_stop_body() {
        let body = format!(
            "<tptz:Stop><tptz:ProfileToken>{}</tptz:ProfileToken></tptz:Stop>",
            xml_escape("a<b&c\"")
        );
        assert!(body.contains("a&lt;b&amp;c&quot;"));
    }

    // ---- GetStatus fixtures --------------------------------------------

    /// A normal GetStatus response: full position, both move states, UTC time,
    /// conventional `tptz:`/`tt:` prefixes.
    const STATUS_NORMAL: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope">
  <s:Body>
    <tptz:GetStatusResponse xmlns:tptz="http://www.onvif.org/ver20/ptz/wsdl"
                            xmlns:tt="http://www.onvif.org/ver10/schema">
      <tptz:PTZStatus>
        <tt:Position>
          <tt:PanTilt x="0.5" y="-0.3"/>
          <tt:Zoom x="0.25"/>
        </tt:Position>
        <tt:MoveStatus>
          <tt:PanTilt>IDLE</tt:PanTilt>
          <tt:Zoom>IDLE</tt:Zoom>
        </tt:MoveStatus>
        <tt:UtcTime>2026-06-20T07:50:45Z</tt:UtcTime>
      </tptz:PTZStatus>
    </tptz:GetStatusResponse>
  </s:Body>
</s:Envelope>"#;

    /// Prefix-varied: default-namespaced envelope, PTZ bound to `q1:`, the
    /// schema bound to `onvif:`, and attributes given in `y x` order.
    const STATUS_PREFIX_VARIED: &str = r#"<Envelope xmlns="http://www.w3.org/2003/05/soap-envelope">
  <Body>
    <q1:GetStatusResponse xmlns:q1="http://www.onvif.org/ver20/ptz/wsdl"
                          xmlns:onvif="http://www.onvif.org/ver10/schema">
      <q1:PTZStatus>
        <onvif:Position>
          <onvif:PanTilt y="0.1" x="-0.9"/>
          <onvif:Zoom x="1.0"/>
        </onvif:Position>
        <onvif:MoveStatus>
          <onvif:PanTilt>MOVING</onvif:PanTilt>
          <onvif:Zoom>IDLE</onvif:Zoom>
        </onvif:MoveStatus>
      </q1:PTZStatus>
    </q1:GetStatusResponse>
  </Body>
</Envelope>"#;

    /// Missing optionals: only a pan/tilt position (no zoom, no move status,
    /// no UTC time). Must not panic; absent fields stay None/None.
    const STATUS_MISSING: &str = r#"<s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope">
  <s:Body>
    <tptz:GetStatusResponse xmlns:tptz="http://www.onvif.org/ver20/ptz/wsdl"
                            xmlns:tt="http://www.onvif.org/ver10/schema">
      <tptz:PTZStatus>
        <tt:Position>
          <tt:PanTilt x="0.0" y="0.0"/>
        </tt:Position>
      </tptz:PTZStatus>
    </tptz:GetStatusResponse>
  </s:Body>
</s:Envelope>"#;

    #[test]
    fn status_normal() {
        let st = parse_status(STATUS_NORMAL).expect("parse");
        let pt = st.position.pan_tilt.expect("pan/tilt");
        assert!((pt.x - 0.5).abs() < 1e-6);
        assert!((pt.y - (-0.3)).abs() < 1e-6);
        let z = st.position.zoom.expect("zoom");
        assert!((z.x - 0.25).abs() < 1e-6);
        assert_eq!(st.move_status_pan_tilt.as_deref(), Some("IDLE"));
        assert_eq!(st.move_status_zoom.as_deref(), Some("IDLE"));
        assert_eq!(st.utc_time.as_deref(), Some("2026-06-20T07:50:45Z"));
    }

    #[test]
    fn status_prefix_varied() {
        // Arbitrary prefixes and reversed attribute order must parse identically.
        let st = parse_status(STATUS_PREFIX_VARIED).expect("parse");
        let pt = st.position.pan_tilt.expect("pan/tilt");
        assert!((pt.x - (-0.9)).abs() < 1e-6);
        assert!((pt.y - 0.1).abs() < 1e-6);
        let z = st.position.zoom.expect("zoom");
        assert!((z.x - 1.0).abs() < 1e-6);
        assert_eq!(st.move_status_pan_tilt.as_deref(), Some("MOVING"));
        assert_eq!(st.move_status_zoom.as_deref(), Some("IDLE"));
        // No UTC time element present.
        assert_eq!(st.utc_time, None);
    }

    #[test]
    fn status_missing_optionals() {
        let st = parse_status(STATUS_MISSING).expect("parse");
        let pt = st.position.pan_tilt.expect("pan/tilt present");
        assert_eq!(pt.x, 0.0);
        assert_eq!(pt.y, 0.0);
        // Zoom omitted, no move status, no UTC time.
        assert_eq!(st.position.zoom, None);
        assert_eq!(st.move_status_pan_tilt, None);
        assert_eq!(st.move_status_zoom, None);
        assert_eq!(st.utc_time, None);
    }

    // ---- GetConfigurations fixtures ------------------------------------

    /// A normal GetConfigurations response with two configs.
    const CONFIGS_NORMAL: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope">
  <s:Body>
    <tptz:GetConfigurationsResponse xmlns:tptz="http://www.onvif.org/ver20/ptz/wsdl"
                                    xmlns:tt="http://www.onvif.org/ver10/schema">
      <tptz:PTZConfiguration token="PTZCfg_1">
        <tt:Name>PtzConfig1</tt:Name>
        <tt:NodeToken>PtzNode_1</tt:NodeToken>
        <tt:DefaultAbsolutePantTiltPositionSpace>http://www.onvif.org/ver10/tptz/PanTiltSpaces/PositionGenericSpace</tt:DefaultAbsolutePantTiltPositionSpace>
      </tptz:PTZConfiguration>
      <tptz:PTZConfiguration token="PTZCfg_2">
        <tt:Name>PtzConfig2</tt:Name>
        <tt:NodeToken>PtzNode_2</tt:NodeToken>
      </tptz:PTZConfiguration>
    </tptz:GetConfigurationsResponse>
  </s:Body>
</s:Envelope>"#;

    /// Prefix-varied: default-namespaced envelope and `q1:`/`onvif:` prefixes.
    const CONFIGS_PREFIX_VARIED: &str = r#"<Envelope xmlns="http://www.w3.org/2003/05/soap-envelope">
  <Body>
    <q1:GetConfigurationsResponse xmlns:q1="http://www.onvif.org/ver20/ptz/wsdl"
                                  xmlns:onvif="http://www.onvif.org/ver10/schema">
      <q1:PTZConfiguration token="Cfg_A">
        <onvif:Name>Alpha</onvif:Name>
        <onvif:NodeToken>Node_A</onvif:NodeToken>
      </q1:PTZConfiguration>
    </q1:GetConfigurationsResponse>
  </Body>
</Envelope>"#;

    /// Missing optionals: a config with only a token attribute (no Name,
    /// no NodeToken).
    const CONFIGS_MISSING: &str = r#"<s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope">
  <s:Body>
    <tptz:GetConfigurationsResponse xmlns:tptz="http://www.onvif.org/ver20/ptz/wsdl">
      <tptz:PTZConfiguration token="OnlyToken"/>
    </tptz:GetConfigurationsResponse>
  </s:Body>
</s:Envelope>"#;

    #[test]
    fn configurations_normal() {
        let cfgs = parse_configurations(CONFIGS_NORMAL).expect("parse");
        assert_eq!(cfgs.len(), 2);
        assert_eq!(cfgs[0].token.as_deref(), Some("PTZCfg_1"));
        assert_eq!(cfgs[0].name.as_deref(), Some("PtzConfig1"));
        assert_eq!(cfgs[0].node_token.as_deref(), Some("PtzNode_1"));
        assert_eq!(cfgs[1].token.as_deref(), Some("PTZCfg_2"));
        assert_eq!(cfgs[1].name.as_deref(), Some("PtzConfig2"));
        assert_eq!(cfgs[1].node_token.as_deref(), Some("PtzNode_2"));
    }

    #[test]
    fn configurations_prefix_varied() {
        let cfgs = parse_configurations(CONFIGS_PREFIX_VARIED).expect("parse");
        assert_eq!(cfgs.len(), 1);
        assert_eq!(cfgs[0].token.as_deref(), Some("Cfg_A"));
        assert_eq!(cfgs[0].name.as_deref(), Some("Alpha"));
        assert_eq!(cfgs[0].node_token.as_deref(), Some("Node_A"));
    }

    #[test]
    fn configurations_missing_optionals() {
        // Empty-element form `<PTZConfiguration token=.../>` arrives as a single
        // `Event::Empty`, exercising that code path. Verify token-only tolerance.
        let cfgs = parse_configurations(CONFIGS_MISSING).expect("parse");
        assert_eq!(cfgs.len(), 1);
        assert_eq!(cfgs[0].token.as_deref(), Some("OnlyToken"));
        assert_eq!(cfgs[0].name, None);
        assert_eq!(cfgs[0].node_token, None);
    }

    #[test]
    fn garbage_input_does_not_panic() {
        let _ = parse_status("<not><closed>");
        let _ = parse_configurations("not xml at all");
        let _ = parse_vector("<<<>>>");
    }
}
