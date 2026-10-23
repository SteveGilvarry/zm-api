//! Rate-limiting key extraction.
//!
//! The key is the client's IP. Forwarding headers (`X-Forwarded-For`,
//! `X-Real-IP`) are written by whoever sends them, so they are believed only
//! when the connection comes from a trusted proxy
//! (`server.middleware.trusted_proxies`, loopback by default), and then only
//! as far as the proxies go: the client is the rightmost address in
//! `X-Forwarded-For` that isn't a trusted proxy. Proxies append the address
//! they saw to whatever the client sent, so the leftmost entries are the
//! client's own and prove nothing.
//!
//! `tower_governor`'s `SmartIpKeyExtractor` trusts the headers from anyone and
//! takes the leftmost address, so a client could mint a fresh bucket per
//! request, directly or through a proxy.

use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use axum::extract::ConnectInfo;
use axum::http::{HeaderMap, Request};
use ipnet::IpNet;
use tower_governor::errors::GovernorError;
use tower_governor::key_extractor::KeyExtractor;

/// Loopback: a proxy on the same host (ZoneMinder's Apache).
pub const DEFAULT_TRUSTED_PROXIES: [&str; 2] = ["127.0.0.0/8", "::1/128"];

/// Rate-limit key = the client IP.
///
/// - `trust_proxy = false`: the socket peer IP, whatever the headers say.
/// - `trust_proxy = true`: as above unless the peer is in `trusted`; then
///   the client address the trusted proxies passed on.
#[derive(Debug, Clone)]
pub struct IpKeyExtractor {
    pub trust_proxy: bool,
    pub trusted: Arc<[IpNet]>,
}

impl IpKeyExtractor {
    /// Parse `trusted_proxies` (addresses or CIDRs); bad entries are logged
    /// and skipped.
    pub fn new(trust_proxy: bool, trusted_proxies: &[String]) -> Self {
        let trusted: Vec<IpNet> = trusted_proxies
            .iter()
            .filter_map(|s| {
                let s = s.trim();
                s.parse::<IpNet>()
                    .or_else(|_| s.parse::<IpAddr>().map(IpNet::from))
                    .map_err(|_| {
                        tracing::warn!("server.middleware.trusted_proxies: {s:?} is not an address or CIDR; ignored")
                    })
                    .ok()
            })
            .collect();
        Self {
            trust_proxy,
            trusted: trusted.into(),
        }
    }

    fn is_trusted(&self, ip: IpAddr) -> bool {
        let ip = canonical(ip);
        self.trusted.iter().any(|n| n.contains(&ip))
    }

    /// The client's address for a request from `peer`.
    pub fn client_ip(&self, peer: IpAddr, headers: &HeaderMap) -> IpAddr {
        if !self.trust_proxy || !self.is_trusted(peer) {
            return peer;
        }
        let chain: Vec<&str> = headers
            .get_all("x-forwarded-for")
            .iter()
            .filter_map(|v| v.to_str().ok())
            .flat_map(|v| v.split(','))
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();
        if chain.is_empty() {
            return headers
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .and_then(parse_ip)
                .unwrap_or(peer);
        }
        // From the right: each trusted hop vouches for the address before it.
        let mut client = peer;
        for entry in chain.iter().rev() {
            let Some(ip) = parse_ip(entry) else {
                // Can't read the chain past here; the last address that was
                // vouched for stands.
                return client;
            };
            client = ip;
            if !self.is_trusted(ip) {
                return ip;
            }
        }
        client
    }
}

/// IPv4-mapped IPv6 as plain IPv4, so `::ffff:127.0.0.1` is loopback.
fn canonical(ip: IpAddr) -> IpAddr {
    match ip {
        IpAddr::V6(v6) => v6.to_ipv4_mapped().map_or(ip, IpAddr::V4),
        v4 => v4,
    }
}

/// An address as proxies write it: bare, `[v6]`, or with a port.
fn parse_ip(s: &str) -> Option<IpAddr> {
    let s = s.trim();
    s.parse::<IpAddr>()
        .ok()
        .or_else(|| s.parse::<SocketAddr>().ok().map(|a| a.ip()))
        .or_else(|| {
            s.strip_prefix('[')
                .and_then(|r| r.strip_suffix(']'))
                .and_then(|r| r.parse().ok())
        })
        .map(canonical)
}

impl KeyExtractor for IpKeyExtractor {
    type Key = IpAddr;

    fn extract<T>(&self, req: &Request<T>) -> Result<Self::Key, GovernorError> {
        let peer = req
            .extensions()
            .get::<ConnectInfo<SocketAddr>>()
            .map(|ConnectInfo(a)| a.ip())
            .ok_or(GovernorError::UnableToExtractKey)?;
        Ok(self.client_ip(peer, req.headers()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    fn defaults(trust_proxy: bool) -> IpKeyExtractor {
        let list: Vec<String> = DEFAULT_TRUSTED_PROXIES
            .iter()
            .map(|s| s.to_string())
            .collect();
        IpKeyExtractor::new(trust_proxy, &list)
    }

    fn key(e: &IpKeyExtractor, peer: &str, headers: &[(&str, &str)]) -> IpAddr {
        let mut builder = Request::builder().uri("/");
        for (k, v) in headers {
            builder = builder.header(*k, *v);
        }
        let mut req = builder.body(()).unwrap();
        let peer: IpAddr = peer.parse().unwrap();
        req.extensions_mut()
            .insert(ConnectInfo(SocketAddr::new(peer, 40000)));
        e.extract(&req).expect("key")
    }

    fn ip(s: &str) -> IpAddr {
        s.parse().unwrap()
    }

    #[test]
    fn untrusted_uses_peer_ip_and_ignores_forwarded_header() {
        let e = defaults(false);
        assert_eq!(
            key(&e, "127.0.0.1", &[("x-forwarded-for", "198.51.100.99")]),
            ip("127.0.0.1")
        );
    }

    /// Exposed directly: a client's own header changes nothing, even with
    /// header trust on.
    #[test]
    fn a_direct_client_cannot_pick_its_key() {
        let e = defaults(true);
        for spoof in ["198.51.100.99", "10.10.10.10", "127.0.0.1"] {
            assert_eq!(
                key(
                    &e,
                    "203.0.113.7",
                    &[("x-forwarded-for", spoof), ("x-real-ip", spoof)]
                ),
                ip("203.0.113.7"),
                "{spoof}"
            );
        }
    }

    #[test]
    fn behind_a_local_proxy_the_client_is_what_the_proxy_saw() {
        let e = defaults(true);
        assert_eq!(
            key(&e, "127.0.0.1", &[("x-forwarded-for", "198.51.100.99")]),
            ip("198.51.100.99")
        );
        // The client sent its own header; the proxy appended the real address.
        assert_eq!(
            key(
                &e,
                "127.0.0.1",
                &[("x-forwarded-for", "6.6.6.6, 198.51.100.99")]
            ),
            ip("198.51.100.99")
        );
        // Split across two header lines, as some proxies send it.
        assert_eq!(
            key(
                &e,
                "::1",
                &[
                    ("x-forwarded-for", "6.6.6.6"),
                    ("x-forwarded-for", "198.51.100.99")
                ]
            ),
            ip("198.51.100.99")
        );
        assert_eq!(
            key(&e, "127.0.0.1", &[("x-real-ip", "198.51.100.5")]),
            ip("198.51.100.5")
        );
        assert_eq!(key(&e, "127.0.0.1", &[]), ip("127.0.0.1"));
    }

    #[test]
    fn a_chain_of_trusted_proxies_is_followed() {
        let e = IpKeyExtractor::new(true, &["127.0.0.1".into(), "10.0.0.0/8".into()]);
        assert_eq!(
            key(
                &e,
                "10.0.0.3",
                &[("x-forwarded-for", "6.6.6.6, 198.51.100.99, 10.0.0.2")]
            ),
            ip("198.51.100.99")
        );
    }

    #[test]
    fn unreadable_entries_and_odd_forms() {
        let e = defaults(true);
        // Junk where the proxy's entry should be: fall back to the proxy.
        assert_eq!(
            key(&e, "127.0.0.1", &[("x-forwarded-for", "1.2.3.4, junk")]),
            ip("127.0.0.1")
        );
        assert_eq!(
            key(
                &e,
                "127.0.0.1",
                &[("x-forwarded-for", "198.51.100.99:5555")]
            ),
            ip("198.51.100.99")
        );
        assert_eq!(
            key(&e, "127.0.0.1", &[("x-forwarded-for", "[2001:db8::1]")]),
            ip("2001:db8::1")
        );
        // A v4-mapped peer is loopback too.
        assert_eq!(
            key(
                &e,
                "::ffff:127.0.0.1",
                &[("x-forwarded-for", "198.51.100.99")]
            ),
            ip("198.51.100.99")
        );
        let _ = Ipv4Addr::LOCALHOST;
    }

    #[test]
    fn bad_trusted_entries_are_skipped() {
        let e = IpKeyExtractor::new(true, &["not-an-ip".into(), "127.0.0.1".into()]);
        assert_eq!(e.trusted.len(), 1);
    }
}
