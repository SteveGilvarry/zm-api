//! SRP-6a as used by HomeKit Pair-Setup (HAP spec 5.6).
//!
//! HAP fixes SRP-6a to SHA-512 with the RFC 5054 3072-bit group, username
//! `"Pair-Setup"`, and the PIN as the password. The proof formula is the full
//! RFC 5054 one:
//!
//! ```text
//! k  = H(N | PAD(g))
//! x  = H(s | H(I | ":" | P))
//! v  = g^x mod N
//! B  = (k*v + g^b) mod N
//! u  = H(PAD(A) | PAD(B))
//! S  = (A * v^u)^b mod N
//! K  = H(S)
//! M1 = H( H(N)^H(g) | H(I) | s | A | B | K )      (client proof)
//! M2 = H( A | M1 | K )                              (server proof)
//! ```
//!
//! The stock `srp` crate computes `M1 = H(A | B | K)` and returns `S` (not
//! `H(S)`), so it is *not* iOS-compatible — hence this dedicated module. `PAD`
//! left-pads to the byte length of `N` (384 bytes); `H(g)` and the proof's `A`,
//! `B` use the unpadded protocol buffers, matching the proven reference
//! implementations (fast-srp-hap / HomeKitADK).

use num_bigint::BigUint;
use num_traits::Zero;
use sha2::{Digest, Sha512};
use subtle::ConstantTimeEq;

use super::super::crypto::fill_random;

/// RFC 5054 3072-bit group modulus `N` (big-endian, 384 bytes). Identical to
/// the `srp` crate's `groups/3072.bin`.
const N_HEX: &str = concat!(
    "ffffffffffffffffc90fdaa22168c234c4c6628b80dc1cd129024e088a67cc74",
    "020bbea63b139b22514a08798e3404ddef9519b3cd3a431b302b0a6df25f1437",
    "4fe1356d6d51c245e485b576625e7ec6f44c42e9a637ed6b0bff5cb6f406b7ed",
    "ee386bfb5a899fa5ae9f24117c4b1fe649286651ece45b3dc2007cb8a163bf05",
    "98da48361c55d39a69163fa8fd24cf5f83655d23dca3ad961c62f356208552bb",
    "9ed529077096966d670c354e4abc9804f1746c08ca18217c32905e462e36ce3b",
    "e39e772c180e86039b2783a2ec07a28fb5c55df06f4c52c9de2bcbf695581718",
    "3995497cea956ae515d2261898fa051015728e5a8aaac42dad33170d04507a33",
    "a85521abdf1cba64ecfb850458dbef0a8aea71575d060c7db3970f85a6e1e4c7",
    "abf5ae8cdb0933d71e8c94e04a25619dcee3d2261ad2ee6bf12ffa06d98a0864",
    "d87602733ec86a64521f2b18177b200cbbe117577a615d6c770988c0bad946e2",
    "08e24fa074e5ab3143db5bfce0fd108e4b82d120a93ad2caffffffffffffffff",
);

/// Generator `g = 5`.
const G: u8 = 5;
/// Byte length of `N`.
const N_LEN: usize = 384;
/// HAP SRP username.
pub const USERNAME: &[u8] = b"Pair-Setup";

fn modulus() -> BigUint {
    BigUint::parse_bytes(N_HEX.as_bytes(), 16).expect("valid 3072-bit modulus hex")
}

/// The 3072-bit group modulus as a hex string (used by tests to build a
/// reference controller).
#[cfg(test)]
pub(crate) fn n_hex() -> &'static str {
    N_HEX
}

fn sha512(parts: &[&[u8]]) -> Vec<u8> {
    let mut h = Sha512::new();
    for p in parts {
        h.update(p);
    }
    h.finalize().to_vec()
}

/// Left-pad `bytes` with zeros to exactly `N_LEN` bytes (PAD per RFC 5054).
fn pad(bytes: &[u8]) -> Vec<u8> {
    let mut out = vec![0u8; N_LEN.saturating_sub(bytes.len())];
    out.extend_from_slice(bytes);
    out
}

/// Server-side SRP-6a session for one Pair-Setup attempt.
pub struct SrpAuth {
    n: BigUint,
    g: BigUint,
    salt: Vec<u8>,
    b: BigUint,
    b_pub: BigUint,
    v: BigUint,
}

/// Result of processing the controller's `A`/proof: the derived session key and
/// the two proofs.
pub struct SrpOutcome {
    /// Session key `K = H(S)` (64 bytes) — the HKDF ikm for later derivations.
    pub session_key: Vec<u8>,
    /// Expected controller proof `M1`; compare against the controller-supplied
    /// proof in constant time via [`SrpOutcome::verify_client_proof`].
    expected_m1: Vec<u8>,
    /// Server proof `M2` to return once the client proof is accepted.
    pub server_proof: Vec<u8>,
}

impl SrpOutcome {
    /// Constant-time comparison of the controller's proof against `M1`.
    pub fn verify_client_proof(&self, client_proof: &[u8]) -> bool {
        self.expected_m1.ct_eq(client_proof).unwrap_u8() == 1
    }
}

impl SrpAuth {
    /// Create a session with a random salt and server ephemeral (production).
    pub fn generate(password: &[u8]) -> Self {
        let mut salt = vec![0u8; 16];
        fill_random(&mut salt);
        let mut b = vec![0u8; 32];
        fill_random(&mut b);
        Self::with_params(password, &salt, &b)
    }

    /// Create a session with caller-supplied `salt` and server private `b`
    /// (deterministic; used by tests).
    pub fn with_params(password: &[u8], salt: &[u8], b_priv: &[u8]) -> Self {
        let n = modulus();
        let g = BigUint::from(G);

        // x = H(s | H(I | ":" | P)); v = g^x mod N
        let inner = sha512(&[USERNAME, b":", password]);
        let x = BigUint::from_bytes_be(&sha512(&[salt, &inner]));
        let v = g.modpow(&x, &n);

        // k = H(N | PAD(g)); B = (k*v + g^b) mod N
        let k = BigUint::from_bytes_be(&sha512(&[&pad(&n.to_bytes_be()), &pad(&g.to_bytes_be())]));
        let b = BigUint::from_bytes_be(b_priv);
        let b_pub = (&k * &v + g.modpow(&b, &n)) % &n;

        Self {
            n,
            g,
            salt: salt.to_vec(),
            b,
            b_pub,
            v,
        }
    }

    /// 16-byte salt to send to the controller (TLV `Salt`).
    pub fn salt(&self) -> &[u8] {
        &self.salt
    }

    /// Server public ephemeral `B` to send to the controller (TLV `PublicKey`).
    pub fn public_key(&self) -> Vec<u8> {
        self.b_pub.to_bytes_be()
    }

    /// Process the controller's public ephemeral `A` and derive the session key
    /// and proofs. Returns `None` if `A` is illegal (`A mod N == 0`).
    pub fn process(&self, a_pub: &[u8]) -> Option<SrpOutcome> {
        let a = BigUint::from_bytes_be(a_pub);
        if (&a % &self.n).is_zero() {
            return None;
        }

        let b_pub_bytes = self.b_pub.to_bytes_be();
        // u = H(PAD(A) | PAD(B))
        let u = BigUint::from_bytes_be(&sha512(&[&pad(a_pub), &pad(&b_pub_bytes)]));
        // S = (A * v^u)^b mod N
        let s = (&a * self.v.modpow(&u, &self.n)).modpow(&self.b, &self.n);
        // K = H(S)
        let k = sha512(&[&s.to_bytes_be()]);

        // H(N) ^ H(g)  (g unpadded, per the reference implementations)
        let h_n = sha512(&[&self.n.to_bytes_be()]);
        let h_g = sha512(&[&self.g.to_bytes_be()]);
        let hng: Vec<u8> = h_n.iter().zip(h_g.iter()).map(|(a, b)| a ^ b).collect();
        let h_i = sha512(&[USERNAME]);

        // M1 = H( H(N)^H(g) | H(I) | s | A | B | K )
        let m1 = sha512(&[&hng, &h_i, &self.salt, a_pub, &b_pub_bytes, &k]);
        // M2 = H( A | M1 | K )
        let m2 = sha512(&[a_pub, &m1, &k]);

        Some(SrpOutcome {
            session_key: k,
            expected_m1: m1,
            server_proof: m2,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Reference client side, mirroring the same formula, to validate that the
    /// server's `M1`/`M2`/`K` are internally consistent (a controller computing
    /// the spec formula will agree with us).
    struct RefClient {
        n: BigUint,
        g: BigUint,
        a: BigUint,
        a_pub: BigUint,
        password: Vec<u8>,
    }

    impl RefClient {
        fn new(password: &[u8], a_priv: &[u8]) -> Self {
            let n = modulus();
            let g = BigUint::from(G);
            let a = BigUint::from_bytes_be(a_priv);
            let a_pub = g.modpow(&a, &n);
            Self {
                n,
                g,
                a,
                a_pub,
                password: password.to_vec(),
            }
        }

        /// Returns (K, M1) computed the client's way.
        fn finish(&self, salt: &[u8], b_pub: &[u8]) -> (Vec<u8>, Vec<u8>) {
            let b = BigUint::from_bytes_be(b_pub);
            let k_param = BigUint::from_bytes_be(&sha512(&[
                &pad(&self.n.to_bytes_be()),
                &pad(&self.g.to_bytes_be()),
            ]));
            let inner = sha512(&[USERNAME, b":", &self.password]);
            let x = BigUint::from_bytes_be(&sha512(&[salt, &inner]));
            let a_pub_bytes = self.a_pub.to_bytes_be();
            let u = BigUint::from_bytes_be(&sha512(&[&pad(&a_pub_bytes), &pad(b_pub)]));
            // S = (B - k*g^x)^(a + u*x) mod N
            let g_x = self.g.modpow(&x, &self.n);
            let base = (&b + &self.n - (&k_param * &g_x) % &self.n) % &self.n;
            let exp = &self.a + &u * &x;
            let s = base.modpow(&exp, &self.n);
            let k = sha512(&[&s.to_bytes_be()]);

            let h_n = sha512(&[&self.n.to_bytes_be()]);
            let h_g = sha512(&[&self.g.to_bytes_be()]);
            let hng: Vec<u8> = h_n.iter().zip(h_g.iter()).map(|(a, b)| a ^ b).collect();
            let h_i = sha512(&[USERNAME]);
            let m1 = sha512(&[&hng, &h_i, salt, &a_pub_bytes, b_pub, &k]);
            (k, m1)
        }
    }

    #[test]
    fn full_handshake_agrees_with_reference_client() {
        let password = b"03145154";
        let salt = [0x11u8; 16];
        let b_priv = [0x22u8; 32];
        let a_priv = [0x33u8; 32];

        let server = SrpAuth::with_params(password, &salt, &b_priv);
        let client = RefClient::new(password, &a_priv);

        let a_pub = client.a_pub.to_bytes_be();
        let outcome = server.process(&a_pub).expect("legal A");

        let (client_k, client_m1) = client.finish(server.salt(), &server.public_key());

        // Shared session key matches.
        assert_eq!(outcome.session_key, client_k);
        // Server accepts the client's proof.
        assert!(outcome.verify_client_proof(&client_m1));
        // Server's M2 = H(A | M1 | K) is what the client would expect.
        assert_eq!(
            outcome.server_proof,
            sha512(&[&a_pub, &client_m1, &client_k])
        );
    }

    #[test]
    fn wrong_password_is_rejected() {
        let salt = [0x11u8; 16];
        let b_priv = [0x22u8; 32];
        let a_priv = [0x33u8; 32];

        let server = SrpAuth::with_params(b"03145154", &salt, &b_priv);
        let client = RefClient::new(b"99999999", &a_priv);
        let outcome = server.process(&client.a_pub.to_bytes_be()).unwrap();
        let (_k, client_m1) = client.finish(server.salt(), &server.public_key());
        assert!(!outcome.verify_client_proof(&client_m1));
    }

    #[test]
    fn rejects_illegal_a() {
        let server = SrpAuth::with_params(b"03145154", &[0x11; 16], &[0x22; 32]);
        // A = N → A mod N == 0.
        assert!(server.process(&modulus().to_bytes_be()).is_none());
    }
}
