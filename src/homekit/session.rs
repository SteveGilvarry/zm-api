//! HAP session transport encryption (HAP spec 6.5.2).
//!
//! After a successful Pair-Verify, every byte on the TCP connection is wrapped
//! in AEAD frames. Each frame is:
//!
//! ```text
//! [ length: u16 little-endian ]   <- additional authenticated data (AAD)
//! [ ciphertext: `length` bytes ]
//! [ Poly1305 tag: 16 bytes     ]
//! ```
//!
//! `length` is the size of the plaintext block, capped at 1024 bytes — larger
//! payloads (e.g. `/accessories`) are split across multiple frames. The two
//! directions use independent keys and independent 64-bit frame counters that
//! start at 0 and increment per frame.

use super::crypto::{aead_open, aead_seal, nonce_counter};

/// Maximum plaintext bytes per HAP frame.
const MAX_FRAME: usize = 1024;
/// Poly1305 tag length.
const TAG_LEN: usize = 16;
/// AAD length prefix.
const LEN_PREFIX: usize = 2;

/// Bidirectional session cipher for one paired connection.
///
/// Key naming follows the controller's point of view, matching the HAP
/// `Control-Read`/`Control-Write` derivation:
/// - `controller_to_accessory` decrypts inbound frames (controller writes).
/// - `accessory_to_controller` encrypts outbound frames (accessory writes).
pub struct SessionCrypto {
    controller_to_accessory: [u8; 32],
    accessory_to_controller: [u8; 32],
    inbound_counter: u64,
    outbound_counter: u64,
}

impl SessionCrypto {
    pub fn new(controller_to_accessory: [u8; 32], accessory_to_controller: [u8; 32]) -> Self {
        Self {
            controller_to_accessory,
            accessory_to_controller,
            inbound_counter: 0,
            outbound_counter: 0,
        }
    }

    /// Encrypt an outbound plaintext message into one or more wire frames.
    pub fn encrypt(&mut self, plaintext: &[u8]) -> Vec<u8> {
        let mut out = Vec::with_capacity(plaintext.len() + TAG_LEN + LEN_PREFIX);
        // An empty message still has no frames; HAP only frames real data.
        for chunk in plaintext.chunks(MAX_FRAME) {
            let len = chunk.len() as u16;
            let aad = len.to_le_bytes();
            let nonce = nonce_counter(self.outbound_counter);
            let ct = aead_seal(&self.accessory_to_controller, &nonce, &aad, chunk);
            out.extend_from_slice(&aad);
            out.extend_from_slice(&ct);
            self.outbound_counter += 1;
        }
        out
    }

    /// Attempt to decrypt as many complete inbound frames as `buf` contains.
    ///
    /// Returns the decrypted plaintext and the number of bytes consumed from
    /// `buf`. Incomplete trailing data is left unconsumed for the caller to
    /// retry once more bytes arrive. Returns [`SessionError::Auth`] if a
    /// complete frame fails authentication (the connection must be dropped).
    pub fn decrypt_available(&mut self, buf: &[u8]) -> Result<(Vec<u8>, usize), SessionError> {
        let mut plaintext = Vec::new();
        let mut offset = 0;

        loop {
            if offset + LEN_PREFIX > buf.len() {
                break;
            }
            let len = u16::from_le_bytes([buf[offset], buf[offset + 1]]) as usize;
            let frame_end = offset + LEN_PREFIX + len + TAG_LEN;
            if frame_end > buf.len() {
                // Frame not fully arrived yet.
                break;
            }
            let aad = &buf[offset..offset + LEN_PREFIX];
            let ct = &buf[offset + LEN_PREFIX..frame_end];
            let nonce = nonce_counter(self.inbound_counter);
            let pt = aead_open(&self.controller_to_accessory, &nonce, aad, ct)
                .ok_or(SessionError::Auth)?;
            plaintext.extend_from_slice(&pt);
            self.inbound_counter += 1;
            offset = frame_end;
        }

        Ok((plaintext, offset))
    }
}

/// Errors from the session transport layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionError {
    /// A fully-received frame failed Poly1305 authentication.
    Auth,
}

impl std::fmt::Display for SessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionError::Auth => write!(f, "HAP session frame authentication failed"),
        }
    }
}

impl std::error::Error for SessionError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn pair() -> (SessionCrypto, SessionCrypto) {
        let c2a = [3u8; 32];
        let a2c = [7u8; 32];
        // Accessory uses (c2a inbound, a2c outbound).
        let accessory = SessionCrypto::new(c2a, a2c);
        // A simulated controller mirrors the keys: it decrypts a2c and encrypts c2a.
        let controller = SessionCrypto::new(a2c, c2a);
        (accessory, controller)
    }

    #[test]
    fn accessory_to_controller_round_trip() {
        let (mut accessory, mut controller) = pair();
        let msg = b"HTTP/1.1 200 OK\r\n\r\n{}";
        let wire = accessory.encrypt(msg);
        let (pt, consumed) = controller.decrypt_available(&wire).unwrap();
        assert_eq!(consumed, wire.len());
        assert_eq!(pt, msg);
    }

    #[test]
    fn large_payload_is_split_and_reassembled() {
        let (mut accessory, mut controller) = pair();
        let msg = vec![0x5Au8; MAX_FRAME * 2 + 100]; // 3 frames
        let wire = accessory.encrypt(&msg);
        let (pt, _) = controller.decrypt_available(&wire).unwrap();
        assert_eq!(pt, msg);
    }

    #[test]
    fn partial_frame_is_not_consumed() {
        let (mut accessory, mut controller) = pair();
        let msg = b"some data here";
        let wire = accessory.encrypt(msg);
        // Feed all but the last 5 bytes: nothing should decode yet.
        let (pt, consumed) = controller
            .decrypt_available(&wire[..wire.len() - 5])
            .unwrap();
        assert!(pt.is_empty());
        assert_eq!(consumed, 0);
        // Now feed the whole thing.
        let (pt, consumed) = controller.decrypt_available(&wire).unwrap();
        assert_eq!(pt, msg);
        assert_eq!(consumed, wire.len());
    }

    #[test]
    fn counters_advance_so_frames_are_ordered() {
        let (mut accessory, mut controller) = pair();
        let w1 = accessory.encrypt(b"one");
        let w2 = accessory.encrypt(b"two");
        // Decrypting in order works.
        let mut joined = w1.clone();
        joined.extend_from_slice(&w2);
        let (pt, _) = controller.decrypt_available(&joined).unwrap();
        assert_eq!(pt, b"onetwo");
    }

    #[test]
    fn tampered_frame_is_rejected() {
        let (mut accessory, mut controller) = pair();
        let mut wire = accessory.encrypt(b"trust me");
        let n = wire.len();
        wire[n - 1] ^= 0x01; // corrupt the tag
        assert_eq!(controller.decrypt_available(&wire), Err(SessionError::Auth));
    }
}
