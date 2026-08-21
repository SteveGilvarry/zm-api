//! Rate-limiting key extraction.
//!
//! `tower_governor`'s `SmartIpKeyExtractor` always trusts `X-Forwarded-For` /
//! `X-Real-IP`. When the service is exposed directly (no reverse proxy) those
//! headers are attacker-controlled, so a client can rotate them to mint a fresh
//! rate-limit bucket per request and defeat the limiter entirely. This
//! extractor makes the source explicit and configurable.

use std::net::IpAddr;

use axum::http::Request;
use tower_governor::errors::GovernorError;
use tower_governor::key_extractor::{KeyExtractor, PeerIpKeyExtractor, SmartIpKeyExtractor};

/// Rate-limit key = the client IP, sourced according to `trust_proxy`:
///
/// - `false` (default): the socket **peer** IP. Forwarding headers are ignored,
///   so a direct client cannot forge a new bucket per request.
/// - `true`: the proxy-set forwarding headers (`X-Forwarded-For` / `X-Real-IP`
///   / `Forwarded`, falling back to the peer IP). Only enable this when a
///   trusted reverse proxy sets those headers, otherwise it reintroduces the
///   spoofing bypass.
#[derive(Debug, Clone, Copy)]
pub struct IpKeyExtractor {
    pub trust_proxy: bool,
}

impl KeyExtractor for IpKeyExtractor {
    type Key = IpAddr;

    fn extract<T>(&self, req: &Request<T>) -> Result<Self::Key, GovernorError> {
        if self.trust_proxy {
            SmartIpKeyExtractor.extract(req)
        } else {
            PeerIpKeyExtractor.extract(req)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::ConnectInfo;
    use std::net::{Ipv4Addr, SocketAddr};

    fn request_with(peer: SocketAddr, xff: Option<&str>) -> Request<()> {
        let mut builder = Request::builder().uri("/");
        if let Some(v) = xff {
            builder = builder.header("X-Forwarded-For", v);
        }
        let mut req = builder.body(()).unwrap();
        // PeerIpKeyExtractor reads the peer address from ConnectInfo.
        req.extensions_mut().insert(ConnectInfo(peer));
        req
    }

    #[test]
    fn untrusted_uses_peer_ip_and_ignores_forwarded_header() {
        let extractor = IpKeyExtractor { trust_proxy: false };
        let peer = SocketAddr::from((Ipv4Addr::new(203, 0, 113, 7), 40000));
        // A spoofed X-Forwarded-For must not change the key: it stays the peer IP.
        let key = extractor
            .extract(&request_with(peer, Some("198.51.100.99")))
            .expect("peer ip");
        assert_eq!(key, IpAddr::from(Ipv4Addr::new(203, 0, 113, 7)));

        // Rotating the forged header does not mint a different bucket.
        let key2 = extractor
            .extract(&request_with(peer, Some("10.10.10.10")))
            .expect("peer ip");
        assert_eq!(key, key2);
    }

    #[test]
    fn trusted_uses_forwarded_header() {
        let extractor = IpKeyExtractor { trust_proxy: true };
        let peer = SocketAddr::from((Ipv4Addr::new(203, 0, 113, 7), 40000));
        let key = extractor
            .extract(&request_with(peer, Some("198.51.100.99")))
            .expect("forwarded ip");
        assert_eq!(key, IpAddr::from(Ipv4Addr::new(198, 51, 100, 99)));
    }
}
