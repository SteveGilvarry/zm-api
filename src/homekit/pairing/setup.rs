//! HAP Pair-Setup state machine (HAP spec 5.6), M1–M6.
//!
//! The controller and accessory exchange six TLV8 messages over the
//! `/pair-setup` endpoint:
//!
//! - **M1→M2**: SRP start. Accessory returns salt `s` and `B`.
//! - **M3→M4**: SRP verify. Controller sends `A` + proof `M1`; accessory checks
//!   it and returns its proof `M2`. The SRP session key `K` is now shared.
//! - **M5→M6**: Exchange of long-term Ed25519 keys, each signed and encrypted
//!   under a key derived from `K`. Accessory stores the controller's
//!   `iOSDeviceLTPK` and returns its own `AccessoryLTPK`.
//!
//! This type is transport-agnostic: [`PairSetupSession::handle`] takes the
//! request TLV body and returns the response TLV body (errors are themselves
//! encoded as TLV per the spec).

use ed25519_dalek::{Signature, Signer, Verifier, VerifyingKey};

use super::super::crypto::{
    aead_open, aead_seal, hkdf_sha512, nonce_label, PAIR_SETUP_ACCESSORY_SIGN_INFO,
    PAIR_SETUP_ACCESSORY_SIGN_SALT, PAIR_SETUP_CONTROLLER_SIGN_INFO,
    PAIR_SETUP_CONTROLLER_SIGN_SALT, PAIR_SETUP_ENCRYPT_INFO, PAIR_SETUP_ENCRYPT_SALT,
};
use super::super::tlv8::{TlvError, TlvReader, TlvType, TlvWriter};
use super::srp6a::SrpAuth;
use super::{hex_encode, Controller, PairingStore};

/// State carried across the multi-message Pair-Setup exchange for one connection.
#[derive(Default)]
pub struct PairSetupSession {
    srp: Option<SrpAuth>,
    /// SRP session key `K` once M3 succeeds.
    session_key: Option<Vec<u8>>,
}

impl PairSetupSession {
    pub fn new() -> Self {
        Self::default()
    }

    /// Process one Pair-Setup request, returning the TLV response body.
    pub fn handle(&mut self, body: &[u8], store: &PairingStore, pin_digits: &str) -> Vec<u8> {
        let reader = match TlvReader::parse(body) {
            Some(r) => r,
            None => return error_tlv(2, TlvError::Unknown),
        };
        match reader.get_u8(TlvType::State) {
            Some(1) => self.m2(pin_digits),
            Some(3) => self.m4(&reader),
            Some(5) => self.m6(&reader, store),
            _ => error_tlv(2, TlvError::Unknown),
        }
    }

    /// M2: start SRP, return salt + B.
    fn m2(&mut self, pin_digits: &str) -> Vec<u8> {
        let srp = SrpAuth::generate(pin_digits.as_bytes());
        let mut w = TlvWriter::new();
        w.push_u8(TlvType::State, 2)
            .push(TlvType::Salt, srp.salt())
            .push(TlvType::PublicKey, &srp.public_key());
        self.srp = Some(srp);
        w.into_bytes()
    }

    /// M4: verify controller proof, return accessory proof.
    fn m4(&mut self, reader: &TlvReader) -> Vec<u8> {
        let srp = match self.srp.as_ref() {
            Some(s) => s,
            None => return error_tlv(4, TlvError::Unknown),
        };
        let (a_pub, proof) = match (reader.get(TlvType::PublicKey), reader.get(TlvType::Proof)) {
            (Some(a), Some(p)) => (a, p),
            _ => return error_tlv(4, TlvError::Authentication),
        };
        let outcome = match srp.process(a_pub) {
            Some(o) => o,
            None => return error_tlv(4, TlvError::Authentication),
        };
        if !outcome.verify_client_proof(proof) {
            // Wrong PIN.
            return error_tlv(4, TlvError::Authentication);
        }
        self.session_key = Some(outcome.session_key.clone());
        let mut w = TlvWriter::new();
        w.push_u8(TlvType::State, 4)
            .push(TlvType::Proof, &outcome.server_proof);
        w.into_bytes()
    }

    /// M6: decrypt controller LTPK, verify signature, persist, return accessory LTPK.
    fn m6(&mut self, reader: &TlvReader, store: &PairingStore) -> Vec<u8> {
        let k = match self.session_key.as_ref() {
            Some(k) => k,
            None => return error_tlv(6, TlvError::Unknown),
        };
        let encrypted = match reader.get(TlvType::EncryptedData) {
            Some(e) => e,
            None => return error_tlv(6, TlvError::Authentication),
        };

        let encrypt_key = hkdf_sha512(PAIR_SETUP_ENCRYPT_SALT, PAIR_SETUP_ENCRYPT_INFO, k);
        let sub = match aead_open(&encrypt_key, &nonce_label(b"PS-Msg05"), &[], encrypted) {
            Some(p) => p,
            None => return error_tlv(6, TlvError::Authentication),
        };
        let sub = match TlvReader::parse(&sub) {
            Some(t) => t,
            None => return error_tlv(6, TlvError::Authentication),
        };

        let (ios_id, ios_ltpk, ios_sig) = match (
            sub.get(TlvType::Identifier),
            sub.get(TlvType::PublicKey),
            sub.get(TlvType::Signature),
        ) {
            (Some(id), Some(pk), Some(sig)) => (id, pk, sig),
            _ => return error_tlv(6, TlvError::Authentication),
        };

        // iOSDeviceInfo = iOSDeviceX | iOSDevicePairingID | iOSDeviceLTPK
        let ios_x = hkdf_sha512(
            PAIR_SETUP_CONTROLLER_SIGN_SALT,
            PAIR_SETUP_CONTROLLER_SIGN_INFO,
            k,
        );
        let mut ios_info = Vec::new();
        ios_info.extend_from_slice(&ios_x);
        ios_info.extend_from_slice(ios_id);
        ios_info.extend_from_slice(ios_ltpk);

        if !verify_ed25519(ios_ltpk, &ios_info, ios_sig) {
            return error_tlv(6, TlvError::Authentication);
        }

        // Persist the controller (first controller becomes admin).
        let controller = Controller {
            id: String::from_utf8_lossy(ios_id).into_owned(),
            ltpk_hex: hex_encode(ios_ltpk),
            admin: true,
        };
        if store.add_controller(controller).is_err() {
            return error_tlv(6, TlvError::Unknown);
        }

        // Build accessory sub-TLV: AccessoryInfo signed with AccessoryLTSK.
        let accessory_x = hkdf_sha512(
            PAIR_SETUP_ACCESSORY_SIGN_SALT,
            PAIR_SETUP_ACCESSORY_SIGN_INFO,
            k,
        );
        let device_id = store.device_id().as_bytes().to_vec();
        let accessory_ltpk = store.public_key().to_bytes();
        let mut accessory_info = Vec::new();
        accessory_info.extend_from_slice(&accessory_x);
        accessory_info.extend_from_slice(&device_id);
        accessory_info.extend_from_slice(&accessory_ltpk);
        let signature: Signature = store.signing_key().sign(&accessory_info);

        let mut sub_w = TlvWriter::new();
        sub_w
            .push(TlvType::Identifier, &device_id)
            .push(TlvType::PublicKey, &accessory_ltpk)
            .push(TlvType::Signature, &signature.to_bytes());
        let encrypted_out = aead_seal(
            &encrypt_key,
            &nonce_label(b"PS-Msg06"),
            &[],
            sub_w.as_bytes(),
        );

        let mut w = TlvWriter::new();
        w.push_u8(TlvType::State, 6)
            .push(TlvType::EncryptedData, &encrypted_out);
        w.into_bytes()
    }
}

/// Verify an Ed25519 signature given a raw 32-byte public key and 64-byte sig.
fn verify_ed25519(public: &[u8], message: &[u8], signature: &[u8]) -> bool {
    let pk: [u8; 32] = match public.try_into() {
        Ok(p) => p,
        Err(_) => return false,
    };
    let sig: [u8; 64] = match signature.try_into() {
        Ok(s) => s,
        Err(_) => return false,
    };
    let Ok(vk) = VerifyingKey::from_bytes(&pk) else {
        return false;
    };
    vk.verify(message, &Signature::from_bytes(&sig)).is_ok()
}

/// Build a `{ State, Error }` TLV response.
fn error_tlv(state: u8, err: TlvError) -> Vec<u8> {
    let mut w = TlvWriter::new();
    w.push_u8(TlvType::State, state)
        .push_u8(TlvType::Error, err as u8);
    w.into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::homekit::pairing::srp6a::USERNAME;
    use ed25519_dalek::SigningKey;
    use num_bigint::BigUint;
    use sha2::{Digest, Sha512};

    fn store(tag: &str) -> (PairingStore, std::path::PathBuf) {
        let dir = std::env::temp_dir().join(format!("zmhk-setup-{}-{tag}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        (PairingStore::load_or_create(&dir).unwrap(), dir)
    }

    #[test]
    fn rejects_wrong_state() {
        let (s, dir) = store("wrong-state");
        let mut session = PairSetupSession::new();
        let mut w = TlvWriter::new();
        w.push_u8(TlvType::State, 99);
        let resp = session.handle(w.as_bytes(), &s, "03145154");
        let r = TlvReader::parse(&resp).unwrap();
        assert!(r.get(TlvType::Error).is_some());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn m1_returns_salt_and_public_key() {
        let (s, dir) = store("m1");
        let mut session = PairSetupSession::new();
        let mut w = TlvWriter::new();
        w.push_u8(TlvType::State, 1).push_u8(TlvType::Method, 0);
        let resp = session.handle(w.as_bytes(), &s, "03145154");
        let r = TlvReader::parse(&resp).unwrap();
        assert_eq!(r.get_u8(TlvType::State), Some(2));
        assert_eq!(r.get(TlvType::Salt).unwrap().len(), 16);
        assert!(!r.get(TlvType::PublicKey).unwrap().is_empty());
        let _ = std::fs::remove_dir_all(dir);
    }

    /// Drive a full M1→M6 handshake from a simulated controller and assert the
    /// controller's LTPK is persisted and the accessory's signed reply verifies.
    #[test]
    fn full_pair_setup_handshake() {
        let (s, dir) = store("full");
        let pin = "03145154";
        let mut session = PairSetupSession::new();

        // M1 → M2
        let mut m1 = TlvWriter::new();
        m1.push_u8(TlvType::State, 1).push_u8(TlvType::Method, 0);
        let m2 = TlvReader::parse(&session.handle(m1.as_bytes(), &s, pin)).unwrap();
        let salt = m2.get(TlvType::Salt).unwrap().to_vec();
        let b_pub = m2.get(TlvType::PublicKey).unwrap().to_vec();

        // Controller SRP: compute A, K, M1 the spec way.
        let n = BigUint::parse_bytes(super::super::srp6a::n_hex().as_bytes(), 16).unwrap();
        let g = BigUint::from(5u8);
        let a_priv = BigUint::from_bytes_be(&[0x44u8; 32]);
        let a_pub = g.modpow(&a_priv, &n);
        let a_pub_bytes = a_pub.to_bytes_be();

        let sha = |parts: &[&[u8]]| {
            let mut h = Sha512::new();
            for p in parts {
                h.update(p);
            }
            h.finalize().to_vec()
        };
        let pad = |b: &[u8]| {
            let mut o = vec![0u8; 384 - b.len()];
            o.extend_from_slice(b);
            o
        };
        let k_param =
            BigUint::from_bytes_be(&sha(&[&pad(&n.to_bytes_be()), &pad(&g.to_bytes_be())]));
        let inner = sha(&[USERNAME, b":", pin.as_bytes()]);
        let x = BigUint::from_bytes_be(&sha(&[&salt, &inner]));
        let u = BigUint::from_bytes_be(&sha(&[&pad(&a_pub_bytes), &pad(&b_pub)]));
        let g_x = g.modpow(&x, &n);
        let base = (&BigUint::from_bytes_be(&b_pub) + &n - (&k_param * &g_x) % &n) % &n;
        let exp = &a_priv + &u * &x;
        let s_val = base.modpow(&exp, &n);
        let k = sha(&[&s_val.to_bytes_be()]);
        let h_n = sha(&[&n.to_bytes_be()]);
        let h_g = sha(&[&g.to_bytes_be()]);
        let hng: Vec<u8> = h_n.iter().zip(h_g.iter()).map(|(a, b)| a ^ b).collect();
        let proof = sha(&[&hng, &sha(&[USERNAME]), &salt, &a_pub_bytes, &b_pub, &k]);

        // M3 → M4
        let mut m3 = TlvWriter::new();
        m3.push_u8(TlvType::State, 3)
            .push(TlvType::PublicKey, &a_pub_bytes)
            .push(TlvType::Proof, &proof);
        let m4 = TlvReader::parse(&session.handle(m3.as_bytes(), &s, pin)).unwrap();
        assert_eq!(m4.get_u8(TlvType::State), Some(4), "wrong M4: {m4:?}");
        // Accessory proof M2 == H(A | M1 | K)
        assert_eq!(
            m4.get(TlvType::Proof).unwrap(),
            sha(&[&a_pub_bytes, &proof, &k]).as_slice()
        );

        // M5 → M6: controller sends its signed LTPK encrypted under K.
        let ctrl_sk = SigningKey::from_bytes(&[0x55u8; 32]);
        let ctrl_ltpk = ctrl_sk.verifying_key().to_bytes();
        let ctrl_id = b"AA:BB:CC:DD:EE:FF".to_vec();
        let ios_x = super::hkdf_sha512(
            super::PAIR_SETUP_CONTROLLER_SIGN_SALT,
            super::PAIR_SETUP_CONTROLLER_SIGN_INFO,
            &k,
        );
        let mut ios_info = Vec::new();
        ios_info.extend_from_slice(&ios_x);
        ios_info.extend_from_slice(&ctrl_id);
        ios_info.extend_from_slice(&ctrl_ltpk);
        let ctrl_sig = ctrl_sk.sign(&ios_info);
        let mut sub = TlvWriter::new();
        sub.push(TlvType::Identifier, &ctrl_id)
            .push(TlvType::PublicKey, &ctrl_ltpk)
            .push(TlvType::Signature, &ctrl_sig.to_bytes());
        let enc_key = super::hkdf_sha512(
            super::PAIR_SETUP_ENCRYPT_SALT,
            super::PAIR_SETUP_ENCRYPT_INFO,
            &k,
        );
        let enc = super::aead_seal(
            &enc_key,
            &super::nonce_label(b"PS-Msg05"),
            &[],
            sub.as_bytes(),
        );
        let mut m5 = TlvWriter::new();
        m5.push_u8(TlvType::State, 5)
            .push(TlvType::EncryptedData, &enc);
        let m6 = TlvReader::parse(&session.handle(m5.as_bytes(), &s, pin)).unwrap();
        assert_eq!(m6.get_u8(TlvType::State), Some(6), "wrong M6: {m6:?}");

        // Controller is persisted as admin.
        let stored = s
            .controller("AA:BB:CC:DD:EE:FF")
            .expect("controller stored");
        assert!(stored.admin);
        assert_eq!(stored.ltpk_hex, hex_encode(&ctrl_ltpk));

        let _ = std::fs::remove_dir_all(dir);
    }
}
