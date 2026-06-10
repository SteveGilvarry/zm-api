//! Database configuration with ZoneMinder zm.conf fallback support
//!
//! Configuration priority (highest to lowest):
//! 1. Environment variables (APP_DB__HOST, etc.)
//! 2. Profile TOML files (settings/prod.toml, etc.)
//! 3. Base TOML file (settings/base.toml)
//! 4. ZoneMinder zm.conf (/etc/zm/zm.conf + /etc/zm/conf.d/*.conf)
//!
//! This allows zm_api to work out of the box on ZoneMinder installations
//! without requiring duplicate database configuration.

use serde::Deserialize;
use tracing::info;

use super::zmconf::ZmConfig;

/// Database configuration
///
/// When using placeholder values in TOML (like "username", "password"),
/// the actual values will be loaded from ZoneMinder's zm.conf if available.
#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    #[serde(default = "default_username")]
    pub username: String,
    #[serde(default = "default_password")]
    pub password: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
    #[serde(default = "default_min_connections")]
    pub min_connections: u32,
    #[serde(default = "default_connect_timeout_secs")]
    pub connect_timeout_secs: u64,
    #[serde(default = "default_acquire_timeout_secs")]
    pub acquire_timeout_secs: u64,
    #[serde(default = "default_idle_timeout_secs")]
    pub idle_timeout_secs: u64,
    #[serde(default = "default_max_lifetime_secs")]
    pub max_lifetime_secs: u64,
    #[serde(default = "default_database_name")]
    pub database_name: String,
    /// Log every SQL statement the ORM executes.
    ///
    /// Off by default: at scale this emits one log line per query, which is
    /// both noise and measurable overhead on the request hot path. Enable it
    /// only for local debugging.
    #[serde(default = "default_sqlx_logging")]
    pub sqlx_logging: bool,
}

fn default_username() -> String {
    "zmuser".to_string()
}
fn default_password() -> String {
    "zmpass".to_string()
}
fn default_port() -> u16 {
    3306
}
fn default_host() -> String {
    "localhost".to_string()
}
fn default_max_connections() -> u32 {
    25
}
fn default_min_connections() -> u32 {
    5
}
fn default_connect_timeout_secs() -> u64 {
    8
}
fn default_acquire_timeout_secs() -> u64 {
    8
}
fn default_idle_timeout_secs() -> u64 {
    300
}
fn default_max_lifetime_secs() -> u64 {
    1800
}
fn default_sqlx_logging() -> bool {
    false
}
fn default_database_name() -> String {
    "zm".to_string()
}

/// Placeholder values that trigger zm.conf fallback
const PLACEHOLDER_VALUES: &[&str] = &[
    "username",
    "password",
    "database_name",
    "host",
    "localhost", // Also treat localhost as a potential override target
];

impl DatabaseConfig {
    /// Apply ZoneMinder zm.conf values as fallbacks
    ///
    /// This method checks if any config values are placeholders and
    /// replaces them with values from zm.conf if available.
    pub fn apply_zmconf_fallback(&mut self) {
        let zmconf = ZmConfig::load();

        if !zmconf.has_db_config() {
            return;
        }

        info!("Applying ZoneMinder zm.conf database configuration");

        // Apply host (with port extraction)
        if self.is_placeholder(&self.host) || self.host == "localhost" {
            if let Some(host) = zmconf.db_host() {
                info!("Using ZM_DB_HOST: {}", host);
                self.host = host.to_string();
            }
            // Also check for port in ZM_DB_HOST
            if let Some(port) = zmconf.db_port() {
                info!("Using port from ZM_DB_HOST: {}", port);
                self.port = port;
            }
        }

        // Apply database name
        if self.is_placeholder(&self.database_name) {
            if let Some(name) = zmconf.db_name() {
                info!("Using ZM_DB_NAME: {}", name);
                self.database_name = name.to_string();
            }
        }

        // Apply username
        if self.is_placeholder(&self.username) {
            if let Some(user) = zmconf.db_user() {
                info!("Using ZM_DB_USER: {}", user);
                self.username = user.to_string();
            }
        }

        // Apply password (don't log actual password)
        if self.is_placeholder(&self.password) {
            if let Some(pass) = zmconf.db_pass() {
                info!("Using ZM_DB_PASS: ****");
                self.password = pass.to_string();
            }
        }
    }

    /// Check if a value is a placeholder that should be replaced
    fn is_placeholder(&self, value: &str) -> bool {
        PLACEHOLDER_VALUES.contains(&value)
    }

    /// Get the database connection URL
    pub fn get_url(&self) -> String {
        Self::create_url(
            &self.username,
            &self.password,
            &self.host,
            self.port,
            &self.database_name,
        )
    }

    /// Create a database connection URL from components.
    ///
    /// The username and password are percent-encoded: ZoneMinder's packaged
    /// installers auto-generate DB passwords that routinely contain `@`, `/`,
    /// `?` or `#` (and the `/etc/zm/zm.conf` fallback can supply anything).
    /// Interpolated raw, an `@` in the password would split the authority and
    /// the URL parser would misread the host. See REVIEW_FIXES_PLAN §5.3.
    pub fn create_url(
        username: &str,
        password: &str,
        host: &str,
        port: u16,
        database_name: &str,
    ) -> String {
        let user = urlencoding::encode(username);
        let pass = urlencoding::encode(password);
        format!("mysql://{user}:{pass}@{host}:{port}/{database_name}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_url() {
        let config = DatabaseConfig {
            username: "testuser".to_string(),
            password: "testpass".to_string(),
            host: "dbhost".to_string(),
            port: 3307,
            database_name: "testdb".to_string(),
            max_connections: 10,
            min_connections: default_min_connections(),
            connect_timeout_secs: default_connect_timeout_secs(),
            acquire_timeout_secs: default_acquire_timeout_secs(),
            idle_timeout_secs: default_idle_timeout_secs(),
            max_lifetime_secs: default_max_lifetime_secs(),
            sqlx_logging: default_sqlx_logging(),
        };

        assert_eq!(
            config.get_url(),
            "mysql://testuser:testpass@dbhost:3307/testdb"
        );
    }

    #[test]
    fn create_url_percent_encodes_special_chars() {
        // A password with `@` and `/` must be encoded, and the result must parse
        // back to the original host/password (REVIEW_FIXES_PLAN §5.3).
        let url = DatabaseConfig::create_url("zm@user", "p@ss/w0rd#x", "10.0.0.5", 3306, "zm");
        assert_eq!(url, "mysql://zm%40user:p%40ss%2Fw0rd%23x@10.0.0.5:3306/zm");
        let parsed = url::Url::parse(&url).expect("URL must parse");
        assert_eq!(parsed.host_str(), Some("10.0.0.5"));
        assert_eq!(parsed.port(), Some(3306));
        assert_eq!(parsed.username(), "zm%40user");
        assert_eq!(
            parsed
                .password()
                .map(|p| urlencoding::decode(p).unwrap().into_owned()),
            Some("p@ss/w0rd#x".to_string())
        );
    }

    #[test]
    fn test_is_placeholder() {
        let config = DatabaseConfig {
            username: "username".to_string(),
            password: "password".to_string(),
            host: "localhost".to_string(),
            port: 3306,
            database_name: "database_name".to_string(),
            max_connections: 5,
            min_connections: default_min_connections(),
            connect_timeout_secs: default_connect_timeout_secs(),
            acquire_timeout_secs: default_acquire_timeout_secs(),
            idle_timeout_secs: default_idle_timeout_secs(),
            max_lifetime_secs: default_max_lifetime_secs(),
            sqlx_logging: default_sqlx_logging(),
        };

        assert!(config.is_placeholder("username"));
        assert!(config.is_placeholder("password"));
        assert!(config.is_placeholder("database_name"));
        assert!(config.is_placeholder("localhost"));
        assert!(!config.is_placeholder("realuser"));
        assert!(!config.is_placeholder("192.168.1.1"));
    }

    #[test]
    fn test_defaults() {
        // Test that defaults match ZoneMinder defaults
        assert_eq!(default_username(), "zmuser");
        assert_eq!(default_database_name(), "zm");
        assert_eq!(default_port(), 3306);
        assert_eq!(default_host(), "localhost");
        // A 30-minute connection lifetime, not the previous 8 seconds.
        assert_eq!(default_max_lifetime_secs(), 1800);
        assert!(default_min_connections() <= default_max_connections());
    }
}
