use futures::FutureExt;
use tracing::info;
use zm_api::constant::CONFIG;
use zm_api::error::AppResult;
use zm_api::server::AppServer;
use zm_api::{configure, util};

#[tokio::main]
#[allow(clippy::result_large_err)]
async fn main() -> AppResult<()> {
    // Install the rustls CryptoProvider before any TLS/DTLS usage.
    // Required by rustls 0.23+ (used by webrtc-rs DTLS, axum-server TLS, sqlx).
    zm_api::install_crypto_provider();

    // Initialize the ffmpeg libraries once at startup (idempotent, thread-safe).
    // Snapshot/VOD decode paths rely on registered codecs/demuxers; doing it
    // here means production requests never race first-use registration.
    // REVIEW_FIXES_PLAN §5.2.
    ffmpeg_next::init().ok();

    let _file_appender_guard = configure::tracing::init()?;
    info!("The initialization of Tracing was successful.");
    let config = CONFIG.clone();
    info!("Reading the config file was successful.");
    info!("Create a new server.");
    let server = AppServer::new(config).await?;
    info!("Run the server.");
    util::task::join_all(vec![(true, server.run().boxed())]).await?;
    Ok(())
}
