//! Shared quick-xml event helpers.
//!
//! quick-xml ≥ 0.38 no longer unescapes entity references inside `Text`
//! events; instead the reader emits each `&name;` / `&#N;` as a separate
//! [`quick_xml::events::Event::GeneralRef`] event. Parsers that previously
//! called `BytesText::unescape()` must now stitch text fragments and resolved
//! references back together.

use std::borrow::Cow;

use quick_xml::escape::resolve_predefined_entity;
use quick_xml::events::{BytesRef, BytesText};

/// Decoded content of a `Text` event. Entity references arrive separately as
/// `GeneralRef` events and never appear here.
pub(crate) fn text_content<'a>(t: &'a BytesText<'a>) -> Cow<'a, str> {
    t.xml10_content().unwrap_or_default()
}

/// Resolve a `GeneralRef` event to its replacement text: numeric character
/// references directly, named references via the predefined XML entity set
/// (`amp`, `lt`, `gt`, `quot`, `apos`). Unknown or malformed references
/// resolve to the empty string, matching the leniency of the old
/// `unescape().unwrap_or_default()` call sites.
pub(crate) fn general_ref_content(r: &BytesRef<'_>) -> String {
    if let Ok(Some(ch)) = r.resolve_char_ref() {
        return ch.to_string();
    }
    match r.decode() {
        Ok(name) => resolve_predefined_entity(&name)
            .unwrap_or_default()
            .to_string(),
        Err(_) => String::new(),
    }
}
