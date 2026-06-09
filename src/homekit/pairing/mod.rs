//! Persistent pairing state for the HAP accessory.
//!
//! HAP requires the accessory to keep a stable long-term identity across
//! restarts:
//! - a per-accessory **device id** (a random, MAC-address-shaped string) used
//!   as the mDNS `id` and the SRP/Ed25519 "AccessoryPairingID";
//! - a long-term **Ed25519 keypair** (`AccessoryLTSK`/`AccessoryLTPK`) used to
//!   sign Pair-Setup/Pair-Verify exchanges;
//! - the set of **paired controllers**, each identified by its iOS-supplied
//!   pairing id and long-term public key (`iOSDeviceLTPK`), with permissions.
//!
//! This module owns that state and its on-disk persistence (a single JSON file
//! under [`crate::homekit::config::HomeKitConfig::persist_dir`]).

pub mod setup;
pub mod srp6a;
pub mod verify;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use ed25519_dalek::{SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};

use super::crypto::fill_random;
use crate::error::{AppError, AppResult};

const STORE_FILE: &str = "pairings.json";

/// A paired iOS controller.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Controller {
    /// iOS pairing identifier (`iOSDevicePairingID`), an opaque UTF-8 string.
    pub id: String,
    /// Controller long-term public key (Ed25519), hex-encoded.
    pub ltpk_hex: String,
    /// True if this controller has admin (pairing-management) permission.
    pub admin: bool,
}

impl Controller {
    pub fn ltpk(&self) -> Option<VerifyingKey> {
        let bytes = hex_decode(&self.ltpk_hex)?;
        let arr: [u8; 32] = bytes.try_into().ok()?;
        VerifyingKey::from_bytes(&arr).ok()
    }
}

/// On-disk representation of the accessory identity + pairings.
#[derive(Debug, Serialize, Deserialize)]
struct PersistedState {
    device_id: String,
    /// Accessory Ed25519 signing-key seed (32 bytes), hex-encoded.
    signing_seed_hex: String,
    controllers: Vec<Controller>,
}

/// Live pairing state, backed by a JSON file.
pub struct PairingStore {
    path: PathBuf,
    device_id: String,
    signing_key: SigningKey,
    controllers: RwLock<HashMap<String, Controller>>,
}

impl PairingStore {
    /// Load the store from `persist_dir`, creating a fresh identity (and the
    /// directory) on first run.
    pub fn load_or_create(persist_dir: &Path) -> AppResult<Self> {
        std::fs::create_dir_all(persist_dir).map_err(|e| {
            AppError::UnknownError(anyhow::anyhow!(
                "homekit: cannot create persist dir {persist_dir:?}: {e}"
            ))
        })?;
        let path = persist_dir.join(STORE_FILE);

        if path.exists() {
            let raw = std::fs::read(&path).map_err(io_err)?;
            let state: PersistedState = serde_json::from_slice(&raw).map_err(|e| {
                AppError::UnknownError(anyhow::anyhow!("homekit: corrupt pairing store: {e}"))
            })?;
            let seed: [u8; 32] = hex_decode(&state.signing_seed_hex)
                .and_then(|v| v.try_into().ok())
                .ok_or_else(|| {
                    AppError::UnknownError(anyhow::anyhow!("homekit: bad signing seed in store"))
                })?;
            let controllers = state
                .controllers
                .into_iter()
                .map(|c| (c.id.clone(), c))
                .collect();
            Ok(Self {
                path,
                device_id: state.device_id,
                signing_key: SigningKey::from_bytes(&seed),
                controllers: RwLock::new(controllers),
            })
        } else {
            let device_id = random_device_id();
            let mut seed = [0u8; 32];
            fill_random(&mut seed);
            let store = Self {
                path,
                device_id,
                signing_key: SigningKey::from_bytes(&seed),
                controllers: RwLock::new(HashMap::new()),
            };
            store.persist()?;
            Ok(store)
        }
    }

    /// Accessory device id (mDNS `id` / AccessoryPairingID).
    pub fn device_id(&self) -> &str {
        &self.device_id
    }

    /// Accessory long-term signing key.
    pub fn signing_key(&self) -> &SigningKey {
        &self.signing_key
    }

    /// Accessory long-term public key (`AccessoryLTPK`).
    pub fn public_key(&self) -> VerifyingKey {
        self.signing_key.verifying_key()
    }

    /// True if at least one controller is paired (drives the mDNS `sf` flag).
    pub fn is_paired(&self) -> bool {
        !self.controllers.read().unwrap().is_empty()
    }

    /// Look up a paired controller by its pairing id.
    pub fn controller(&self, id: &str) -> Option<Controller> {
        self.controllers.read().unwrap().get(id).cloned()
    }

    /// Add or replace a controller pairing, then persist.
    pub fn add_controller(&self, controller: Controller) -> AppResult<()> {
        self.controllers
            .write()
            .unwrap()
            .insert(controller.id.clone(), controller);
        self.persist()
    }

    /// Remove a controller pairing, then persist. Returns true if one was removed.
    pub fn remove_controller(&self, id: &str) -> AppResult<bool> {
        let removed = self.controllers.write().unwrap().remove(id).is_some();
        if removed {
            self.persist()?;
        }
        Ok(removed)
    }

    fn persist(&self) -> AppResult<()> {
        let controllers: Vec<Controller> =
            self.controllers.read().unwrap().values().cloned().collect();
        let state = PersistedState {
            device_id: self.device_id.clone(),
            signing_seed_hex: hex_encode(self.signing_key.to_bytes().as_slice()),
            controllers,
        };
        let json = serde_json::to_vec_pretty(&state).map_err(|e| {
            AppError::UnknownError(anyhow::anyhow!("homekit: serialize pairing store: {e}"))
        })?;
        // Write-then-rename for atomicity so a crash mid-write can't truncate
        // the accessory identity.
        let tmp = self.path.with_extension("json.tmp");
        std::fs::write(&tmp, &json).map_err(io_err)?;
        std::fs::rename(&tmp, &self.path).map_err(io_err)?;
        Ok(())
    }
}

fn io_err(e: std::io::Error) -> AppError {
    AppError::UnknownError(anyhow::anyhow!("homekit: pairing store io: {e}"))
}

/// Generate a random `XX:XX:XX:XX:XX:XX` device id. HomeKit treats this as an
/// opaque identifier; it need not correspond to a real NIC.
fn random_device_id() -> String {
    let mut b = [0u8; 6];
    fill_random(&mut b);
    format!(
        "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
        b[0], b[1], b[2], b[3], b[4], b[5]
    )
}

pub(crate) fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

pub(crate) fn hex_decode(s: &str) -> Option<Vec<u8>> {
    if !s.len().is_multiple_of(2) {
        return None;
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_and_reloads_stable_identity() {
        let dir = std::env::temp_dir().join(format!("zmhk-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);

        let s1 = PairingStore::load_or_create(&dir).unwrap();
        let id = s1.device_id().to_string();
        let pk = s1.public_key().to_bytes();
        assert!(!s1.is_paired());

        // Reload: identity must be byte-identical.
        let s2 = PairingStore::load_or_create(&dir).unwrap();
        assert_eq!(s2.device_id(), id);
        assert_eq!(s2.public_key().to_bytes(), pk);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn persists_and_removes_controllers() {
        let dir = std::env::temp_dir().join(format!("zmhk-ctrl-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);

        let ltpk_hex = hex_encode(&[0xABu8; 32]);
        {
            let s = PairingStore::load_or_create(&dir).unwrap();
            s.add_controller(Controller {
                id: "ios-1".into(),
                ltpk_hex: ltpk_hex.clone(),
                admin: true,
            })
            .unwrap();
            assert!(s.is_paired());
        }
        {
            let s = PairingStore::load_or_create(&dir).unwrap();
            let c = s.controller("ios-1").expect("reloaded controller");
            assert_eq!(c.ltpk_hex, ltpk_hex);
            assert!(c.admin);
            assert!(s.remove_controller("ios-1").unwrap());
            assert!(!s.is_paired());
        }

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn hex_round_trip() {
        let data = [0u8, 1, 2, 250, 255];
        assert_eq!(hex_decode(&hex_encode(&data)).unwrap(), data);
    }
}
