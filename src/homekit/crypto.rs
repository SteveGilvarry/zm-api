//! Shared cryptographic primitives for HAP pairing and sessions.
//!
//! HAP fixes specific constructions (HAP spec ch. 5): HKDF-SHA-512 for all key
//! derivation, and ChaCha20-Poly1305 (RFC 7539) for all AEAD. The only thing
//! that varies is the 12-byte nonce: pairing messages use a fixed ASCII label
//! in the low 8 bytes; the encrypted session uses a 64-bit little-endian frame
//! counter. Both are produced by [`nonce_label`] / [`nonce_counter`].

use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use hkdf::Hkdf;
use sha2::Sha512;

/// Fill `buf` with cryptographically-secure random bytes.
///
/// Uses the process CSPRNG via `rand` (matching the rest of the codebase) so we
/// don't couple the dalek key types to a specific `rand_core` version — keys are
/// always built from raw bytes.
pub fn fill_random(buf: &mut [u8]) {
    use rand::RngCore;
    rand::rng().fill_bytes(buf);
}

/// Derive a 32-byte key with HKDF-SHA-512 (the only KDF HAP uses).
pub fn hkdf_sha512(salt: &[u8], info: &[u8], ikm: &[u8]) -> [u8; 32] {
    let hk = Hkdf::<Sha512>::new(Some(salt), ikm);
    let mut okm = [0u8; 32];
    hk.expand(info, &mut okm)
        .expect("32 is a valid HKDF-SHA512 output length");
    okm
}

/// Build a 12-byte nonce from an ASCII pairing label (e.g. `b"PS-Msg05"`).
///
/// HAP places the label in the trailing 8 bytes, leaving the leading 4 bytes
/// zero.
pub fn nonce_label(label: &[u8]) -> [u8; 12] {
    let mut n = [0u8; 12];
    let start = 12 - 8;
    let len = label.len().min(8);
    n[start..start + len].copy_from_slice(&label[..len]);
    n
}

/// Build a 12-byte nonce from a 64-bit session frame counter (little-endian in
/// the trailing 8 bytes).
pub fn nonce_counter(counter: u64) -> [u8; 12] {
    let mut n = [0u8; 12];
    n[4..].copy_from_slice(&counter.to_le_bytes());
    n
}

/// AEAD-encrypt `plaintext` with optional associated data, returning
/// ciphertext with the 16-byte Poly1305 tag appended (HAP layout).
pub fn aead_seal(key: &[u8; 32], nonce: &[u8; 12], aad: &[u8], plaintext: &[u8]) -> Vec<u8> {
    let cipher = ChaCha20Poly1305::new(Key::from_slice(key));
    cipher
        .encrypt(
            Nonce::from_slice(nonce),
            Payload {
                msg: plaintext,
                aad,
            },
        )
        .expect("chacha20poly1305 encryption is infallible for valid inputs")
}

/// AEAD-decrypt `ciphertext` (tag-appended) with optional associated data.
/// Returns `None` on authentication failure.
pub fn aead_open(
    key: &[u8; 32],
    nonce: &[u8; 12],
    aad: &[u8],
    ciphertext: &[u8],
) -> Option<Vec<u8>> {
    let cipher = ChaCha20Poly1305::new(Key::from_slice(key));
    cipher
        .decrypt(
            Nonce::from_slice(nonce),
            Payload {
                msg: ciphertext,
                aad,
            },
        )
        .ok()
}

// --- HAP key-derivation salt/info constants (HAP spec ch. 5.6–5.7). ---

pub const PAIR_SETUP_ENCRYPT_SALT: &[u8] = b"Pair-Setup-Encrypt-Salt";
pub const PAIR_SETUP_ENCRYPT_INFO: &[u8] = b"Pair-Setup-Encrypt-Info";
pub const PAIR_SETUP_CONTROLLER_SIGN_SALT: &[u8] = b"Pair-Setup-Controller-Sign-Salt";
pub const PAIR_SETUP_CONTROLLER_SIGN_INFO: &[u8] = b"Pair-Setup-Controller-Sign-Info";
pub const PAIR_SETUP_ACCESSORY_SIGN_SALT: &[u8] = b"Pair-Setup-Accessory-Sign-Salt";
pub const PAIR_SETUP_ACCESSORY_SIGN_INFO: &[u8] = b"Pair-Setup-Accessory-Sign-Info";

pub const PAIR_VERIFY_ENCRYPT_SALT: &[u8] = b"Pair-Verify-Encrypt-Salt";
pub const PAIR_VERIFY_ENCRYPT_INFO: &[u8] = b"Pair-Verify-Encrypt-Info";

pub const CONTROL_SALT: &[u8] = b"Control-Salt";
pub const CONTROL_READ_INFO: &[u8] = b"Control-Read-Encryption-Key";
pub const CONTROL_WRITE_INFO: &[u8] = b"Control-Write-Encryption-Key";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aead_round_trip() {
        let key = [9u8; 32];
        let nonce = nonce_label(b"PS-Msg05");
        let ct = aead_seal(&key, &nonce, &[], b"hello homekit");
        // Tag is appended: ciphertext is plaintext_len + 16.
        assert_eq!(ct.len(), b"hello homekit".len() + 16);
        let pt = aead_open(&key, &nonce, &[], &ct).unwrap();
        assert_eq!(pt, b"hello homekit");
    }

    #[test]
    fn aead_rejects_tampering() {
        let key = [1u8; 32];
        let nonce = nonce_counter(0);
        let mut ct = aead_seal(&key, &nonce, b"aad", b"secret");
        ct[0] ^= 0xFF;
        assert!(aead_open(&key, &nonce, b"aad", &ct).is_none());
    }

    #[test]
    fn nonce_label_layout() {
        assert_eq!(
            nonce_label(b"PV-Msg02"),
            [0, 0, 0, 0, b'P', b'V', b'-', b'M', b's', b'g', b'0', b'2']
        );
    }

    #[test]
    fn nonce_counter_layout() {
        assert_eq!(nonce_counter(1), [0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn hkdf_is_deterministic() {
        let a = hkdf_sha512(CONTROL_SALT, CONTROL_READ_INFO, b"shared-secret");
        let b = hkdf_sha512(CONTROL_SALT, CONTROL_READ_INFO, b"shared-secret");
        assert_eq!(a, b);
        let c = hkdf_sha512(CONTROL_SALT, CONTROL_WRITE_INFO, b"shared-secret");
        assert_ne!(a, c, "different info must yield different keys");
    }
}
