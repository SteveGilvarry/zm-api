//! ZoneMinder configuration file parser
//!
//! Parses ZoneMinder's zm.conf format (KEY=VALUE) with support for
//! the conf.d override directory pattern.
//!
//! Configuration is loaded from:
//! 1. `/etc/zm/zm.conf` (base configuration)
//! 2. `/etc/zm/conf.d/*.conf` (overrides, loaded alphabetically)
//!
//! Later files override earlier values.

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tracing::{debug, info, warn};

/// Parsed ZoneMinder configuration
#[derive(Debug, Clone, Default)]
pub struct ZmConfig {
    values: HashMap<String, String>,
}

impl ZmConfig {
    /// Load ZoneMinder configuration from the standard paths
    ///
    /// Loads `/etc/zm/zm.conf` and then all files in `/etc/zm/conf.d/`
    /// with a `.conf` extension, in alphabetical order.
    pub fn load() -> Self {
        Self::load_from_path("/etc/zm")
    }

    /// Load ZoneMinder configuration from a custom base path
    ///
    /// Useful for testing or non-standard installations.
    pub fn load_from_path(base_path: &str) -> Self {
        let mut config = Self::default();

        let base = Path::new(base_path);

        // Load main zm.conf
        let main_conf = base.join("zm.conf");
        if main_conf.exists() {
            info!("Loading ZoneMinder config from {:?}", main_conf);
            config.load_file(&main_conf);
        } else {
            debug!("ZoneMinder config not found at {:?}", main_conf);
        }

        // Load conf.d overrides
        let conf_d = base.join("conf.d");
        if conf_d.is_dir() {
            config.load_conf_d(&conf_d);
        }

        config
    }

    /// Load a single configuration file
    fn load_file(&mut self, path: &Path) {
        match fs::read_to_string(path) {
            Ok(contents) => {
                for line in contents.lines() {
                    self.parse_line(line);
                }
                debug!("Loaded {} values from {:?}", self.values.len(), path);
            }
            Err(e) => {
                warn!("Failed to read {:?}: {}", path, e);
            }
        }
    }

    /// Load all .conf files from a conf.d directory
    fn load_conf_d(&mut self, conf_d: &Path) {
        let mut conf_files: Vec<_> = match fs::read_dir(conf_d) {
            Ok(entries) => entries
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.extension().map(|ext| ext == "conf").unwrap_or(false))
                .collect(),
            Err(e) => {
                debug!("Could not read conf.d directory {:?}: {}", conf_d, e);
                return;
            }
        };

        // Sort alphabetically for deterministic override order
        conf_files.sort();

        for conf_file in conf_files {
            debug!("Loading override config from {:?}", conf_file);
            self.load_file(&conf_file);
        }
    }

    /// Parse a single line from a config file
    fn parse_line(&mut self, line: &str) {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            return;
        }

        // Parse KEY=VALUE
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim().to_string();
            let value = value.trim().to_string();

            if !key.is_empty() {
                self.values.insert(key, value);
            }
        }
    }

    /// Get a configuration value
    pub fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(|s| s.as_str())
    }

    /// Get database host (ZM_DB_HOST)
    ///
    /// Handles the format "hostname[:port]" or "localhost:/path/to/socket"
    /// Returns just the hostname part.
    pub fn db_host(&self) -> Option<&str> {
        self.get("ZM_DB_HOST").map(|host| {
            // Handle socket path: localhost:/path/to/socket
            if let Some(idx) = host.find(":/") {
                &host[..idx]
            }
            // Handle port: hostname:port
            else if let Some(idx) = host.rfind(':') {
                // Only split if it looks like a port (numeric after colon)
                let after_colon = &host[idx + 1..];
                if after_colon.chars().all(|c| c.is_ascii_digit()) {
                    &host[..idx]
                } else {
                    host
                }
            } else {
                host
            }
        })
    }

    /// Get database port from ZM_DB_HOST if specified
    ///
    /// Returns None if no port is specified (use MySQL default 3306)
    pub fn db_port(&self) -> Option<u16> {
        self.get("ZM_DB_HOST").and_then(|host| {
            // Skip socket paths
            if host.contains(":/") {
                return None;
            }
            // Extract port from hostname:port
            if let Some(idx) = host.rfind(':') {
                let port_str = &host[idx + 1..];
                port_str.parse().ok()
            } else {
                None
            }
        })
    }

    /// Get database name (ZM_DB_NAME)
    pub fn db_name(&self) -> Option<&str> {
        self.get("ZM_DB_NAME")
    }

    /// Get database user (ZM_DB_USER)
    pub fn db_user(&self) -> Option<&str> {
        self.get("ZM_DB_USER")
    }

    /// Get database password (ZM_DB_PASS)
    pub fn db_pass(&self) -> Option<&str> {
        self.get("ZM_DB_PASS")
    }

    /// Check if any database configuration is available
    pub fn has_db_config(&self) -> bool {
        self.db_host().is_some()
            || self.db_name().is_some()
            || self.db_user().is_some()
            || self.db_pass().is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_config(dir: &Path, filename: &str, content: &str) {
        let path = dir.join(filename);
        let mut file = fs::File::create(&path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
    }

    #[test]
    fn test_parse_simple_config() {
        let tmp = TempDir::new().unwrap();
        create_test_config(
            tmp.path(),
            "zm.conf",
            r#"
# Comment line
ZM_DB_HOST=localhost
ZM_DB_NAME=zm
ZM_DB_USER=zmuser
ZM_DB_PASS=zmpass
"#,
        );

        let config = ZmConfig::load_from_path(tmp.path().to_str().unwrap());

        assert_eq!(config.db_host(), Some("localhost"));
        assert_eq!(config.db_name(), Some("zm"));
        assert_eq!(config.db_user(), Some("zmuser"));
        assert_eq!(config.db_pass(), Some("zmpass"));
        assert_eq!(config.db_port(), None);
    }

    #[test]
    fn test_parse_host_with_port() {
        let tmp = TempDir::new().unwrap();
        create_test_config(tmp.path(), "zm.conf", "ZM_DB_HOST=dbserver:3307\n");

        let config = ZmConfig::load_from_path(tmp.path().to_str().unwrap());

        assert_eq!(config.db_host(), Some("dbserver"));
        assert_eq!(config.db_port(), Some(3307));
    }

    #[test]
    fn test_parse_host_with_socket() {
        let tmp = TempDir::new().unwrap();
        create_test_config(
            tmp.path(),
            "zm.conf",
            "ZM_DB_HOST=localhost:/var/run/mysqld/mysqld.sock\n",
        );

        let config = ZmConfig::load_from_path(tmp.path().to_str().unwrap());

        assert_eq!(config.db_host(), Some("localhost"));
        assert_eq!(config.db_port(), None);
    }

    #[test]
    fn test_conf_d_override() {
        let tmp = TempDir::new().unwrap();

        // Create main config
        create_test_config(
            tmp.path(),
            "zm.conf",
            r#"
ZM_DB_HOST=localhost
ZM_DB_NAME=zm
ZM_DB_USER=zmuser
ZM_DB_PASS=zmpass
"#,
        );

        // Create conf.d directory
        let conf_d = tmp.path().join("conf.d");
        fs::create_dir(&conf_d).unwrap();

        // Create override config
        create_test_config(
            &conf_d,
            "01-db.conf",
            r#"
ZM_DB_HOST=dbserver
ZM_DB_PASS=secretpass
"#,
        );

        let config = ZmConfig::load_from_path(tmp.path().to_str().unwrap());

        // Overridden values
        assert_eq!(config.db_host(), Some("dbserver"));
        assert_eq!(config.db_pass(), Some("secretpass"));
        // Original values kept
        assert_eq!(config.db_name(), Some("zm"));
        assert_eq!(config.db_user(), Some("zmuser"));
    }

    #[test]
    fn test_conf_d_alphabetical_order() {
        let tmp = TempDir::new().unwrap();

        create_test_config(tmp.path(), "zm.conf", "ZM_DB_PASS=original\n");

        let conf_d = tmp.path().join("conf.d");
        fs::create_dir(&conf_d).unwrap();

        // Create files that would sort differently if not alphabetical
        create_test_config(&conf_d, "99-last.conf", "ZM_DB_PASS=last\n");
        create_test_config(&conf_d, "01-first.conf", "ZM_DB_PASS=first\n");

        let config = ZmConfig::load_from_path(tmp.path().to_str().unwrap());

        // 99-last.conf should override 01-first.conf
        assert_eq!(config.db_pass(), Some("last"));
    }

    #[test]
    fn test_empty_config() {
        let tmp = TempDir::new().unwrap();
        let config = ZmConfig::load_from_path(tmp.path().to_str().unwrap());

        assert!(!config.has_db_config());
        assert_eq!(config.db_host(), None);
    }
}
