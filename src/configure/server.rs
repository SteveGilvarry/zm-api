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
    /// Browser origins allowed to call the API (CORS). Accepts a TOML array
    /// or, so a single environment variable can carry the whole list, one
    /// comma-separated string: `APP_SERVER__ALLOWED_ORIGINS=https://zm.example.com`.
    ///
    /// Each entry is an exact origin (`https://zm.example.com`) or a host with
    /// a `:*` port wildcard (`http://localhost:*`). Empty means "unset", which
    /// falls back to the legacy `ALLOWED_ORIGINS` variable and then to a
    /// localhost-only default — see `routes::resolve_allowed_origins`.
    #[serde(default, deserialize_with = "de_string_or_seq")]
    pub allowed_origins: Vec<String>,
    /// HTTP middleware tuning (body limits, rate limiting).
    #[serde(default)]
    pub middleware: MiddlewareConfig,
}

/// Accept either a comma-separated string or a sequence. TOML users write an
/// array; the `APP_` environment layer can only ever deliver a string, and
/// `APP_SERVER__ALLOWED_ORIGINS__0=` indexing is a poor fit for a setting most
/// people set once to a single origin.
fn de_string_or_seq<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrSeq {
        String(String),
        Seq(Vec<String>),
    }

    let split = |s: &str| -> Vec<String> {
        s.split(',')
            .map(str::trim)
            .filter(|p| !p.is_empty())
            .map(str::to_owned)
            .collect()
    };

    Ok(match StringOrSeq::deserialize(deserializer)? {
        StringOrSeq::String(s) => split(&s),
        // Split sequence entries too, so a one-element array holding a
        // comma-separated string behaves the same as the bare string.
        StringOrSeq::Seq(v) => v.iter().flat_map(|s| split(s)).collect(),
    })
}

/// Tuning for the cross-cutting HTTP middleware stack.
#[derive(Debug, Deserialize, Clone)]
pub struct MiddlewareConfig {
    /// Maximum accepted request body size, in bytes.
    #[serde(default = "default_body_limit_bytes")]
    pub body_limit_bytes: usize,
    /// Governor replenish interval for the global limiter, in seconds (one
    /// unit of quota is restored per interval). `0` disables the global
    /// limiter entirely.
    #[serde(default)]
    pub rate_limit_per_second: u64,
    /// Burst allowance for the global limiter. Ignored when the global limiter
    /// is disabled (`rate_limit_per_second = 0`).
    #[serde(default)]
    pub rate_limit_burst: u32,
    /// Trust proxy forwarding headers (`X-Forwarded-For` / `X-Real-IP`) for
    /// rate-limit keying. Leave `false` unless the service sits behind a
    /// trusted reverse proxy: when the service is exposed directly these
    /// headers are attacker-controlled and would let a client mint a fresh
    /// rate-limit bucket per request. When `false`, the socket peer IP is used.
    #[serde(default)]
    pub trust_proxy_headers: bool,
    /// Governor replenish interval, in seconds, for the dedicated limiter on
    /// the authentication endpoints (`/auth/login`, `/auth/refresh`, and the
    /// other `/auth` + `/me` routes). Kept tight to blunt credential
    /// brute-forcing regardless of the global limiter. `0` disables it.
    #[serde(default = "default_auth_rate_limit_period_secs")]
    pub auth_rate_limit_period_secs: u64,
    /// Burst allowance for the auth-endpoint limiter. `0` disables it.
    #[serde(default = "default_auth_rate_limit_burst")]
    pub auth_rate_limit_burst: u32,
}

fn default_body_limit_bytes() -> usize {
    2 * 1024 * 1024
}

fn default_auth_rate_limit_period_secs() -> u64 {
    2
}

fn default_auth_rate_limit_burst() -> u32 {
    10
}

impl Default for MiddlewareConfig {
    fn default() -> Self {
        Self {
            body_limit_bytes: default_body_limit_bytes(),
            rate_limit_per_second: 0,
            rate_limit_burst: 0,
            trust_proxy_headers: false,
            auth_rate_limit_period_secs: default_auth_rate_limit_period_secs(),
            auth_rate_limit_burst: default_auth_rate_limit_burst(),
        }
    }
}

impl MiddlewareConfig {
    /// Whether the global per-IP rate limiter should be installed.
    pub fn rate_limiting_enabled(&self) -> bool {
        self.rate_limit_per_second > 0
    }

    /// Whether the dedicated auth-endpoint rate limiter should be installed.
    /// On by default (brute-force protection matters even when the global
    /// limiter is off); disable by setting either field to `0`.
    pub fn auth_rate_limiting_enabled(&self) -> bool {
        self.auth_rate_limit_period_secs > 0 && self.auth_rate_limit_burst > 0
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

    /// The deployment path that matters: `APP_SERVER__ALLOWED_ORIGINS` arriving
    /// through the real `config` env layer. Worth its own test because the
    /// string-or-seq deserializer is `untagged`, which needs the backend to
    /// support `deserialize_any` — serde_json proves nothing about the `config`
    /// crate's own deserializer. Uses `Environment::source` rather than real
    /// environment variables, which are process-global and race other tests.
    #[test]
    fn allowed_origins_round_trips_through_the_config_env_layer() {
        let env = config::Environment::with_prefix("APP")
            .prefix_separator("_")
            .separator("__")
            .source(Some(
                [(
                    "APP_SERVER__ALLOWED_ORIGINS".to_string(),
                    "https://zm.example.com,https://alt.example.com".to_string(),
                )]
                .into_iter()
                .collect(),
            ));

        let cfg = config::Config::builder()
            .set_default("server.addr", "0.0.0.0")
            .unwrap()
            .set_default("server.port", 8080)
            .unwrap()
            .add_source(env)
            .build()
            .expect("build config")
            .get::<ServerConfig>("server")
            .expect("deserialize ServerConfig");

        assert_eq!(
            cfg.allowed_origins,
            vec!["https://zm.example.com", "https://alt.example.com"]
        );
    }

    #[test]
    pub fn app_config_http_addr_test() {
        let config = ServerConfig {
            addr: "127.0.0.1".to_string(),
            port: 1024,
            tls: None,
            acme: None,
            allowed_origins: Vec::new(),
            middleware: MiddlewareConfig::default(),
        };
        assert_eq!(config.get_http_addr(), "http://127.0.0.1:1024");
    }

    #[test]
    pub fn middleware_config_defaults() {
        let mw = MiddlewareConfig::default();
        assert_eq!(mw.body_limit_bytes, 2 * 1024 * 1024);
        // Global limiter off by default; auth limiter on by default.
        assert!(!mw.rate_limiting_enabled());
        assert!(mw.auth_rate_limiting_enabled());
        // Forwarding headers are not trusted by default (no proxy assumed).
        assert!(!mw.trust_proxy_headers);
    }

    #[test]
    pub fn auth_rate_limiting_can_be_disabled() {
        let mw = MiddlewareConfig {
            auth_rate_limit_burst: 0,
            ..Default::default()
        };
        assert!(!mw.auth_rate_limiting_enabled());
        let mw = MiddlewareConfig {
            auth_rate_limit_period_secs: 0,
            ..Default::default()
        };
        assert!(!mw.auth_rate_limiting_enabled());
    }

    #[derive(Debug, Deserialize)]
    struct OriginsHolder {
        #[serde(default, deserialize_with = "de_string_or_seq")]
        allowed_origins: Vec<String>,
    }

    fn parse_origins(json: &str) -> Vec<String> {
        serde_json::from_str::<OriginsHolder>(json)
            .expect("parse")
            .allowed_origins
    }

    #[test]
    fn allowed_origins_accepts_a_sequence() {
        // The shape a TOML array in base.toml deserializes to.
        assert_eq!(
            parse_origins(r#"{"allowed_origins": ["https://a.example", "http://localhost:*"]}"#),
            vec!["https://a.example", "http://localhost:*"]
        );
    }

    #[test]
    fn allowed_origins_accepts_a_comma_separated_string() {
        // The only shape the APP_SERVER__ALLOWED_ORIGINS env layer can produce.
        assert_eq!(
            parse_origins(r#"{"allowed_origins": "https://a.example, http://localhost:*"}"#),
            vec!["https://a.example", "http://localhost:*"]
        );
    }

    #[test]
    fn allowed_origins_defaults_to_empty_and_drops_blanks() {
        assert!(parse_origins("{}").is_empty());
        assert!(parse_origins(r#"{"allowed_origins": "  ,, "}"#).is_empty());
        assert!(parse_origins(r#"{"allowed_origins": []}"#).is_empty());
    }
}
