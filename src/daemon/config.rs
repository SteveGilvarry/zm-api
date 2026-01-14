//! Configuration for the daemon controller.

use serde::Deserialize;
use std::path::PathBuf;
use std::time::Duration;

/// Configuration for the daemon controller service.
#[derive(Debug, Deserialize, Clone)]
pub struct DaemonConfig {
    /// Whether the daemon controller is enabled
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

    /// Path to ZM scripts (default: /usr/share/zoneminder/scripts)
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

    /// Resolve a daemon command to its full path.
    pub fn resolve_daemon_path(&self, command: &str) -> PathBuf {
        if command.ends_with(".pl") {
            self.script_path.join(command)
        } else {
            self.bin_path.join(command)
        }
    }
}

fn default_enabled() -> bool {
    true
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = DaemonConfig::default();
        assert!(config.enabled);
        assert_eq!(config.socket_path, PathBuf::from("/run/zm"));
        assert_eq!(config.socket_name, "zmdc.sock");
        assert_eq!(config.min_backoff_seconds, 5);
        assert_eq!(config.max_backoff_seconds, 900);
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
    }
}
