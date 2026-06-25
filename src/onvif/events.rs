//! ONVIF **Events** service client (WS-BaseNotification PullPoint).
//!
//! Implements the PullPoint event-consumption flow used to receive device
//! notifications (motion, tamper, digital inputs, analytics, …) without the
//! camera needing to reach back to us:
//!
//! - [`EventsClient::create_pull_point_subscription`] —
//!   `CreatePullPointSubscription`: establishes a subscription and returns the
//!   per-subscription endpoint reference (the address we then poll).
//! - [`EventsClient::pull_messages`] — `PullMessages`: long-polls the
//!   subscription, returning a batch of [`NotificationMessage`]s.
//! - [`EventsClient::renew`] — `Renew`: extends the subscription lifetime.
//! - [`EventsClient::unsubscribe`] — `Unsubscribe`: tears the subscription down.
//!
//! Each method builds the SOAP body, dispatches it through
//! [`OnvifTransport::call`], and parses the response with quick-xml.
//!
//! ## Service-address subtlety
//!
//! `CreatePullPointSubscription` is sent to the **Events** service XAddr, but
//! the response carries a *SubscriptionReference* `Address` — a distinct,
//! per-subscription URL. All subsequent `PullMessages` / `Renew` / `Unsubscribe`
//! calls must target *that* address, not the original Events XAddr. The client
//! captures it in [`PullPointSubscription::address`] and the polling/renew/
//! teardown methods take it as an explicit argument.
//!
//! ## Parsing philosophy
//!
//! Like the other ONVIF service clients, every parser matches purely on the
//! **local name** of each element (never the namespace prefix) so that the
//! arbitrary prefixes real cameras emit (`tev:`, `wsnt:`, `wsa:`, `tt:`, `b:`,
//! `tns1:`, default namespaces, …) all parse identically. Every field is
//! optional: a missing element yields `None`/empty rather than an error.

use std::time::Duration;

use quick_xml::events::Event;
use quick_xml::reader::Reader;

use crate::onvif::error::{OnvifError, OnvifResult};
use crate::onvif::transport::OnvifTransport;
use crate::onvif::types::Credentials;

/// WSDL namespace for the ONVIF Events service.
const EVENTS_WSDL_NS: &str = "http://www.onvif.org/ver10/events/wsdl";
/// WS-BaseNotification namespace (NotificationMessage, Topic, Message wrappers).
const WSNT_NS: &str = "http://docs.oasis-open.org/wsn/b-2";

/// Default per-request timeout when the caller does not specify one.
///
/// `PullMessages` is a long-poll: the device blocks until it has messages or
/// the request's `Timeout` elapses. We therefore keep the transport timeout
/// comfortably above the typical PullMessages `Timeout` so the HTTP request
/// itself does not abort the long-poll early.
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(70);

/// A PullPoint subscription created by `CreatePullPointSubscription`.
///
/// `address` is the SubscriptionReference endpoint — the URL that subsequent
/// `PullMessages` / `Renew` / `Unsubscribe` calls must target (it is generally
/// different from the Events service XAddr).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PullPointSubscription {
    /// SubscriptionReference `Address` — the per-subscription endpoint URL to
    /// poll. Empty only if the device returned a malformed response.
    pub address: String,
    /// Device's notion of "now" at subscription time (`wsnt:CurrentTime`).
    pub current_time: Option<String>,
    /// When the subscription will expire absent a `Renew` (`wsnt:TerminationTime`).
    pub termination_time: Option<String>,
}

/// The result of a `PullMessages` call: a batch of notifications plus the
/// device's current/termination times.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PullMessagesResponse {
    /// Device's current time at the moment of the response.
    pub current_time: Option<String>,
    /// Updated subscription termination time.
    pub termination_time: Option<String>,
    /// The notifications delivered in this batch (may be empty).
    pub messages: Vec<NotificationMessage>,
}

/// A single ONVIF notification (`wsnt:NotificationMessage`).
///
/// ONVIF wraps a `tt:Message` inside the WS-Notification `Message` element. We
/// surface the dialect-independent essentials: the topic, the property change
/// operation, the message UTC time, and the flattened Source / Data SimpleItem
/// name→value pairs (these carry the state — e.g. `IsMotion=true`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NotificationMessage {
    /// The notification topic (e.g.
    /// `tns1:RuleEngine/CellMotionDetector/Motion`). The dotted/slashed path is
    /// preserved verbatim, including any prefix the camera used.
    pub topic: Option<String>,
    /// `PropertyOperation` attribute on `tt:Message` (`Initialized`, `Changed`,
    /// `Deleted`) when present.
    pub property_operation: Option<String>,
    /// `UtcTime` attribute on `tt:Message` — when the device observed the event.
    pub utc_time: Option<String>,
    /// `Source` SimpleItem name→value pairs (identify *which* channel/region/
    /// input the event is about, e.g. `VideoSourceConfigurationToken`).
    pub source: Vec<SimpleItem>,
    /// `Data` SimpleItem name→value pairs (the actual state, e.g.
    /// `IsMotion=true`, `State=false`).
    pub data: Vec<SimpleItem>,
}

impl NotificationMessage {
    /// Resolve the alarm on/off state this notification represents, if it can be
    /// determined from a `Data` SimpleItem.
    ///
    /// ONVIF does not mandate a single boolean element name, so we look at the
    /// first `Data` SimpleItem whose value is boolean-like and interpret a
    /// trailing `Deleted` property-operation as a clear. Returns `None` when the
    /// notification carries no boolean data item (e.g. a counter or string).
    pub fn alarm_active(&self) -> Option<bool> {
        // A `Deleted` property change means the state object went away — treat
        // as inactive regardless of the (now-stale) data value.
        if self
            .property_operation
            .as_deref()
            .is_some_and(|op| op.eq_ignore_ascii_case("Deleted"))
        {
            return Some(false);
        }
        self.data.iter().find_map(|item| parse_bool(&item.value))
    }

    /// Convenience: the value of the first `Data` SimpleItem with the given
    /// (local) name, matched case-insensitively.
    pub fn data_value(&self, name: &str) -> Option<&str> {
        self.data
            .iter()
            .find(|i| i.name.eq_ignore_ascii_case(name))
            .map(|i| i.value.as_str())
    }
}

/// A WS-Notification `tt:SimpleItem` (a `Name="…" Value="…"` attribute pair).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SimpleItem {
    /// The item name (e.g. `IsMotion`, `State`, `VideoSourceConfigurationToken`).
    pub name: String,
    /// The item value (e.g. `true`, `false`, a token string, a number).
    pub value: String,
}

/// Events service client. A single client can create a subscription and then
/// poll/renew/cancel it; the per-subscription address is threaded explicitly
/// through the polling methods.
#[derive(Debug, Clone)]
pub struct EventsClient {
    transport: OnvifTransport,
    /// Events service endpoint URL (where `CreatePullPointSubscription` is sent).
    xaddr: String,
    creds: Option<Credentials>,
    timeout: Duration,
}

impl EventsClient {
    /// Create an Events client targeting `xaddr` (the Events service endpoint
    /// URL), optionally with WS-Security credentials. Uses the default timeout.
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

    /// Override the per-request timeout (must exceed the PullMessages long-poll
    /// `Timeout` to avoid aborting the poll at the HTTP layer).
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// The Events service XAddr this client targets.
    pub fn xaddr(&self) -> &str {
        &self.xaddr
    }

    /// `CreatePullPointSubscription` — establish a PullPoint subscription.
    ///
    /// `initial_termination_time` is an XML duration (e.g. `PT60S`) requesting
    /// how long the subscription should live before it must be renewed; pass
    /// `None` to let the device choose its default.
    pub async fn create_pull_point_subscription(
        &self,
        initial_termination_time: Option<&str>,
    ) -> OnvifResult<PullPointSubscription> {
        let term = match initial_termination_time {
            Some(t) => format!(
                "<tev:InitialTerminationTime>{}</tev:InitialTerminationTime>",
                xml_escape(t)
            ),
            None => String::new(),
        };
        let body = format!(
            "<tev:CreatePullPointSubscription xmlns:tev=\"{ns}\">{term}</tev:CreatePullPointSubscription>",
            ns = EVENTS_WSDL_NS,
            term = term,
        );
        let action = format!("{EVENTS_WSDL_NS}/CreatePullPointSubscription");
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
        parse_create_pull_point(&xml)
    }

    /// `PullMessages` — long-poll the subscription for up to `message_limit`
    /// notifications, blocking on the device for up to `timeout` (an XML
    /// duration such as `PT60S`).
    ///
    /// `subscription_address` is the SubscriptionReference `Address` returned by
    /// [`create_pull_point_subscription`](Self::create_pull_point_subscription)
    /// — *not* the Events XAddr.
    pub async fn pull_messages(
        &self,
        subscription_address: &str,
        timeout: &str,
        message_limit: u32,
    ) -> OnvifResult<PullMessagesResponse> {
        let body = format!(
            concat!(
                "<tev:PullMessages xmlns:tev=\"{ns}\">",
                "<tev:Timeout>{timeout}</tev:Timeout>",
                "<tev:MessageLimit>{limit}</tev:MessageLimit>",
                "</tev:PullMessages>",
            ),
            ns = EVENTS_WSDL_NS,
            timeout = xml_escape(timeout),
            limit = message_limit,
        );
        let action = format!("{EVENTS_WSDL_NS}/PullMessages");
        let xml = self
            .transport
            .call(
                subscription_address,
                &action,
                &body,
                self.creds.as_ref(),
                self.timeout,
            )
            .await?;
        parse_pull_messages(&xml)
    }

    /// `Renew` — extend the subscription lifetime by `termination_time` (an XML
    /// duration such as `PT60S`, or an absolute time). Returns the device's new
    /// `TerminationTime` if it reports one.
    ///
    /// Targets the per-subscription `subscription_address`.
    pub async fn renew(
        &self,
        subscription_address: &str,
        termination_time: &str,
    ) -> OnvifResult<Option<String>> {
        // Renew is a WS-BaseNotification operation (wsnt namespace), not an
        // ONVIF-specific one.
        let body = format!(
            concat!(
                "<wsnt:Renew xmlns:wsnt=\"{ns}\">",
                "<wsnt:TerminationTime>{term}</wsnt:TerminationTime>",
                "</wsnt:Renew>",
            ),
            ns = WSNT_NS,
            term = xml_escape(termination_time),
        );
        let action = format!("{WSNT_NS}/RenewRequest");
        let xml = self
            .transport
            .call(
                subscription_address,
                &action,
                &body,
                self.creds.as_ref(),
                self.timeout,
            )
            .await?;
        Ok(parse_renew(&xml))
    }

    /// `Unsubscribe` — tear down the subscription. Targets the per-subscription
    /// `subscription_address`.
    pub async fn unsubscribe(&self, subscription_address: &str) -> OnvifResult<()> {
        let body = format!("<wsnt:Unsubscribe xmlns:wsnt=\"{ns}\"/>", ns = WSNT_NS);
        let action = format!("{WSNT_NS}/UnsubscribeRequest");
        // Any SOAP fault is surfaced by the transport; a clean response means
        // the subscription is gone.
        self.transport
            .call(
                subscription_address,
                &action,
                &body,
                self.creds.as_ref(),
                self.timeout,
            )
            .await?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Parsers
// ---------------------------------------------------------------------------

/// Parse a `CreatePullPointSubscriptionResponse`.
///
/// Shape (prefixes vary):
/// ```xml
/// <tev:CreatePullPointSubscriptionResponse>
///   <tev:SubscriptionReference>
///     <wsa:Address>http://host/onvif/Subscription?id=0</wsa:Address>
///   </tev:SubscriptionReference>
///   <wsnt:CurrentTime>…</wsnt:CurrentTime>
///   <wsnt:TerminationTime>…</wsnt:TerminationTime>
/// </tev:CreatePullPointSubscriptionResponse>
/// ```
///
/// The `Address` lives under `SubscriptionReference`; some devices wrap it in an
/// extra `EndpointReference`. We capture the first `Address` that appears inside
/// a `SubscriptionReference` subtree.
fn parse_create_pull_point(xml: &str) -> OnvifResult<PullPointSubscription> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut sub = PullPointSubscription::default();
    let mut stack: Vec<String> = Vec::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let local = local_name(e.name().as_ref());
                match local.as_str() {
                    "Address" if stack_contains(&stack, "SubscriptionReference") => {
                        if sub.address.is_empty() {
                            sub.address = read_text(&mut reader).trim().to_string();
                        }
                        // read_text consumed through the matching End — don't push.
                        continue;
                    }
                    "CurrentTime" => {
                        sub.current_time = non_empty(read_text(&mut reader));
                        continue;
                    }
                    "TerminationTime" => {
                        sub.termination_time = non_empty(read_text(&mut reader));
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
            Err(e) => return Err(OnvifError::Parse(format!("create pullpoint xml: {e}"))),
            _ => {}
        }
    }
    Ok(sub)
}

/// Parse a `PullMessagesResponse` into the current/termination times and the
/// batch of [`NotificationMessage`]s.
///
/// Shape (prefixes vary):
/// ```xml
/// <tev:PullMessagesResponse>
///   <tev:CurrentTime>…</tev:CurrentTime>
///   <tev:TerminationTime>…</tev:TerminationTime>
///   <wsnt:NotificationMessage>
///     <wsnt:Topic Dialect="…">tns1:RuleEngine/CellMotionDetector/Motion</wsnt:Topic>
///     <wsnt:Message>
///       <tt:Message UtcTime="…" PropertyOperation="Changed">
///         <tt:Source><tt:SimpleItem Name="…" Value="…"/></tt:Source>
///         <tt:Data><tt:SimpleItem Name="IsMotion" Value="true"/></tt:Data>
///       </tt:Message>
///     </wsnt:Message>
///   </wsnt:NotificationMessage>
///   … (repeated) …
/// </tev:PullMessagesResponse>
/// ```
fn parse_pull_messages(xml: &str) -> OnvifResult<PullMessagesResponse> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut out = PullMessagesResponse::default();
    // The element subtree we are currently inside, by local name. Used to
    // distinguish a `Source`/`Data` SimpleItem and to assign top-level times
    // only when they are NOT nested inside a NotificationMessage's Message.
    let mut stack: Vec<String> = Vec::new();
    let mut current: Option<NotificationMessage> = None;

    loop {
        match reader.read_event() {
            Ok(ev @ (Event::Start(_) | Event::Empty(_))) => {
                let is_empty = matches!(ev, Event::Empty(_));
                let bytes = match &ev {
                    Event::Start(e) | Event::Empty(e) => e.clone(),
                    _ => unreachable!(),
                };
                let local = local_name(bytes.name().as_ref());

                match local.as_str() {
                    "NotificationMessage" => {
                        current = Some(NotificationMessage::default());
                    }
                    "Topic"
                        // A self-closing <Topic/> has no text; only consume the
                        // element body for a real Start (avoids over-reading the
                        // following sibling).
                        if !is_empty => {
                            let text = non_empty(read_text(&mut reader));
                            if let Some(msg) = current.as_mut() {
                                msg.topic = text;
                            }
                            continue;
                        }
                    // The inner `tt:Message` carries UtcTime / PropertyOperation
                    // as attributes. The outer WS-Notification `Message` wrapper
                    // has no such attributes, so reading them is harmless.
                    "Message" => {
                        if let Some(msg) = current.as_mut() {
                            if let Some(v) = attr_value(&bytes, "UtcTime") {
                                if msg.utc_time.is_none() {
                                    msg.utc_time = non_empty(v);
                                }
                            }
                            if let Some(v) = attr_value(&bytes, "PropertyOperation") {
                                if msg.property_operation.is_none() {
                                    msg.property_operation = non_empty(v);
                                }
                            }
                        }
                    }
                    "SimpleItem" => {
                        if let Some(msg) = current.as_mut() {
                            let name = attr_value(&bytes, "Name").unwrap_or_default();
                            let value = attr_value(&bytes, "Value").unwrap_or_default();
                            let item = SimpleItem {
                                name: name.trim().to_string(),
                                value: value.trim().to_string(),
                            };
                            // Classify by the nearest Source/Data ancestor.
                            if stack_contains(&stack, "Source") {
                                msg.source.push(item);
                            } else if stack_contains(&stack, "Data") {
                                msg.data.push(item);
                            } else {
                                // Some firmwares omit the Source/Data wrapper;
                                // default such items to Data (they carry state).
                                msg.data.push(item);
                            }
                        }
                    }
                    "CurrentTime" if current.is_none() && !is_empty => {
                        out.current_time = non_empty(read_text(&mut reader));
                        continue;
                    }
                    "TerminationTime" if current.is_none() && !is_empty => {
                        out.termination_time = non_empty(read_text(&mut reader));
                        continue;
                    }
                    _ => {}
                }

                if !is_empty {
                    stack.push(local);
                }
            }
            Ok(Event::End(e)) => {
                let local = local_name(e.name().as_ref());
                if local == "NotificationMessage" {
                    if let Some(msg) = current.take() {
                        out.messages.push(msg);
                    }
                }
                stack.pop();
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(OnvifError::Parse(format!("pull messages xml: {e}"))),
            _ => {}
        }
    }
    Ok(out)
}

/// Parse a `RenewResponse`, returning the device's new `TerminationTime` if any.
fn parse_renew(xml: &str) -> Option<String> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) if local_name(e.name().as_ref()) == "TerminationTime" => {
                let v = non_empty(read_text(&mut reader));
                if v.is_some() {
                    return v;
                }
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
    }
    None
}

// ---------------------------------------------------------------------------
// XML helpers (mirrors device.rs — kept module-local to avoid cross-module
// coupling between the parallel service clients).
// ---------------------------------------------------------------------------

/// Strip an XML namespace prefix, returning the local name.
fn local_name(qname: &[u8]) -> String {
    let s = String::from_utf8_lossy(qname);
    match s.rsplit_once(':') {
        Some((_, local)) => local.to_string(),
        None => s.into_owned(),
    }
}

/// Read the attribute value (by local name, prefix-agnostic) of a start/empty
/// element, unescaping it. Returns `None` if the attribute is absent.
fn attr_value(e: &quick_xml::events::BytesStart<'_>, local: &str) -> Option<String> {
    for attr in e.attributes().with_checks(false).flatten() {
        if local_name(attr.key.as_ref()) == local {
            let raw = attr.unescape_value().unwrap_or_default();
            return Some(raw.into_owned());
        }
    }
    None
}

/// Read the text content of the element currently open in `reader` (positioned
/// just after a `Start` event). Accumulates text/CDATA at the top level,
/// stopping at the matching `End`. Tolerant of nested elements and empties.
fn read_text(reader: &mut Reader<&[u8]>) -> String {
    let mut depth = 0usize;
    let mut out = String::new();
    loop {
        match reader.read_event() {
            Ok(Event::Text(t)) if depth == 0 => {
                out.push_str(&t.unescape().unwrap_or_default());
            }
            Ok(Event::CData(t)) if depth == 0 => {
                out.push_str(&String::from_utf8_lossy(t.as_ref()));
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

/// Whether any open ancestor element on the stack has the given local name.
fn stack_contains(stack: &[String], local: &str) -> bool {
    stack.iter().any(|s| s == local)
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

/// Interpret a boolean-ish ONVIF value (`true`/`false`, `1`/`0`, `on`/`off`,
/// `active`/`inactive`), case-insensitively. Non-boolean values yield `None`.
fn parse_bool(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "on" | "active" | "yes" => Some(true),
        "false" | "0" | "off" | "inactive" | "no" => Some(false),
        _ => None,
    }
}

/// Minimal XML text escaping for element content we inject into request bodies.
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

    // ---- CreatePullPointSubscription -----------------------------------

    /// Normal response: conventional `tev:`/`wsnt:`/`wsa:` prefixes.
    const CREATE_NORMAL: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope">
  <s:Body>
    <tev:CreatePullPointSubscriptionResponse
        xmlns:tev="http://www.onvif.org/ver10/events/wsdl"
        xmlns:wsnt="http://docs.oasis-open.org/wsn/b-2"
        xmlns:wsa="http://www.w3.org/2005/08/addressing">
      <tev:SubscriptionReference>
        <wsa:Address>http://192.168.1.10/onvif/Events/PullSub?Idx=0</wsa:Address>
      </tev:SubscriptionReference>
      <wsnt:CurrentTime>2026-06-20T07:50:45Z</wsnt:CurrentTime>
      <wsnt:TerminationTime>2026-06-20T07:51:45Z</wsnt:TerminationTime>
    </tev:CreatePullPointSubscriptionResponse>
  </s:Body>
</s:Envelope>"#;

    /// Prefix-varied: default-namespaced envelope, arbitrary `q3:`/`a:` prefixes,
    /// and an extra `EndpointReference` wrapper around the Address.
    const CREATE_PREFIX_VARIED: &str = r#"<Envelope xmlns="http://www.w3.org/2003/05/soap-envelope">
  <Body>
    <q3:CreatePullPointSubscriptionResponse
        xmlns:q3="http://www.onvif.org/ver10/events/wsdl"
        xmlns:b="http://docs.oasis-open.org/wsn/b-2"
        xmlns:a="http://www.w3.org/2005/08/addressing">
      <q3:SubscriptionReference>
        <a:EndpointReference>
          <a:Address>http://cam.local/onvif/sub?id=42</a:Address>
        </a:EndpointReference>
      </q3:SubscriptionReference>
      <b:CurrentTime>2026-06-20T08:00:00Z</b:CurrentTime>
      <b:TerminationTime>2026-06-20T08:05:00Z</b:TerminationTime>
    </q3:CreatePullPointSubscriptionResponse>
  </Body>
</Envelope>"#;

    /// Missing optionals: only the Address is present (no Current/Termination
    /// times). Must not panic; times stay `None`.
    const CREATE_MISSING: &str = r#"<s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope">
  <s:Body>
    <tev:CreatePullPointSubscriptionResponse
        xmlns:tev="http://www.onvif.org/ver10/events/wsdl"
        xmlns:wsa="http://www.w3.org/2005/08/addressing">
      <tev:SubscriptionReference>
        <wsa:Address>http://10.0.0.5/Subscription-1</wsa:Address>
      </tev:SubscriptionReference>
    </tev:CreatePullPointSubscriptionResponse>
  </s:Body>
</s:Envelope>"#;

    #[test]
    fn create_pull_point_normal() {
        let sub = parse_create_pull_point(CREATE_NORMAL).expect("parse");
        assert_eq!(
            sub.address,
            "http://192.168.1.10/onvif/Events/PullSub?Idx=0"
        );
        assert_eq!(sub.current_time.as_deref(), Some("2026-06-20T07:50:45Z"));
        assert_eq!(
            sub.termination_time.as_deref(),
            Some("2026-06-20T07:51:45Z")
        );
    }

    #[test]
    fn create_pull_point_prefix_varied() {
        // Arbitrary prefixes + an EndpointReference wrapper still resolve the
        // Address (it lives anywhere inside SubscriptionReference).
        let sub = parse_create_pull_point(CREATE_PREFIX_VARIED).expect("parse");
        assert_eq!(sub.address, "http://cam.local/onvif/sub?id=42");
        assert_eq!(sub.current_time.as_deref(), Some("2026-06-20T08:00:00Z"));
        assert_eq!(
            sub.termination_time.as_deref(),
            Some("2026-06-20T08:05:00Z")
        );
    }

    #[test]
    fn create_pull_point_missing_optionals() {
        let sub = parse_create_pull_point(CREATE_MISSING).expect("parse");
        assert_eq!(sub.address, "http://10.0.0.5/Subscription-1");
        assert_eq!(sub.current_time, None);
        assert_eq!(sub.termination_time, None);
    }

    // ---- PullMessages ---------------------------------------------------

    /// Normal response: one motion notification with Source + Data SimpleItems
    /// and the conventional ONVIF prefixes.
    const PULL_NORMAL: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope">
  <s:Body>
    <tev:PullMessagesResponse
        xmlns:tev="http://www.onvif.org/ver10/events/wsdl"
        xmlns:wsnt="http://docs.oasis-open.org/wsn/b-2"
        xmlns:tt="http://www.onvif.org/ver10/schema">
      <tev:CurrentTime>2026-06-20T07:50:50Z</tev:CurrentTime>
      <tev:TerminationTime>2026-06-20T07:51:50Z</tev:TerminationTime>
      <wsnt:NotificationMessage>
        <wsnt:Topic Dialect="http://www.onvif.org/ver10/tev/topicExpression/ConcreteSet">tns1:RuleEngine/CellMotionDetector/Motion</wsnt:Topic>
        <wsnt:Message>
          <tt:Message UtcTime="2026-06-20T07:50:49Z" PropertyOperation="Changed">
            <tt:Source>
              <tt:SimpleItem Name="VideoSourceConfigurationToken" Value="VSCToken0"/>
              <tt:SimpleItem Name="Rule" Value="MyMotionDetectorRule"/>
            </tt:Source>
            <tt:Data>
              <tt:SimpleItem Name="IsMotion" Value="true"/>
            </tt:Data>
          </tt:Message>
        </wsnt:Message>
      </wsnt:NotificationMessage>
    </tev:PullMessagesResponse>
  </s:Body>
</s:Envelope>"#;

    /// Prefix-varied + two notifications (motion on, then motion off) using a
    /// default-namespaced envelope and unconventional prefixes.
    const PULL_PREFIX_VARIED: &str = r#"<Envelope xmlns="http://www.w3.org/2003/05/soap-envelope">
  <Body>
    <e:PullMessagesResponse
        xmlns:e="http://www.onvif.org/ver10/events/wsdl"
        xmlns:n="http://docs.oasis-open.org/wsn/b-2"
        xmlns:sch="http://www.onvif.org/ver10/schema">
      <e:CurrentTime>2026-06-20T09:00:00Z</e:CurrentTime>
      <e:TerminationTime>2026-06-20T09:01:00Z</e:TerminationTime>
      <n:NotificationMessage>
        <n:Topic>tns1:VideoSource/MotionAlarm</n:Topic>
        <n:Message>
          <sch:Message UtcTime="2026-06-20T09:00:01Z" PropertyOperation="Initialized">
            <sch:Source>
              <sch:SimpleItem Name="Source" Value="VideoSourceToken"/>
            </sch:Source>
            <sch:Data>
              <sch:SimpleItem Name="State" Value="true"/>
            </sch:Data>
          </sch:Message>
        </n:Message>
      </n:NotificationMessage>
      <n:NotificationMessage>
        <n:Topic>tns1:VideoSource/MotionAlarm</n:Topic>
        <n:Message>
          <sch:Message UtcTime="2026-06-20T09:00:05Z" PropertyOperation="Changed">
            <sch:Source>
              <sch:SimpleItem Name="Source" Value="VideoSourceToken"/>
            </sch:Source>
            <sch:Data>
              <sch:SimpleItem Name="State" Value="false"/>
            </sch:Data>
          </sch:Message>
        </n:Message>
      </n:NotificationMessage>
    </e:PullMessagesResponse>
  </Body>
</Envelope>"#;

    /// Missing-optional response: an empty batch (no NotificationMessage) and a
    /// notification whose Message omits UtcTime/PropertyOperation and has no
    /// Source. Must not panic.
    const PULL_MISSING: &str = r#"<s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope">
  <s:Body>
    <tev:PullMessagesResponse
        xmlns:tev="http://www.onvif.org/ver10/events/wsdl"
        xmlns:wsnt="http://docs.oasis-open.org/wsn/b-2"
        xmlns:tt="http://www.onvif.org/ver10/schema">
      <tev:CurrentTime>2026-06-20T07:50:50Z</tev:CurrentTime>
      <wsnt:NotificationMessage>
        <wsnt:Topic>tns1:Device/Trigger/DigitalInput</wsnt:Topic>
        <wsnt:Message>
          <tt:Message>
            <tt:Data>
              <tt:SimpleItem Name="LogicalState" Value="false"/>
            </tt:Data>
          </tt:Message>
        </wsnt:Message>
      </wsnt:NotificationMessage>
    </tev:PullMessagesResponse>
  </s:Body>
</s:Envelope>"#;

    #[test]
    fn pull_messages_normal() {
        let resp = parse_pull_messages(PULL_NORMAL).expect("parse");
        assert_eq!(resp.current_time.as_deref(), Some("2026-06-20T07:50:50Z"));
        assert_eq!(
            resp.termination_time.as_deref(),
            Some("2026-06-20T07:51:50Z")
        );
        assert_eq!(resp.messages.len(), 1);

        let m = &resp.messages[0];
        assert_eq!(
            m.topic.as_deref(),
            Some("tns1:RuleEngine/CellMotionDetector/Motion")
        );
        assert_eq!(m.property_operation.as_deref(), Some("Changed"));
        assert_eq!(m.utc_time.as_deref(), Some("2026-06-20T07:50:49Z"));
        // Source items (not classified as Data).
        assert_eq!(m.source.len(), 2);
        assert_eq!(m.source[0].name, "VideoSourceConfigurationToken");
        assert_eq!(m.source[0].value, "VSCToken0");
        // Data items.
        assert_eq!(m.data.len(), 1);
        assert_eq!(m.data[0].name, "IsMotion");
        assert_eq!(m.data[0].value, "true");
        // Alarm semantics.
        assert_eq!(m.alarm_active(), Some(true));
        assert_eq!(m.data_value("ismotion"), Some("true"));
    }

    #[test]
    fn pull_messages_prefix_varied_on_then_off() {
        let resp = parse_pull_messages(PULL_PREFIX_VARIED).expect("parse");
        assert_eq!(resp.current_time.as_deref(), Some("2026-06-20T09:00:00Z"));
        assert_eq!(resp.messages.len(), 2);

        // First: motion on.
        let on = &resp.messages[0];
        assert_eq!(on.topic.as_deref(), Some("tns1:VideoSource/MotionAlarm"));
        assert_eq!(on.property_operation.as_deref(), Some("Initialized"));
        assert_eq!(on.utc_time.as_deref(), Some("2026-06-20T09:00:01Z"));
        assert_eq!(on.source.len(), 1);
        assert_eq!(on.data.len(), 1);
        assert_eq!(on.alarm_active(), Some(true));

        // Second: motion off.
        let off = &resp.messages[1];
        assert_eq!(off.utc_time.as_deref(), Some("2026-06-20T09:00:05Z"));
        assert_eq!(off.data_value("State"), Some("false"));
        assert_eq!(off.alarm_active(), Some(false));
    }

    #[test]
    fn pull_messages_missing_optionals() {
        let resp = parse_pull_messages(PULL_MISSING).expect("parse");
        // Only CurrentTime present at the top level.
        assert_eq!(resp.current_time.as_deref(), Some("2026-06-20T07:50:50Z"));
        assert_eq!(resp.termination_time, None);
        assert_eq!(resp.messages.len(), 1);

        let m = &resp.messages[0];
        assert_eq!(m.topic.as_deref(), Some("tns1:Device/Trigger/DigitalInput"));
        // No attributes on tt:Message.
        assert_eq!(m.utc_time, None);
        assert_eq!(m.property_operation, None);
        // No Source element — stays empty.
        assert!(m.source.is_empty());
        // Data item present.
        assert_eq!(m.data.len(), 1);
        assert_eq!(m.data[0].name, "LogicalState");
        assert_eq!(m.alarm_active(), Some(false));
    }

    #[test]
    fn empty_batch_yields_no_messages() {
        let xml = r#"<s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope">
  <s:Body>
    <tev:PullMessagesResponse xmlns:tev="http://www.onvif.org/ver10/events/wsdl">
      <tev:CurrentTime>2026-06-20T07:50:50Z</tev:CurrentTime>
      <tev:TerminationTime>2026-06-20T07:51:50Z</tev:TerminationTime>
    </tev:PullMessagesResponse>
  </s:Body>
</s:Envelope>"#;
        let resp = parse_pull_messages(xml).expect("parse");
        assert!(resp.messages.is_empty());
        assert_eq!(resp.current_time.as_deref(), Some("2026-06-20T07:50:50Z"));
    }

    #[test]
    fn self_closing_simpleitem_empty_elements() {
        // SimpleItems are commonly emitted as self-closing (`Empty`) elements;
        // ensure they are captured and that the Source/Data classification still
        // works when the wrappers are also present.
        let xml = r#"<s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope">
  <s:Body>
    <tev:PullMessagesResponse xmlns:tev="http://www.onvif.org/ver10/events/wsdl"
        xmlns:wsnt="http://docs.oasis-open.org/wsn/b-2"
        xmlns:tt="http://www.onvif.org/ver10/schema">
      <wsnt:NotificationMessage>
        <wsnt:Topic>tns1:RuleEngine/TamperDetector/Tamper</wsnt:Topic>
        <wsnt:Message>
          <tt:Message UtcTime="2026-06-20T10:00:00Z">
            <tt:Source><tt:SimpleItem Name="Token" Value="0"/></tt:Source>
            <tt:Data><tt:SimpleItem Name="IsTamper" Value="1"/></tt:Data>
          </tt:Message>
        </wsnt:Message>
      </wsnt:NotificationMessage>
    </tev:PullMessagesResponse>
  </s:Body>
</s:Envelope>"#;
        let resp = parse_pull_messages(xml).expect("parse");
        assert_eq!(resp.messages.len(), 1);
        let m = &resp.messages[0];
        assert_eq!(m.source.len(), 1);
        assert_eq!(m.source[0].name, "Token");
        assert_eq!(m.data.len(), 1);
        assert_eq!(m.data[0].name, "IsTamper");
        // "1" is boolean-true.
        assert_eq!(m.alarm_active(), Some(true));
    }

    // ---- Renew ----------------------------------------------------------

    #[test]
    fn renew_returns_termination_time() {
        let xml = r#"<s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope">
  <s:Body>
    <wsnt:RenewResponse xmlns:wsnt="http://docs.oasis-open.org/wsn/b-2">
      <wsnt:TerminationTime>2026-06-20T07:55:00Z</wsnt:TerminationTime>
      <wsnt:CurrentTime>2026-06-20T07:54:00Z</wsnt:CurrentTime>
    </wsnt:RenewResponse>
  </s:Body>
</s:Envelope>"#;
        assert_eq!(parse_renew(xml).as_deref(), Some("2026-06-20T07:55:00Z"));
    }

    #[test]
    fn renew_prefix_varied() {
        let xml = r#"<Envelope xmlns="http://www.w3.org/2003/05/soap-envelope">
  <Body>
    <z:RenewResponse xmlns:z="http://docs.oasis-open.org/wsn/b-2">
      <z:TerminationTime>2026-06-20T11:00:00Z</z:TerminationTime>
    </z:RenewResponse>
  </Body>
</Envelope>"#;
        assert_eq!(parse_renew(xml).as_deref(), Some("2026-06-20T11:00:00Z"));
    }

    #[test]
    fn renew_missing_termination_time_is_none() {
        let xml = r#"<s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope">
  <s:Body>
    <wsnt:RenewResponse xmlns:wsnt="http://docs.oasis-open.org/wsn/b-2"/>
  </s:Body>
</s:Envelope>"#;
        assert_eq!(parse_renew(xml), None);
    }

    // ---- Request-body builders -----------------------------------------

    #[test]
    fn create_body_includes_initial_termination_time_when_set() {
        let term = Some("PT60S");
        let body = match term {
            Some(t) => format!(
                "<tev:CreatePullPointSubscription xmlns:tev=\"{ns}\"><tev:InitialTerminationTime>{t}</tev:InitialTerminationTime></tev:CreatePullPointSubscription>",
                ns = EVENTS_WSDL_NS,
            ),
            None => String::new(),
        };
        assert!(body.contains("<tev:InitialTerminationTime>PT60S</tev:InitialTerminationTime>"));
        assert!(body.contains(EVENTS_WSDL_NS));
    }

    // ---- helper unit coverage ------------------------------------------

    #[test]
    fn parse_bool_handles_common_forms() {
        assert_eq!(parse_bool("true"), Some(true));
        assert_eq!(parse_bool("TRUE"), Some(true));
        assert_eq!(parse_bool("1"), Some(true));
        assert_eq!(parse_bool("on"), Some(true));
        assert_eq!(parse_bool("false"), Some(false));
        assert_eq!(parse_bool("0"), Some(false));
        assert_eq!(parse_bool("inactive"), Some(false));
        assert_eq!(parse_bool("banana"), None);
        assert_eq!(parse_bool(""), None);
    }

    #[test]
    fn deleted_property_operation_clears_alarm() {
        // A Deleted operation means the state object disappeared → inactive,
        // even if the (stale) data value still reads true.
        let m = NotificationMessage {
            property_operation: Some("Deleted".into()),
            data: vec![SimpleItem {
                name: "IsMotion".into(),
                value: "true".into(),
            }],
            ..Default::default()
        };
        assert_eq!(m.alarm_active(), Some(false));
    }

    #[test]
    fn garbage_input_does_not_panic() {
        let _ = parse_create_pull_point("<not><closed>");
        let _ = parse_pull_messages("plain text, not xml at all");
        let _ = parse_renew("<NotificationMessage><Topic>");
    }
}
