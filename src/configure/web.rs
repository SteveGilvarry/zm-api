//! Serving the browser UI (zm-web) from this binary.
//!
//! zm-web is a static SPA — a `dist/` of hashed assets plus an `index.html`.
//! Historically it needed a reverse proxy in front of both it and this API.
//! Serving it here collapses that to one process: no nginx, no CORS (the UI and
//! the API share an origin by construction), and TLS is already handled by
//! `[server.tls]` / `[server.acme]`.
//!
//! Off by default. Enabling it changes what an unauthenticated request to `/`
//! returns, so it is an explicit choice rather than something that switches on
//! because a directory happens to exist.

use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct WebConfig {
    /// Serve the UI from this binary. Off by default: a deployment that puts a
    /// reverse proxy or a CDN in front should keep serving the files there.
    #[serde(default)]
    pub enabled: bool,

    /// Directory holding the built UI — the `dist/` output, containing
    /// `index.html`. Packaged installs of zm-web use `/usr/share/zm-web`.
    #[serde(default = "default_root")]
    pub root: PathBuf,

    /// `Cache-Control: max-age` for fingerprinted assets, in seconds.
    ///
    /// Build tools hash asset filenames, so a given URL's contents never
    /// change and a year is safe. `index.html` is deliberately excluded and
    /// always sent `no-cache`: it is the file that names the current asset
    /// hashes, so caching it pins a browser to a stale build.
    #[serde(default = "default_asset_max_age")]
    pub asset_max_age_secs: u64,

    /// Value for the `Content-Security-Policy` header on UI responses. Empty
    /// disables the header.
    ///
    /// Applied only to the UI, never to API responses — a JSON endpoint has no
    /// use for one, and a wrong CSP on the UI breaks the page loudly rather
    /// than silently, which is the failure mode you want.
    #[serde(default = "default_csp")]
    pub content_security_policy: String,
}

fn default_root() -> PathBuf {
    PathBuf::from("/usr/share/zm-web")
}

fn default_asset_max_age() -> u64 {
    31_536_000 // one year
}

/// Deliberately permissive about where media and XHR may go, because the UI
/// talks to this same origin and may be pointed at a TURN server; strict about
/// script and object sources, which is where XSS lands.
fn default_csp() -> String {
    [
        "default-src 'self'",
        "script-src 'self'",
        "style-src 'self' 'unsafe-inline'",
        "img-src 'self' data: blob:",
        "media-src 'self' blob:",
        "connect-src 'self' ws: wss:",
        "font-src 'self' data:",
        "object-src 'none'",
        "frame-ancestors 'none'",
        "base-uri 'self'",
    ]
    .join("; ")
}

impl Default for WebConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            root: default_root(),
            asset_max_age_secs: default_asset_max_age(),
            content_security_policy: default_csp(),
        }
    }
}

impl WebConfig {
    /// Whether the UI should actually be mounted.
    ///
    /// Enabled-but-missing is treated as "do not mount" rather than a startup
    /// failure: the API is the more important half, and refusing to boot
    /// because a UI package is not installed yet would be a poor trade. The
    /// caller logs a warning so it is not silent.
    pub fn should_serve(&self) -> bool {
        self.enabled && self.index_path().is_file()
    }

    pub fn index_path(&self) -> PathBuf {
        self.root.join("index.html")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_by_default() {
        let cfg = WebConfig::default();
        assert!(!cfg.enabled);
        assert!(!cfg.should_serve());
    }

    #[test]
    fn enabled_but_absent_does_not_serve() {
        let cfg = WebConfig {
            enabled: true,
            root: PathBuf::from("/nonexistent/zm-web"),
            ..WebConfig::default()
        };
        assert!(
            !cfg.should_serve(),
            "a missing UI must not mount — and must not stop the API booting"
        );
    }

    #[test]
    fn enabled_with_an_index_serves() {
        let dir = std::env::temp_dir().join(format!("zm-web-cfg-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("index.html"), b"<!doctype html>").unwrap();

        let cfg = WebConfig {
            enabled: true,
            root: dir.clone(),
            ..WebConfig::default()
        };
        assert!(cfg.should_serve());
        assert_eq!(cfg.index_path(), dir.join("index.html"));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn the_default_csp_locks_down_the_dangerous_directives() {
        let csp = default_csp();
        assert!(csp.contains("object-src 'none'"));
        assert!(csp.contains("frame-ancestors 'none'"));
        assert!(
            csp.contains("script-src 'self'") && !csp.contains("script-src 'self' 'unsafe-inline'"),
            "inline script must not be allowed: {csp}"
        );
    }
}
