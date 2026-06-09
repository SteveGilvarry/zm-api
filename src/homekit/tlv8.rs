//! TLV8 encoding/decoding for the HomeKit Accessory Protocol (HAP).
//!
//! HAP encodes pairing payloads (and several characteristic values) as a
//! sequence of type-length-value items, defined in the HAP specification
//! ("HomeKit Accessory Protocol Specification", section 14.1, "TLV8").
//!
//! Wire format rules implemented here:
//! - Each item is `[type: u8][len: u8][value: len bytes]`.
//! - A value longer than 255 bytes is split into multiple consecutive items
//!   that share the same `type`; on decode, fragments of the same type that
//!   appear back-to-back are concatenated. A fragment of exactly 255 bytes
//!   signals "more of this type follows".
//! - A zero-length item of type [`TlvType::Separator`] delimits entries when a
//!   list of TLV structures is transmitted.
//!
//! We intentionally keep this dependency-free and fully unit-tested: the
//! fragmentation boundary at 255 bytes is the classic source of HAP interop
//! bugs.

/// HAP TLV item types used during pairing (HAP spec table 5-6).
///
/// Only the values we need are enumerated; arbitrary types are still
/// representable via the raw `u8` API.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TlvType {
    Method = 0x00,
    Identifier = 0x01,
    Salt = 0x02,
    PublicKey = 0x03,
    Proof = 0x04,
    EncryptedData = 0x05,
    State = 0x06,
    Error = 0x07,
    RetryDelay = 0x08,
    Certificate = 0x09,
    Signature = 0x0A,
    Permissions = 0x0B,
    FragmentData = 0x0C,
    FragmentLast = 0x0D,
    Flags = 0x13,
    Separator = 0xFF,
}

impl From<TlvType> for u8 {
    fn from(t: TlvType) -> u8 {
        t as u8
    }
}

/// HAP pairing error codes returned in a [`TlvType::Error`] item (HAP table 5-7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TlvError {
    Unknown = 0x01,
    Authentication = 0x02,
    Backoff = 0x03,
    MaxPeers = 0x04,
    MaxTries = 0x05,
    Unavailable = 0x06,
    Busy = 0x07,
}

/// Maximum bytes carried by a single TLV fragment before it must be split.
const MAX_FRAGMENT: usize = 255;

/// Builder for a TLV8 byte stream.
#[derive(Debug, Default, Clone)]
pub struct TlvWriter {
    buf: Vec<u8>,
}

impl TlvWriter {
    pub fn new() -> Self {
        Self { buf: Vec::new() }
    }

    /// Append a typed item, fragmenting values longer than 255 bytes across
    /// multiple items of the same type (per the HAP spec).
    pub fn push(&mut self, ty: impl Into<u8>, value: &[u8]) -> &mut Self {
        let ty = ty.into();
        if value.is_empty() {
            self.buf.push(ty);
            self.buf.push(0);
            return self;
        }
        for chunk in value.chunks(MAX_FRAGMENT) {
            self.buf.push(ty);
            self.buf.push(chunk.len() as u8);
            self.buf.extend_from_slice(chunk);
        }
        self
    }

    /// Append a single-byte value (state, error, method, flags…).
    pub fn push_u8(&mut self, ty: impl Into<u8>, value: u8) -> &mut Self {
        self.push(ty, &[value])
    }

    /// Append a zero-length [`TlvType::Separator`] marking the end of a list entry.
    pub fn push_separator(&mut self) -> &mut Self {
        self.push(TlvType::Separator, &[])
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.buf
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.buf
    }
}

/// A decoded TLV8 stream: an ordered list of `(type, concatenated-value)` items.
///
/// Adjacent fragments of the same type are merged during parsing, so callers
/// see one logical value per type occurrence.
#[derive(Debug, Default, Clone)]
pub struct TlvReader {
    items: Vec<(u8, Vec<u8>)>,
}

impl TlvReader {
    /// Parse a TLV8 byte stream, merging consecutive same-type fragments.
    ///
    /// Returns `None` if the stream is truncated (a declared length runs past
    /// the end of the buffer).
    pub fn parse(bytes: &[u8]) -> Option<Self> {
        let mut items: Vec<(u8, Vec<u8>)> = Vec::new();
        let mut i = 0;
        // Track the previous item so a 255-byte run can be merged with the
        // fragment that follows it.
        let mut prev_was_full_fragment = false;
        let mut prev_type: Option<u8> = None;

        while i < bytes.len() {
            let ty = bytes[i];
            let len = *bytes.get(i + 1)? as usize;
            let start = i + 2;
            let end = start.checked_add(len)?;
            if end > bytes.len() {
                return None;
            }
            let value = &bytes[start..end];

            if prev_was_full_fragment && prev_type == Some(ty) {
                // Continuation of the previous fragmented value.
                if let Some(last) = items.last_mut() {
                    last.1.extend_from_slice(value);
                }
            } else {
                items.push((ty, value.to_vec()));
            }

            prev_was_full_fragment = len == MAX_FRAGMENT;
            prev_type = Some(ty);
            i = end;
        }

        Some(Self { items })
    }

    /// First value matching `ty`, if present.
    pub fn get(&self, ty: impl Into<u8>) -> Option<&[u8]> {
        let ty = ty.into();
        self.items
            .iter()
            .find(|(t, _)| *t == ty)
            .map(|(_, v)| v.as_slice())
    }

    /// First single-byte value matching `ty` (e.g. State, Error).
    pub fn get_u8(&self, ty: impl Into<u8>) -> Option<u8> {
        self.get(ty).and_then(|v| v.first().copied())
    }

    /// All `(type, value)` items in wire order (used to split list entries on
    /// [`TlvType::Separator`]).
    pub fn items(&self) -> &[(u8, Vec<u8>)] {
        &self.items
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_small_items() {
        let mut w = TlvWriter::new();
        w.push_u8(TlvType::State, 3)
            .push(TlvType::Identifier, b"controller-id")
            .push_u8(TlvType::Error, TlvError::Authentication as u8);
        let bytes = w.into_bytes();

        let r = TlvReader::parse(&bytes).expect("parse");
        assert_eq!(r.get_u8(TlvType::State), Some(3));
        assert_eq!(r.get(TlvType::Identifier), Some(&b"controller-id"[..]));
        assert_eq!(
            r.get_u8(TlvType::Error),
            Some(TlvError::Authentication as u8)
        );
    }

    #[test]
    fn empty_value_encodes_as_zero_length() {
        let mut w = TlvWriter::new();
        w.push(TlvType::Separator, &[]);
        assert_eq!(w.as_bytes(), &[0xFF, 0x00]);

        let r = TlvReader::parse(&[0xFF, 0x00]).unwrap();
        assert_eq!(r.get(TlvType::Separator), Some(&[][..]));
    }

    #[test]
    fn fragments_values_over_255_bytes() {
        // A 600-byte public key must split into 255 + 255 + 90.
        let value: Vec<u8> = (0..600u32).map(|n| (n % 251) as u8).collect();
        let mut w = TlvWriter::new();
        w.push(TlvType::PublicKey, &value);
        let bytes = w.into_bytes();

        // Three items, all of type 0x03, lengths 255/255/90.
        assert_eq!(bytes[0], 0x03);
        assert_eq!(bytes[1], 255);
        assert_eq!(bytes[2 + 255], 0x03);
        assert_eq!(bytes[2 + 255 + 1], 255);
        assert_eq!(bytes[2 + 255 + 2 + 255], 0x03);
        assert_eq!(bytes[2 + 255 + 2 + 255 + 1], 90);

        // Decoding merges them back into the original value.
        let r = TlvReader::parse(&bytes).unwrap();
        assert_eq!(r.get(TlvType::PublicKey), Some(value.as_slice()));
    }

    #[test]
    fn exactly_255_bytes_is_single_unmerged_item() {
        // A value of exactly 255 bytes followed by a *different* type must not
        // accidentally merge with the next item even though len == 255.
        let big: Vec<u8> = vec![7u8; 255];
        let mut w = TlvWriter::new();
        w.push(TlvType::PublicKey, &big).push_u8(TlvType::State, 6);
        let r = TlvReader::parse(w.as_bytes()).unwrap();
        assert_eq!(r.get(TlvType::PublicKey), Some(big.as_slice()));
        assert_eq!(r.get_u8(TlvType::State), Some(6));
    }

    #[test]
    fn two_510_byte_values_same_type_do_not_bleed_together() {
        // Edge case the 255-rule must handle: two logically distinct values of
        // the same type, each 510 bytes (= 255 + 255). Per HAP these are
        // separated by a Separator; without it they are indistinguishable, so
        // we assert the Separator-delimited framing keeps them apart.
        let a: Vec<u8> = vec![1u8; 510];
        let b: Vec<u8> = vec![2u8; 510];
        let mut w = TlvWriter::new();
        w.push(TlvType::Certificate, &a)
            .push_separator()
            .push(TlvType::Certificate, &b);
        let r = TlvReader::parse(w.as_bytes()).unwrap();

        // Split entries on the separator and decode each independently.
        let mut entries: Vec<Vec<(u8, Vec<u8>)>> = vec![Vec::new()];
        for (ty, val) in r.items() {
            if *ty == TlvType::Separator as u8 {
                entries.push(Vec::new());
            } else {
                entries.last_mut().unwrap().push((*ty, val.clone()));
            }
        }
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0][0].1, a);
        assert_eq!(entries[1][0].1, b);
    }

    #[test]
    fn truncated_stream_returns_none() {
        // Declares 10 bytes but only 3 follow.
        assert!(TlvReader::parse(&[0x03, 0x0A, 1, 2, 3]).is_none());
    }
}
