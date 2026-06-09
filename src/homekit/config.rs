//! Configuration for the HomeKit (HAP) accessory bridge.
//!
//! Loaded as the `[homekit]` section of the application config. Disabled by
//! default; when enabled it requires a streaming-capable build (it taps the
//! existing `SourceRouter`/`SnapshotService`).

use std::path::PathBuf;

use serde::Deserialize;

/// HomeKit bridge settings.
///
/// Phase 1 exposes a single monitor ([`HomeKitConfig::monitor_id`]) as one IP
/// camera accessory behind a HAP bridge.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct HomeKitConfig {
    /// Master switch. When false, no HAP server or mDNS advertisement starts.
    pub enabled: bool,

    /// Friendly bridge name shown in the Home app.
    pub name: String,

    /// 8-digit setup code in `XXX-XX-XXX` form, entered by the user when
    /// pairing. Must not be a HAP-disallowed trivial code (e.g. all-same digits).
    pub pin: String,

    /// 4-character setup id used in the optional QR/`X-HM` setup payload and the
    /// mDNS `sh` (setup hash) computation.
    pub setup_id: String,

    /// TCP port the HAP accessory server listens on.
    pub port: u16,

    /// Directory where the accessory long-term keypair, device id, and paired
    /// controller public keys are persisted across restarts.
    pub persist_dir: PathBuf,

    /// ZoneMinder monitor id to expose as the camera accessory (Phase 1: one).
    pub monitor_id: u32,
}

impl Default for HomeKitConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            name: "ZoneMinder".to_string(),
            // HAP-valid example code; operators should override in their profile.
            pin: "031-45-154".to_string(),
            setup_id: "ZMAP".to_string(),
            // 51826 is the de-facto convention for HAP bridges (Homebridge uses 51826/51827).
            port: 51826,
            persist_dir: PathBuf::from("/var/lib/zm_api/homekit"),
            monitor_id: 1,
        }
    }
}

impl HomeKitConfig {
    /// The PIN with separators stripped, as required by the SRP setup payload
    /// (`"031-45-154"` → `"03145154"`).
    pub fn pin_digits(&self) -> String {
        self.pin.chars().filter(|c| c.is_ascii_digit()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_disabled() {
        let c = HomeKitConfig::default();
        assert!(!c.enabled);
        assert_eq!(c.port, 51826);
    }

    #[test]
    fn pin_digits_strips_separators() {
        let c = HomeKitConfig {
            pin: "031-45-154".to_string(),
            ..Default::default()
        };
        assert_eq!(c.pin_digits(), "03145154");
    }
}
