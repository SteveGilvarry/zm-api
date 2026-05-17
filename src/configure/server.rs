use std::net::{AddrParseError, SocketAddr};
use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub addr: String,
    pub port: u16,
    #[serde(default)]
    pub tls: Option<ServerTlsConfig>,
    #[serde(default)]
    pub acme: Option<ServerAcmeConfig>,
    /// HTTP middleware tuning (body limits, rate limiting).
    #[serde(default)]
    pub middleware: MiddlewareConfig,
}

/// Tuning for the cross-cutting HTTP middleware stack.
#[derive(Debug, Deserialize, Clone)]
pub struct MiddlewareConfig {
    /// Maximum accepted request body size, in bytes.
    #[serde(default = "default_body_limit_bytes")]
    pub body_limit_bytes: usize,
    /// Sustained request rate allowed per client IP, in requests/second.
    /// `0` disables rate limiting entirely.
    #[serde(default)]
    pub rate_limit_per_second: u64,
    /// Burst allowance above the sustained rate. Ignored when rate limiting
    /// is disabled (`rate_limit_per_second = 0`).
    #[serde(default)]
    pub rate_limit_burst: u32,
}

fn default_body_limit_bytes() -> usize {
    2 * 1024 * 1024
}

impl Default for MiddlewareConfig {
    fn default() -> Self {
        Self {
            body_limit_bytes: default_body_limit_bytes(),
            rate_limit_per_second: 0,
            rate_limit_burst: 0,
        }
    }
}

impl MiddlewareConfig {
    /// Whether per-IP rate limiting should be installed.
    pub fn rate_limiting_enabled(&self) -> bool {
        self.rate_limit_per_second > 0
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerTlsConfig {
    #[serde(default)]
    pub enabled: bool,
    pub cert_path: Option<PathBuf>,
    pub key_path: Option<PathBuf>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerAcmeConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub domains: Vec<String>,
    #[serde(default)]
    pub contact_emails: Vec<String>,
    pub cache_dir: Option<PathBuf>,
    #[serde(default)]
    pub production: bool,
    #[serde(default)]
    pub challenge: AcmeChallenge,
    pub http_port: Option<u16>,
}

#[derive(Debug, Default, Deserialize, Clone, Copy)]
pub enum AcmeChallenge {
    #[serde(rename = "tls-alpn-01", alias = "tls-alpn01")]
    #[default]
    TlsAlpn01,
    #[serde(rename = "http-01", alias = "http01")]
    Http01,
}

impl ServerConfig {
    pub fn get_addr(&self) -> String {
        format!("{}:{}", self.addr, self.port)
    }

    pub fn get_http_addr(&self) -> String {
        format!("http://{}:{}", self.addr, self.port)
    }
    pub fn get_socket_addr(&self) -> Result<SocketAddr, AddrParseError> {
        self.get_addr().parse()
    }
}

#[cfg(test)]
pub mod tests {

    use super::*;

    #[test]
    pub fn app_config_http_addr_test() {
        let config = ServerConfig {
            addr: "127.0.0.1".to_string(),
            port: 1024,
            tls: None,
            acme: None,
            middleware: MiddlewareConfig::default(),
        };
        assert_eq!(config.get_http_addr(), "http://127.0.0.1:1024");
    }

    #[test]
    pub fn middleware_config_defaults() {
        let mw = MiddlewareConfig::default();
        assert_eq!(mw.body_limit_bytes, 2 * 1024 * 1024);
        assert!(!mw.rate_limiting_enabled());
    }
}
