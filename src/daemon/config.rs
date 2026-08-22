//! Configuration for the daemon controller.

use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Configuration for the daemon controller service.
#[derive(Debug, Deserialize, Clone)]
pub struct DaemonConfig {
    /// Whether the daemon controller is enabled.
    ///
    /// Defaults to `false` ("passive" mode): zm-api runs as a REST API only and
    /// does not create the daemon manager, bind the `zmdc.sock` socket, or run
    /// `kill_orphan_daemons()` — so it coexists safely with a running stock
    /// ZoneMinder install. Set to `true` ("active"/takeover mode) only after
    /// disabling `zoneminder.service`, so zm-api can supervise the ZM daemons.
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Path to ZM socket directory (default: /run/zm)
    #[serde(default = "default_socket_path")]
    pub socket_path: PathBuf,

    /// Socket filename (default: zmdc.sock)
    #[serde(default = "default_socket_name")]
    pub socket_name: String,

    /// Path to ZM binaries (default: /usr/bin)
    #[serde(default = "default_bin_path")]
    pub bin_path: PathBuf,

    /// Path to ZM scripts (default: /usr/bin). Only a hint — see
    /// [`DaemonConfig::resolve_daemon_path`], which searches the standard
    /// per-distribution locations when the configured one has no match.
    #[serde(default = "default_script_path")]
    pub script_path: PathBuf,

    /// Minimum backoff delay in seconds (default: 5)
    #[serde(default = "default_min_backoff_seconds")]
    pub min_backoff_seconds: u64,

    /// Maximum backoff delay in seconds (default: 900 = 15 minutes)
    #[serde(default = "default_max_backoff_seconds")]
    pub max_backoff_seconds: u64,

    /// Graceful shutdown timeout before SIGKILL in seconds (default: 30)
    #[serde(default = "default_shutdown_timeout_seconds")]
    pub shutdown_timeout_seconds: u64,

    /// Database stats update interval in seconds (default: 60)
    #[serde(default = "default_stats_update_interval_seconds")]
    pub stats_update_interval_seconds: u64,

    /// Enable legacy socket IPC (default: true)
    #[serde(default = "default_enable_socket_ipc")]
    pub enable_socket_ipc: bool,

    /// Enable REST API integration (default: true)
    #[serde(default = "default_enable_rest_api")]
    pub enable_rest_api: bool,

    /// Enable watchdog to monitor daemon health (default: true)
    #[serde(default = "default_enable_watchdog")]
    pub enable_watchdog: bool,

    /// Watchdog check interval in seconds (default: 10, matches ZM_WATCH_CHECK_INTERVAL)
    #[serde(default = "default_watch_check_interval_seconds")]
    pub watch_check_interval_seconds: u64,

    /// Maximum heartbeat delay before restart in seconds (default: 30, matches ZM_WATCH_MAX_DELAY)
    #[serde(default = "default_watch_max_delay_seconds")]
    pub watch_max_delay_seconds: u64,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            socket_path: default_socket_path(),
            socket_name: default_socket_name(),
            bin_path: default_bin_path(),
            script_path: default_script_path(),
            min_backoff_seconds: default_min_backoff_seconds(),
            max_backoff_seconds: default_max_backoff_seconds(),
            shutdown_timeout_seconds: default_shutdown_timeout_seconds(),
            stats_update_interval_seconds: default_stats_update_interval_seconds(),
            enable_socket_ipc: default_enable_socket_ipc(),
            enable_rest_api: default_enable_rest_api(),
            enable_watchdog: default_enable_watchdog(),
            watch_check_interval_seconds: default_watch_check_interval_seconds(),
            watch_max_delay_seconds: default_watch_max_delay_seconds(),
        }
    }
}

impl DaemonConfig {
    /// Get the full path to the Unix socket file.
    pub fn socket_file(&self) -> PathBuf {
        self.socket_path.join(&self.socket_name)
    }

    /// Get the minimum backoff duration.
    pub fn min_backoff(&self) -> Duration {
        Duration::from_secs(self.min_backoff_seconds)
    }

    /// Get the maximum backoff duration.
    pub fn max_backoff(&self) -> Duration {
        Duration::from_secs(self.max_backoff_seconds)
    }

    /// Get the shutdown timeout duration.
    pub fn shutdown_timeout(&self) -> Duration {
        Duration::from_secs(self.shutdown_timeout_seconds)
    }

    /// Get the stats update interval duration.
    pub fn stats_update_interval(&self) -> Duration {
        Duration::from_secs(self.stats_update_interval_seconds)
    }

    /// Get the watchdog check interval duration.
    pub fn watch_check_interval(&self) -> Duration {
        Duration::from_secs(self.watch_check_interval_seconds)
    }

    /// Get the maximum heartbeat delay before restart.
    pub fn watch_max_delay(&self) -> Duration {
        Duration::from_secs(self.watch_max_delay_seconds)
    }

    /// Resolve a daemon command to its full path.
    ///
    /// The configured directory is tried first, then the standard locations —
    /// distributions disagree about where ZoneMinder's Perl scripts land
    /// (`/usr/bin` on Debian/Ubuntu, `/usr/share/zoneminder/scripts` on the RPM
    /// distros), and a single packaged default cannot be right for everyone.
    /// Falls back to the configured path when nothing exists, so a genuinely
    /// missing daemon still reports the path the operator configured.
    pub fn resolve_daemon_path(&self, command: &str) -> PathBuf {
        let configured = if command.ends_with(".pl") {
            &self.script_path
        } else {
            &self.bin_path
        };
        let fallbacks = if command.ends_with(".pl") {
            SCRIPT_PATH_FALLBACKS
        } else {
            BIN_PATH_FALLBACKS
        };

        let preferred = configured.join(command);
        if preferred.exists() {
            return preferred;
        }
        for dir in fallbacks {
            let candidate = Path::new(dir).join(command);
            if candidate.exists() {
                return candidate;
            }
        }
        preferred
    }
}

/// Where ZoneMinder's Perl scripts live, by distribution.
const SCRIPT_PATH_FALLBACKS: &[&str] = &[
    "/usr/bin",                      // Debian / Ubuntu
    "/usr/share/zoneminder/scripts", // Fedora / openSUSE / RHEL
    "/usr/local/bin",                // source installs
];

/// Where ZoneMinder's compiled binaries (zmc, zma) live.
const BIN_PATH_FALLBACKS: &[&str] = &["/usr/bin", "/usr/local/bin"];

fn default_enabled() -> bool {
    // Passive by default: never seize daemon control from a running ZoneMinder
    // on a fresh install. Operators opt into takeover explicitly.
    false
}

fn default_socket_path() -> PathBuf {
    PathBuf::from("/run/zm")
}

fn default_socket_name() -> String {
    "zmdc.sock".to_string()
}

fn default_bin_path() -> PathBuf {
    PathBuf::from("/usr/bin")
}

fn default_script_path() -> PathBuf {
    // On Ubuntu/Debian, scripts are also in /usr/bin
    PathBuf::from("/usr/bin")
}

fn default_min_backoff_seconds() -> u64 {
    5
}

fn default_max_backoff_seconds() -> u64 {
    900 // 15 minutes
}

fn default_shutdown_timeout_seconds() -> u64 {
    30
}

fn default_stats_update_interval_seconds() -> u64 {
    60
}

fn default_enable_socket_ipc() -> bool {
    true
}

fn default_enable_rest_api() -> bool {
    true
}

fn default_enable_watchdog() -> bool {
    true
}

fn default_watch_check_interval_seconds() -> u64 {
    10 // ZM_WATCH_CHECK_INTERVAL default
}

fn default_watch_max_delay_seconds() -> u64 {
    30 // ZM_WATCH_MAX_DELAY default
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = DaemonConfig::default();
        // Passive by default — daemon control is opt-in so we never collide
        // with a running stock ZoneMinder on a fresh install.
        assert!(!config.enabled);
        assert_eq!(config.socket_path, PathBuf::from("/run/zm"));
        assert_eq!(config.socket_name, "zmdc.sock");
        assert_eq!(config.min_backoff_seconds, 5);
        assert_eq!(config.max_backoff_seconds, 900);
        assert!(config.enable_watchdog);
        assert_eq!(config.watch_check_interval_seconds, 10);
        assert_eq!(config.watch_max_delay_seconds, 30);
    }

    #[test]
    fn test_socket_file_path() {
        let config = DaemonConfig::default();
        assert_eq!(config.socket_file(), PathBuf::from("/run/zm/zmdc.sock"));
    }

    #[test]
    fn test_resolve_daemon_path() {
        let config = DaemonConfig::default();
        assert_eq!(
            config.resolve_daemon_path("zmc"),
            PathBuf::from("/usr/bin/zmc")
        );
        // On Ubuntu/Debian, scripts are also in /usr/bin
        assert_eq!(
            config.resolve_daemon_path("zmfilter.pl"),
            PathBuf::from("/usr/bin/zmfilter.pl")
        );
    }

    #[test]
    fn test_duration_getters() {
        let config = DaemonConfig::default();
        assert_eq!(config.min_backoff(), Duration::from_secs(5));
        assert_eq!(config.max_backoff(), Duration::from_secs(900));
        assert_eq!(config.shutdown_timeout(), Duration::from_secs(30));
        assert_eq!(config.stats_update_interval(), Duration::from_secs(60));
        assert_eq!(config.watch_check_interval(), Duration::from_secs(10));
        assert_eq!(config.watch_max_delay(), Duration::from_secs(30));
    }

    #[test]
    fn resolve_prefers_the_configured_directory_when_it_has_the_script() {
        // Per-process dir: a fixed name would collide between concurrent runs.
        let dir =
            std::env::temp_dir().join(format!("zmapi-resolve-configured-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("zmdc.pl"), b"").unwrap();

        let cfg = DaemonConfig {
            script_path: dir.clone(),
            ..DaemonConfig::default()
        };
        assert_eq!(cfg.resolve_daemon_path("zmdc.pl"), dir.join("zmdc.pl"));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn resolve_falls_back_to_the_configured_path_when_nothing_exists() {
        // A genuinely missing daemon must still report the configured location,
        // not a fallback the operator never chose.
        let cfg = DaemonConfig {
            script_path: PathBuf::from("/nonexistent/zm/scripts"),
            ..DaemonConfig::default()
        };
        assert_eq!(
            cfg.resolve_daemon_path("definitely-not-a-real-daemon.pl"),
            PathBuf::from("/nonexistent/zm/scripts/definitely-not-a-real-daemon.pl")
        );
    }

    #[test]
    fn scripts_and_binaries_use_their_own_search_paths() {
        // `.pl` resolves against script_path, everything else against bin_path.
        let cfg = DaemonConfig {
            script_path: PathBuf::from("/nonexistent/scripts"),
            bin_path: PathBuf::from("/nonexistent/bin"),
            ..DaemonConfig::default()
        };
        assert!(cfg
            .resolve_daemon_path("zmnosuch.pl")
            .starts_with("/nonexistent/scripts"));
        assert!(cfg
            .resolve_daemon_path("zmnosuch")
            .starts_with("/nonexistent/bin"));
    }
}
