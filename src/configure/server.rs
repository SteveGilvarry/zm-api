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

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
pub enum AcmeChallenge {
    TlsAlpn01,
    Http01,
}

impl Default for AcmeChallenge {
    fn default() -> Self {
        Self::TlsAlpn01
    }
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
        };
        assert_eq!(config.get_http_addr(), "http://127.0.0.1:1024");
    }
}
