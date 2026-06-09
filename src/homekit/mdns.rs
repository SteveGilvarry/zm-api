//! mDNS/Bonjour advertisement for the HAP accessory (HAP spec ch. 6.4).
//!
//! Publishes a `_hap._tcp` service whose TXT records tell controllers how to
//! reach and pair with the bridge. The `sf` (status flags) record flips from
//! `1` (unpaired, discoverable for setup) to `0` (paired) once a controller is
//! stored; re-registering with a new TXT map updates it live.

use std::collections::HashMap;

use mdns_sd::{ServiceDaemon, ServiceInfo};
use sha2::{Digest, Sha512};

use crate::error::{AppError, AppResult};

/// HAP service type.
const SERVICE_TYPE: &str = "_hap._tcp.local.";
/// HAP accessory category: 2 = Bridge (HAP spec table 12-3).
const CATEGORY_BRIDGE: &str = "2";

/// Owns the mDNS daemon and the current registration, allowing the `sf` flag to
/// be updated after pairing.
pub struct Advertiser {
    daemon: ServiceDaemon,
    fullname: String,
    instance: String,
    device_id: String,
    model: String,
    setup_id: String,
    port: u16,
    config_number: u32,
}

impl Advertiser {
    /// Start advertising. `instance` is the user-visible bridge name.
    pub fn start(
        instance: &str,
        device_id: &str,
        model: &str,
        setup_id: &str,
        port: u16,
        paired: bool,
    ) -> AppResult<Self> {
        let daemon = ServiceDaemon::new().map_err(mdns_err)?;
        let adv = Self {
            daemon,
            fullname: format!("{instance}.{SERVICE_TYPE}"),
            instance: instance.to_string(),
            device_id: device_id.to_string(),
            model: model.to_string(),
            setup_id: setup_id.to_string(),
            port,
            config_number: 1,
        };
        adv.register(paired)?;
        Ok(adv)
    }

    /// Re-register reflecting the current paired state (updates `sf`).
    pub fn set_paired(&self, paired: bool) -> AppResult<()> {
        self.register(paired)
    }

    fn register(&self, paired: bool) -> AppResult<()> {
        // `sh` setup hash = base64(first 4 bytes of SHA-512(setupId + deviceId)).
        let mut hasher = Sha512::new();
        hasher.update(self.setup_id.as_bytes());
        hasher.update(self.device_id.as_bytes());
        let digest = hasher.finalize();
        let setup_hash = base64::engine::general_purpose::STANDARD.encode(&digest[..4]);

        let mut txt: HashMap<String, String> = HashMap::new();
        txt.insert("c#".into(), self.config_number.to_string()); // config number
        txt.insert("ff".into(), "0".into()); // feature flags
        txt.insert("id".into(), self.device_id.clone()); // device id
        txt.insert("md".into(), self.model.clone()); // model
        txt.insert("pv".into(), "1.1".into()); // protocol version
        txt.insert("s#".into(), "1".into()); // state number
        txt.insert("sf".into(), if paired { "0" } else { "1" }.into()); // status flags
        txt.insert("ci".into(), CATEGORY_BRIDGE.into()); // category
        txt.insert("sh".into(), setup_hash); // setup hash

        let host_name = format!("{}.local.", sanitize(&self.instance));
        let mut info =
            ServiceInfo::new(SERVICE_TYPE, &self.instance, &host_name, "", self.port, txt)
                .map_err(mdns_err)?;
        // Announce on all routable interface addresses.
        info = info.enable_addr_auto();

        self.daemon.register(info).map_err(mdns_err)?;
        Ok(())
    }
}

impl Drop for Advertiser {
    fn drop(&mut self) {
        let _ = self.daemon.unregister(&self.fullname);
        let _ = self.daemon.shutdown();
    }
}

/// Sanitize an instance name into a valid mDNS host label.
fn sanitize(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect()
}

fn mdns_err(e: mdns_sd::Error) -> AppError {
    AppError::UnknownError(anyhow::anyhow!("homekit mdns: {e}"))
}

use base64::Engine as _;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitizes_host_labels() {
        assert_eq!(sanitize("Front Door Cam!"), "Front-Door-Cam-");
        assert_eq!(sanitize("ZoneMinder"), "ZoneMinder");
    }
}
