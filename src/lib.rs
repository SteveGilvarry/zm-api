/// Install the rustls `CryptoProvider` process-wide.
///
/// Must be called before any TLS/DTLS usage (WebRTC DTLS, axum-server TLS,
/// sqlx TLS connections). Safe to call multiple times — subsequent calls are
/// ignored if a provider is already installed.
pub fn install_crypto_provider() {
    let _ = rustls::crypto::ring::default_provider().install_default();
}

pub mod client;
pub mod configure;
pub mod constant;
pub mod daemon;
pub mod dto;
pub mod entity;
pub mod enum_traits;
pub mod error;
pub mod handlers;
pub mod migration;
pub mod mse_client;
pub mod mse_socket_client;
pub mod ptz;
pub mod repo;
pub mod routes;
pub mod server;
pub mod service;
pub mod streaming;
pub mod util;
pub mod webrtc_ffi;
pub mod zm_shm;
