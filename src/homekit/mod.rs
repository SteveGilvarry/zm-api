//! Apple HomeKit (HAP) accessory bridge for ZoneMinder monitors.
//!
//! HomeKit is not REST: it is the HomeKit Accessory Protocol (HAP), a
//! self-contained TCP server advertised over mDNS/Bonjour with its own pairing
//! (SRP6a → Curve25519 verify → ChaCha20-Poly1305 sessions), TLV8 wire format,
//! accessory database, and SRTP video delivery. This module runs that server
//! alongside the Axum API as a separate background task, exposing a configured
//! monitor as an IP-camera accessory that reuses the existing streaming
//! pipeline (`SourceRouter` for live H.264, `SnapshotService` for JPEGs).
//!
//! Phase 1 scope: pairing + a single live camera (video-only).
//!
//! Submodule layout:
//! - [`config`]   — `[homekit]` settings.
//! - [`tlv8`]     — HAP TLV8 codec.
//! - [`crypto`]   — HKDF-SHA512 + ChaCha20-Poly1305 helpers and HAP constants.
//! - [`session`]  — post-pairing transport framing.
//! - [`pairing`]  — persistent identity + Pair-Setup / Pair-Verify state machines.

pub mod accessory;
pub mod camera;
pub mod config;
pub mod crypto;
pub mod http;
pub mod mdns;
pub mod pairing;
pub mod session;
pub mod tlv8;

pub use config::HomeKitConfig;

use std::net::IpAddr;
use std::sync::{Arc, Mutex};

use base64::Engine as _;
use serde_json::{json, Value};
use tokio::net::TcpListener;
use tracing::{error, info};

use crate::error::{AppError, AppResult};
use crate::streaming::snapshot::SnapshotService;
use crate::streaming::source::SourceRouter;

use camera::stream::{StreamParams, StreamSession};
use camera::{SrtpParams, StreamManager};
use mdns::Advertiser;
use pairing::PairingStore;

/// Dynamic state for the single camera accessory.
#[derive(Default)]
struct CameraState {
    /// Last SetupEndpoints response (base64 TLV) returned on read.
    setup_endpoints_value: String,
    /// True while a stream is active (drives StreamingStatus).
    streaming_in_use: bool,
}

/// The HomeKit accessory bridge: a HAP TCP server + mDNS advertisement exposing
/// one ZoneMinder monitor as an IP camera.
pub struct HomeKitServer {
    config: HomeKitConfig,
    store: Arc<PairingStore>,
    source_router: Arc<SourceRouter>,
    snapshot: Option<Arc<SnapshotService>>,
    advertiser: Mutex<Option<Advertiser>>,
    stream_manager: StreamManager,
    camera_state: Mutex<CameraState>,
    supported_video: String,
    supported_audio: String,
    supported_rtp: String,
}

impl HomeKitServer {
    /// Build the server, loading (or creating) the persistent accessory identity.
    pub fn new(
        config: HomeKitConfig,
        source_router: Arc<SourceRouter>,
        snapshot: Option<Arc<SnapshotService>>,
    ) -> AppResult<Arc<Self>> {
        let store = Arc::new(PairingStore::load_or_create(&config.persist_dir)?);
        Ok(Arc::new(Self {
            config,
            store,
            source_router,
            snapshot,
            advertiser: Mutex::new(None),
            stream_manager: StreamManager::new(),
            camera_state: Mutex::new(CameraState::default()),
            supported_video: camera::supported_video(),
            supported_audio: camera::supported_audio(),
            supported_rtp: camera::supported_rtp(),
        }))
    }

    /// Run the HAP server until the process exits.
    pub async fn run(self: Arc<Self>) -> AppResult<()> {
        let advertiser = Advertiser::start(
            &self.config.name,
            self.store.device_id(),
            "zm-api",
            &self.config.setup_id,
            self.config.port,
            self.store.is_paired(),
        )?;
        *self.advertiser.lock().unwrap() = Some(advertiser);

        let addr = format!("0.0.0.0:{}", self.config.port);
        let listener = TcpListener::bind(&addr)
            .await
            .map_err(|e| AppError::UnknownError(anyhow::anyhow!("homekit: bind {addr}: {e}")))?;
        info!(
            "HomeKit bridge '{}' listening on {addr} (device {}, pin {})",
            self.config.name,
            self.store.device_id(),
            self.config.pin,
        );

        loop {
            match listener.accept().await {
                Ok((stream, _peer)) => {
                    let _ = stream.set_nodelay(true);
                    let server = Arc::clone(&self);
                    tokio::spawn(http::serve_connection(server, stream));
                }
                Err(e) => {
                    error!("homekit accept error: {e}");
                }
            }
        }
    }

    // --- Accessors used by the HTTP layer. ---

    pub(crate) fn advertiser(&self) -> std::sync::MutexGuard<'_, Option<Advertiser>> {
        self.advertiser.lock().unwrap()
    }

    pub(crate) fn snapshot(&self) -> &Option<Arc<SnapshotService>> {
        &self.snapshot
    }

    /// Build the `/accessories` document with the live supported-config blobs.
    pub(crate) fn accessories_json(&self) -> Value {
        accessory::accessories_json(
            &self.config.name,
            &self.config.name,
            "zm-api",
            self.store.device_id(),
            "3.0.0",
            &self.supported_video,
            &self.supported_audio,
            &self.supported_rtp,
        )
    }

    /// Return the value for a characteristic read (camera dynamic + static).
    pub(crate) fn read_characteristic(&self, aid: u64, ch_iid: u64) -> Value {
        use accessory::iid::*;
        if aid != accessory::CAMERA_AID {
            return Value::Null;
        }
        match ch_iid {
            CAMERA_SETUP_ENDPOINTS => {
                json!(self.camera_state.lock().unwrap().setup_endpoints_value)
            }
            CAMERA_STREAMING_STATUS => {
                // TLV { 1: status }, where 1 = "In Use" else 0 = "Available".
                let in_use = self.camera_state.lock().unwrap().streaming_in_use;
                let mut w = tlv8::TlvWriter::new();
                w.push_u8(1u8, if in_use { 1 } else { 0 });
                json!(base64::engine::general_purpose::STANDARD.encode(w.as_bytes()))
            }
            CAMERA_SUPPORTED_VIDEO => json!(self.supported_video),
            CAMERA_SUPPORTED_AUDIO => json!(self.supported_audio),
            CAMERA_SUPPORTED_RTP => json!(self.supported_rtp),
            CAMERA_SELECTED_RTP => json!(""),
            _ => Value::Null,
        }
    }

    /// Handle a SetupEndpoints write: negotiate SRTP and prepare a stream session.
    pub(crate) async fn handle_setup_endpoints(&self, accessory_ip: IpAddr, tlv: &[u8]) {
        let Some(req) = camera::parse_setup_endpoints(tlv) else {
            return;
        };

        // Accessory-generated SRTP keys for the outbound video (sender keys).
        let mut key = vec![0u8; 16];
        let mut salt = vec![0u8; 14];
        crypto::fill_random(&mut key);
        crypto::fill_random(&mut salt);
        let accessory_video = SrtpParams {
            crypto_suite: 0,
            master_key: key,
            master_salt: salt,
        };
        // Audio keys (unused in Phase 1 but required in the response).
        let mut akey = vec![0u8; 16];
        let mut asalt = vec![0u8; 14];
        crypto::fill_random(&mut akey);
        crypto::fill_random(&mut asalt);
        let accessory_audio = SrtpParams {
            crypto_suite: 0,
            master_key: akey,
            master_salt: asalt,
        };

        let mut ssrc_bytes = [0u8; 4];
        crypto::fill_random(&mut ssrc_bytes);
        let video_ssrc = u32::from_le_bytes(ssrc_bytes);

        // Bind the accessory UDP socket now so its port is stable for the response.
        let socket = match std::net::UdpSocket::bind("0.0.0.0:0") {
            Ok(s) => s,
            Err(e) => {
                error!("homekit: udp bind failed: {e}");
                return;
            }
        };
        let accessory_port = socket.local_addr().map(|a| a.port()).unwrap_or(0);

        let resp = camera::build_setup_response(
            &req.session_id,
            accessory_ip,
            accessory_port,
            accessory_port,
            &accessory_video,
            &accessory_audio,
            video_ssrc,
            0,
        );
        self.camera_state.lock().unwrap().setup_endpoints_value =
            base64::engine::general_purpose::STANDARD.encode(&resp);

        let params = StreamParams {
            monitor_id: self.config.monitor_id,
            controller_ip: req.controller_ip,
            video_port: req.video_port,
            video_ssrc,
            video_srtp: accessory_video,
        };
        let session = Arc::new(StreamSession::new(
            params,
            socket,
            Arc::clone(&self.source_router),
        ));
        self.stream_manager.insert(req.session_id, session);
    }

    /// Handle a SelectedRTPStreamConfiguration write: start/stop the stream.
    pub(crate) async fn handle_selected_rtp(&self, tlv: &[u8]) {
        let Some((session_id, command)) = camera::parse_selected_rtp(tlv) else {
            return;
        };
        use camera::StreamCommand::*;
        match command {
            Start | Resume => {
                if let Some(session) = self.stream_manager.get(&session_id) {
                    session.start().await;
                    self.camera_state.lock().unwrap().streaming_in_use = true;
                }
            }
            End | Suspend => {
                if let Some(session) = self.stream_manager.remove(&session_id) {
                    session.stop().await;
                }
                self.camera_state.lock().unwrap().streaming_in_use = false;
            }
            _ => {}
        }
    }
}
