//! HAP Pair-Verify state machine (HAP spec 5.7), M1–M4.
//!
//! Run on every reconnection by an already-paired controller to establish a
//! fresh encrypted session:
//!
//! - **M1→M2**: Curve25519 ECDH. Controller sends its ephemeral public key;
//!   accessory replies with its own ephemeral public key plus a signed,
//!   encrypted proof of identity.
//! - **M3→M4**: Controller returns its signed, encrypted proof; accessory looks
//!   up the controller's stored `iOSDeviceLTPK` and verifies it.
//!
//! On success the shared secret is expanded (HKDF) into the two directional
//! [`SessionCrypto`] keys that protect all subsequent traffic.

use ed25519_dalek::{Signature, Signer, Verifier, VerifyingKey};
use x25519_dalek::{PublicKey, StaticSecret};

use super::super::crypto::{
    aead_open, aead_seal, fill_random, hkdf_sha512, nonce_label, CONTROL_READ_INFO, CONTROL_SALT,
    CONTROL_WRITE_INFO, PAIR_VERIFY_ENCRYPT_INFO, PAIR_VERIFY_ENCRYPT_SALT,
};
use super::super::session::SessionCrypto;
use super::super::tlv8::{TlvError, TlvReader, TlvType, TlvWriter};
use super::PairingStore;

/// Result of one Pair-Verify step: the TLV response, and — once M3 succeeds —
/// the negotiated [`SessionCrypto`] for the connection.
pub struct VerifyResult {
    pub response: Vec<u8>,
    pub session: Option<SessionCrypto>,
}

enum State {
    Init,
    AwaitingM3 {
        shared: [u8; 32],
        accessory_pub: [u8; 32],
        controller_pub: [u8; 32],
        session_key: [u8; 32],
    },
    Done,
}

/// Pair-Verify session state for one connection.
pub struct PairVerifySession {
    state: State,
    /// Optional fixed accessory ephemeral secret (tests); random otherwise.
    fixed_secret: Option<[u8; 32]>,
}

impl Default for PairVerifySession {
    fn default() -> Self {
        Self {
            state: State::Init,
            fixed_secret: None,
        }
    }
}

impl PairVerifySession {
    pub fn new() -> Self {
        Self::default()
    }

    #[cfg(test)]
    fn with_secret(secret: [u8; 32]) -> Self {
        Self {
            state: State::Init,
            fixed_secret: Some(secret),
        }
    }

    pub fn handle(&mut self, body: &[u8], store: &PairingStore) -> VerifyResult {
        let reader = match TlvReader::parse(body) {
            Some(r) => r,
            None => return plain(error_tlv(2, TlvError::Unknown)),
        };
        match reader.get_u8(TlvType::State) {
            Some(1) => plain(self.m2(&reader, store)),
            Some(3) => self.m4(&reader, store),
            _ => plain(error_tlv(2, TlvError::Unknown)),
        }
    }

    /// M2: ECDH + signed, encrypted accessory proof.
    fn m2(&mut self, reader: &TlvReader, store: &PairingStore) -> Vec<u8> {
        let controller_pub: [u8; 32] = match reader
            .get(TlvType::PublicKey)
            .and_then(|p| p.try_into().ok())
        {
            Some(p) => p,
            None => return error_tlv(2, TlvError::Unknown),
        };

        let secret_bytes = self.fixed_secret.unwrap_or_else(|| {
            let mut b = [0u8; 32];
            fill_random(&mut b);
            b
        });
        let secret = StaticSecret::from(secret_bytes);
        let accessory_pub = PublicKey::from(&secret).to_bytes();
        let shared = secret
            .diffie_hellman(&PublicKey::from(controller_pub))
            .to_bytes();

        // AccessoryInfo = AccessoryPublic | AccessoryPairingID | iOSDevicePublic
        let device_id = store.device_id().as_bytes().to_vec();
        let mut info = Vec::new();
        info.extend_from_slice(&accessory_pub);
        info.extend_from_slice(&device_id);
        info.extend_from_slice(&controller_pub);
        let signature: Signature = store.signing_key().sign(&info);

        let mut sub = TlvWriter::new();
        sub.push(TlvType::Identifier, &device_id)
            .push(TlvType::Signature, &signature.to_bytes());

        let session_key = hkdf_sha512(PAIR_VERIFY_ENCRYPT_SALT, PAIR_VERIFY_ENCRYPT_INFO, &shared);
        let encrypted = aead_seal(&session_key, &nonce_label(b"PV-Msg02"), &[], sub.as_bytes());

        self.state = State::AwaitingM3 {
            shared,
            accessory_pub,
            controller_pub,
            session_key,
        };

        let mut w = TlvWriter::new();
        w.push_u8(TlvType::State, 2)
            .push(TlvType::PublicKey, &accessory_pub)
            .push(TlvType::EncryptedData, &encrypted);
        w.into_bytes()
    }

    /// M4: verify controller proof, then derive transport session keys.
    fn m4(&mut self, reader: &TlvReader, store: &PairingStore) -> VerifyResult {
        let (shared, accessory_pub, controller_pub, session_key) = match &self.state {
            State::AwaitingM3 {
                shared,
                accessory_pub,
                controller_pub,
                session_key,
            } => (*shared, *accessory_pub, *controller_pub, *session_key),
            _ => return plain(error_tlv(4, TlvError::Unknown)),
        };

        let encrypted = match reader.get(TlvType::EncryptedData) {
            Some(e) => e,
            None => return plain(error_tlv(4, TlvError::Authentication)),
        };
        let sub = match aead_open(&session_key, &nonce_label(b"PV-Msg03"), &[], encrypted) {
            Some(p) => p,
            None => return plain(error_tlv(4, TlvError::Authentication)),
        };
        let sub = match TlvReader::parse(&sub) {
            Some(t) => t,
            None => return plain(error_tlv(4, TlvError::Authentication)),
        };
        let (ios_id, ios_sig) = match (sub.get(TlvType::Identifier), sub.get(TlvType::Signature)) {
            (Some(id), Some(sig)) => (id, sig),
            _ => return plain(error_tlv(4, TlvError::Authentication)),
        };

        // Look up the paired controller's long-term public key.
        let controller = match store.controller(&String::from_utf8_lossy(ios_id)) {
            Some(c) => c,
            None => return plain(error_tlv(4, TlvError::Authentication)),
        };
        let ltpk = match controller.ltpk() {
            Some(k) => k,
            None => return plain(error_tlv(4, TlvError::Authentication)),
        };

        // iOSDeviceInfo = iOSDevicePublic | iOSDevicePairingID | AccessoryPublic
        let mut info = Vec::new();
        info.extend_from_slice(&controller_pub);
        info.extend_from_slice(ios_id);
        info.extend_from_slice(&accessory_pub);
        if !verify_ed25519(&ltpk, &info, ios_sig) {
            return plain(error_tlv(4, TlvError::Authentication));
        }

        // Derive directional transport keys from the ECDH shared secret.
        let c2a = hkdf_sha512(CONTROL_SALT, CONTROL_WRITE_INFO, &shared);
        let a2c = hkdf_sha512(CONTROL_SALT, CONTROL_READ_INFO, &shared);
        self.state = State::Done;

        let mut w = TlvWriter::new();
        w.push_u8(TlvType::State, 4);
        VerifyResult {
            response: w.into_bytes(),
            session: Some(SessionCrypto::new(c2a, a2c)),
        }
    }
}

fn verify_ed25519(vk: &VerifyingKey, message: &[u8], signature: &[u8]) -> bool {
    let Ok(sig): Result<[u8; 64], _> = signature.try_into() else {
        return false;
    };
    vk.verify(message, &Signature::from_bytes(&sig)).is_ok()
}

fn plain(response: Vec<u8>) -> VerifyResult {
    VerifyResult {
        response,
        session: None,
    }
}

fn error_tlv(state: u8, err: TlvError) -> Vec<u8> {
    let mut w = TlvWriter::new();
    w.push_u8(TlvType::State, state)
        .push_u8(TlvType::Error, err as u8);
    w.into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::homekit::pairing::{hex_encode, Controller};
    use ed25519_dalek::SigningKey;

    fn store_with_controller(
        ctrl_sk: &SigningKey,
        ctrl_id: &str,
    ) -> (PairingStore, std::path::PathBuf) {
        let dir = std::env::temp_dir().join(format!("zmhk-verify-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let store = PairingStore::load_or_create(&dir).unwrap();
        store
            .add_controller(Controller {
                id: ctrl_id.to_string(),
                ltpk_hex: hex_encode(&ctrl_sk.verifying_key().to_bytes()),
                admin: true,
            })
            .unwrap();
        (store, dir)
    }

    #[test]
    fn full_pair_verify_handshake_yields_session() {
        let ctrl_sk = SigningKey::from_bytes(&[0x66u8; 32]);
        let ctrl_id = "11:22:33:44:55:66";
        let (store, dir) = store_with_controller(&ctrl_sk, ctrl_id);

        // Controller ephemeral keypair.
        let ctrl_secret = StaticSecret::from([0x77u8; 32]);
        let ctrl_pub = PublicKey::from(&ctrl_secret).to_bytes();

        let mut session = PairVerifySession::with_secret([0x88u8; 32]);

        // M1 → M2
        let mut m1 = TlvWriter::new();
        m1.push_u8(TlvType::State, 1)
            .push(TlvType::PublicKey, &ctrl_pub);
        let res = session.handle(m1.as_bytes(), &store);
        assert!(res.session.is_none());
        let m2 = TlvReader::parse(&res.response).unwrap();
        assert_eq!(m2.get_u8(TlvType::State), Some(2));
        let acc_pub: [u8; 32] = m2.get(TlvType::PublicKey).unwrap().try_into().unwrap();

        // Controller derives shared + verify-encrypt key the same way.
        let shared = ctrl_secret
            .diffie_hellman(&PublicKey::from(acc_pub))
            .to_bytes();
        let sk = hkdf_sha512(PAIR_VERIFY_ENCRYPT_SALT, PAIR_VERIFY_ENCRYPT_INFO, &shared);

        // Controller signs iOSDeviceInfo and encrypts its sub-TLV for M3.
        let mut info = Vec::new();
        info.extend_from_slice(&ctrl_pub);
        info.extend_from_slice(ctrl_id.as_bytes());
        info.extend_from_slice(&acc_pub);
        let sig = ctrl_sk.sign(&info);
        let mut sub = TlvWriter::new();
        sub.push(TlvType::Identifier, ctrl_id.as_bytes())
            .push(TlvType::Signature, &sig.to_bytes());
        let enc = aead_seal(&sk, &nonce_label(b"PV-Msg03"), &[], sub.as_bytes());

        let mut m3 = TlvWriter::new();
        m3.push_u8(TlvType::State, 3)
            .push(TlvType::EncryptedData, &enc);
        let res = session.handle(m3.as_bytes(), &store);
        let m4 = TlvReader::parse(&res.response).unwrap();
        assert_eq!(m4.get_u8(TlvType::State), Some(4), "wrong M4: {m4:?}");
        assert!(
            res.session.is_some(),
            "session keys must be derived on success"
        );

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn unknown_controller_is_rejected() {
        let ctrl_sk = SigningKey::from_bytes(&[0x66u8; 32]);
        let (store, dir) = store_with_controller(&ctrl_sk, "known-id");

        let ctrl_secret = StaticSecret::from([0x77u8; 32]);
        let ctrl_pub = PublicKey::from(&ctrl_secret).to_bytes();
        let mut session = PairVerifySession::with_secret([0x88u8; 32]);

        let mut m1 = TlvWriter::new();
        m1.push_u8(TlvType::State, 1)
            .push(TlvType::PublicKey, &ctrl_pub);
        let res = session.handle(m1.as_bytes(), &store);
        let acc_pub: [u8; 32] = TlvReader::parse(&res.response)
            .unwrap()
            .get(TlvType::PublicKey)
            .unwrap()
            .try_into()
            .unwrap();

        let shared = ctrl_secret
            .diffie_hellman(&PublicKey::from(acc_pub))
            .to_bytes();
        let sk = hkdf_sha512(PAIR_VERIFY_ENCRYPT_SALT, PAIR_VERIFY_ENCRYPT_INFO, &shared);
        // Sign as an *unknown* controller id.
        let mut info = Vec::new();
        info.extend_from_slice(&ctrl_pub);
        info.extend_from_slice(b"stranger");
        info.extend_from_slice(&acc_pub);
        let sig = ctrl_sk.sign(&info);
        let mut sub = TlvWriter::new();
        sub.push(TlvType::Identifier, b"stranger")
            .push(TlvType::Signature, &sig.to_bytes());
        let enc = aead_seal(&sk, &nonce_label(b"PV-Msg03"), &[], sub.as_bytes());
        let mut m3 = TlvWriter::new();
        m3.push_u8(TlvType::State, 3)
            .push(TlvType::EncryptedData, &enc);
        let res = session.handle(m3.as_bytes(), &store);
        let m4 = TlvReader::parse(&res.response).unwrap();
        assert!(m4.get(TlvType::Error).is_some());
        assert!(res.session.is_none());

        let _ = std::fs::remove_dir_all(dir);
    }
}
