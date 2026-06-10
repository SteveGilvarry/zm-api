//! WebRTC live streaming from FIFO sources
//!
//! This module provides WebRTC streaming using the webrtc-rs crate.
//! It uses `TrackLocalStaticSample` which handles RTP packetization,
//! SRTP encryption, and interceptor integration automatically via
//! `write_sample()`.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use dashmap::DashMap;
use interceptor::registry::Registry;
use tokio::sync::RwLock;
use tracing::{debug, info};
use uuid::Uuid;
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::setting_engine::SettingEngine;
use webrtc::api::APIBuilder;
use webrtc::dtls_transport::dtls_transport_state::RTCDtlsTransportState;
use webrtc::ice_transport::ice_candidate::RTCIceCandidateInit;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::peer_connection::RTCPeerConnection;
use webrtc::rtp_transceiver::rtp_codec::RTCRtpCodecCapability;
use webrtc::rtp_transceiver::rtp_transceiver_direction::RTCRtpTransceiverDirection;
use webrtc::rtp_transceiver::RTCRtpTransceiverInit;
use webrtc::track::track_local::track_local_static_sample::TrackLocalStaticSample;
use webrtc::track::track_local::TrackLocal;
use webrtc_media::Sample;

use crate::streaming::source::{h264_nal_type, slice_starts_picture, FifoPacket, VideoCodec};

// Re-export for consumers that imported from here
pub use crate::streaming::source::extract_profile_level_id;

// ============================================================================
// Access Unit Assembler
// ============================================================================

/// Check if an H.264 NAL unit is a VCL (Video Coding Layer) unit — i.e. a
/// coded slice that is part of a picture.  Types 1-5 are VCL.
fn is_h264_vcl(nal_type: u8) -> bool {
    (1..=5).contains(&nal_type)
}

/// Assembles individual NAL units into complete Access Units (frames).
///
/// The FIFO reader emits one NAL unit per `FifoPacket`. WebRTC's H264
/// payloader works correctly with Annex B data containing multiple NALs,
/// but each `write_sample()` call gets a distinct RTP timestamp. For
/// correct decoding, all NALs belonging to the same picture must share
/// one timestamp. This assembler buffers non-VCL NALs (SPS, PPS, SEI)
/// and emits a complete AU when it sees the first VCL NAL of the *next*
/// picture — a slice whose `first_mb_in_slice` is 0. Continuation slices
/// of a multi-slice picture (`first_mb_in_slice > 0`) are kept in the same
/// AU so the RTP packetizer assigns the whole frame one timestamp and a
/// single trailing marker bit.
pub struct AccessUnitAssembler {
    /// Buffered Annex B data for the current access unit being assembled
    buf: Vec<u8>,
    /// Whether we have seen at least one VCL NAL in the current AU
    has_vcl: bool,
    /// Timestamp of the most recent VCL NAL added
    timestamp_us: i64,
    /// Whether the current AU contains a keyframe
    is_keyframe: bool,
    /// When true, drop assembled AUs until a keyframe is seen.
    /// Ensures the stream starts with SPS+PPS+IDR for decoder init.
    needs_keyframe: bool,
}

impl Default for AccessUnitAssembler {
    fn default() -> Self {
        Self::new()
    }
}

impl AccessUnitAssembler {
    pub fn new() -> Self {
        Self {
            buf: Vec::with_capacity(128 * 1024),
            has_vcl: false,
            timestamp_us: 0,
            is_keyframe: false,
            needs_keyframe: true,
        }
    }

    /// Feed a NAL unit (FifoPacket) into the assembler.
    ///
    /// Returns `Some(assembled_au)` when a complete access unit is ready
    /// to be sent to the WebRTC track. The returned bytes are Annex B
    /// formatted (start codes included), suitable for the H264 payloader.
    pub fn push(&mut self, packet: &FifoPacket) -> Option<AssembledAccessUnit> {
        if packet.data.is_empty() {
            return None;
        }

        let nal_type = match h264_nal_type(&packet.data) {
            Some(t) => t,
            None => {
                // Not a recognizable NAL — append to current buffer and continue
                self.buf.extend_from_slice(&packet.data);
                return None;
            }
        };

        if self.needs_keyframe {
            info!(
                "AU assembler: NAL type={}, is_keyframe={}, has_vcl={}, buf={}B",
                nal_type,
                packet.is_keyframe,
                self.has_vcl,
                self.buf.len()
            );
        }

        let vcl = is_h264_vcl(nal_type);
        // SPS (7), PPS (8) and AUD (9) all delimit the start of a new access
        // unit. Flush any buffered VCL data when one arrives so it begins a
        // fresh AU — SPS/PPS group with the following keyframe, not with the
        // preceding slice.
        let starts_new_au = nal_type == 7 || nal_type == 8 || nal_type == 9;

        if starts_new_au && self.has_vcl {
            let au = self.flush();
            self.buf.extend_from_slice(&packet.data);
            return self.filter_needs_keyframe(au);
        }

        if vcl && self.has_vcl {
            // A VCL NAL only completes the current access unit when it begins
            // a new primary coded picture (`first_mb_in_slice == 0`). The
            // remaining slices of a multi-slice picture — 4K cameras emit
            // several slice NALs per frame — carry `first_mb_in_slice > 0` and
            // belong to the SAME access unit. They must be appended, not split
            // off, so the H264 RTP packetizer keeps every slice of the picture
            // under one RTP timestamp with a single trailing marker bit.
            if slice_starts_picture(&packet.data, packet.codec) {
                let au = self.flush();

                // Start a new AU with this slice.
                self.buf.extend_from_slice(&packet.data);
                self.has_vcl = true;
                self.timestamp_us = packet.timestamp_us;
                self.is_keyframe = packet.is_keyframe;

                return self.filter_needs_keyframe(au);
            }
            // Continuation slice of the current picture — fall through to
            // append it to the access unit already being assembled.
        }

        // Append to current AU
        self.buf.extend_from_slice(&packet.data);
        if vcl {
            self.has_vcl = true;
            self.timestamp_us = packet.timestamp_us;
            if packet.is_keyframe {
                self.is_keyframe = true;
            }
        }
        None
    }

    /// Drop non-keyframe AUs while `needs_keyframe` is set, ensuring the
    /// stream starts with SPS+PPS+IDR for correct decoder initialisation.
    fn filter_needs_keyframe(
        &mut self,
        au: Option<AssembledAccessUnit>,
    ) -> Option<AssembledAccessUnit> {
        match au {
            Some(au) if self.needs_keyframe && !au.is_keyframe => {
                debug!(
                    "Dropping non-keyframe AU ({} bytes) while waiting for keyframe",
                    au.data.len()
                );
                None
            }
            Some(au) => {
                self.needs_keyframe = false;
                Some(au)
            }
            None => None,
        }
    }

    /// Mark the assembler as having received a keyframe externally.
    ///
    /// Call this after injecting a cached keyframe (SPS+PPS+IDR) into the
    /// WebRTC track so that subsequent P-frames are not dropped.
    pub fn clear_needs_keyframe(&mut self) {
        self.needs_keyframe = false;
    }

    /// Flush the current buffer as a complete access unit.
    fn flush(&mut self) -> Option<AssembledAccessUnit> {
        if self.buf.is_empty() {
            return None;
        }

        let data = std::mem::take(&mut self.buf);
        let au = AssembledAccessUnit {
            data,
            timestamp_us: self.timestamp_us,
            is_keyframe: self.is_keyframe,
        };
        self.has_vcl = false;
        self.is_keyframe = false;
        Some(au)
    }
}

/// A complete access unit (one video frame) ready for RTP packetization
pub struct AssembledAccessUnit {
    /// Annex B formatted data (may contain multiple NAL units with start codes)
    pub data: Vec<u8>,
    /// Timestamp in microseconds
    pub timestamp_us: i64,
    /// Whether this AU is a keyframe
    pub is_keyframe: bool,
}

// ============================================================================
// WebRTC Session
// ============================================================================

/// WebRTC session state
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum WebRtcSessionState {
    New,
    Connecting,
    Connected,
    Disconnected,
    Failed,
}

/// WebRTC session statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct WebRtcSessionStats {
    pub session_id: String,
    pub monitor_id: u32,
    pub state: WebRtcSessionState,
    pub packets_sent: u64,
    pub bytes_sent: u64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// A WebRTC streaming session
pub struct WebRtcLiveSession {
    pub id: Uuid,
    pub monitor_id: u32,
    pub state: RwLock<WebRtcSessionState>,
    pub peer_connection: Arc<RTCPeerConnection>,
    pub video_track: Arc<TrackLocalStaticSample>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    packets_sent: AtomicU64,
    bytes_sent: AtomicU64,
}

impl WebRtcLiveSession {
    /// Get session statistics
    pub async fn stats(&self) -> WebRtcSessionStats {
        WebRtcSessionStats {
            session_id: self.id.to_string(),
            monitor_id: self.monitor_id,
            state: *self.state.read().await,
            packets_sent: self.packets_sent.load(Ordering::Relaxed),
            bytes_sent: self.bytes_sent.load(Ordering::Relaxed),
            created_at: self.created_at,
        }
    }

    /// Update session state
    pub async fn set_state(&self, state: WebRtcSessionState) {
        *self.state.write().await = state;
    }

    /// Get current state
    pub async fn get_state(&self) -> WebRtcSessionState {
        *self.state.read().await
    }

    /// Write a complete access unit (one video frame) to the WebRTC track.
    ///
    /// The `data` should be Annex B formatted and may contain multiple NAL
    /// units (e.g. SPS + PPS + IDR for a keyframe). The H264 payloader
    /// splits them and assigns the same RTP timestamp to all.
    ///
    /// Uses `TrackLocalStaticSample::write_sample()` which handles RTP
    /// packetization, fragmentation, SRTP encryption, and interceptor
    /// integration (RTCP feedback, NACK, etc.) automatically.
    pub async fn write_access_unit(&self, au: &AssembledAccessUnit) -> Result<(), WebRtcLiveError> {
        if au.data.is_empty() {
            return Ok(());
        }

        // ~30fps default; the packetizer uses duration to compute RTP timestamps
        let duration = Duration::from_millis(33);

        let sample = Sample {
            data: bytes::Bytes::copy_from_slice(&au.data),
            duration,
            ..Default::default()
        };

        self.video_track
            .write_sample(&sample)
            .await
            .map_err(|e| WebRtcLiveError::WebRtcError(format!("Failed to write sample: {}", e)))?;

        self.packets_sent.fetch_add(1, Ordering::Relaxed);
        self.bytes_sent
            .fetch_add(au.data.len() as u64, Ordering::Relaxed);

        Ok(())
    }
}

/// WebRTC live streaming errors
#[derive(Debug, thiserror::Error)]
pub enum WebRtcLiveError {
    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Session already exists for monitor {0}")]
    SessionExists(u32),

    #[error("WebRTC error: {0}")]
    WebRtcError(String),

    #[error("Codec not supported: {0}")]
    UnsupportedCodec(String),

    #[error("No video track available")]
    NoVideoTrack,
}

/// Configuration for WebRTC live streaming
#[derive(Debug, Clone)]
pub struct WebRtcLiveConfig {
    pub stun_servers: Vec<String>,
    pub turn_server: Option<String>,
    pub turn_username: Option<String>,
    pub turn_password: Option<String>,
    pub max_sessions: usize,
}

impl Default for WebRtcLiveConfig {
    fn default() -> Self {
        Self {
            stun_servers: vec![
                "stun:stun.l.google.com:19302".to_string(),
                "stun:stun1.l.google.com:19302".to_string(),
            ],
            turn_server: None,
            turn_username: None,
            turn_password: None,
            max_sessions: 100,
        }
    }
}

impl WebRtcLiveConfig {
    /// Build a live-streaming config from the application's `[streaming.webrtc]`
    /// settings, so the operator's configured STUN/TURN servers actually take
    /// effect on the live WS path (it previously always used the hardcoded
    /// Google STUN defaults). An **empty** `stun_servers` list is honored
    /// verbatim — on a LAN-only deployment that is the right setting: host
    /// candidates alone let ICE gathering complete in milliseconds instead of
    /// stalling on the ~5s STUN gather timeout when Google STUN is unreachable.
    /// See REVIEW_FIXES_PLAN §2.2.
    pub fn from_webrtc_config(cfg: &crate::configure::streaming::WebRtcConfig) -> Self {
        let (turn_server, turn_username, turn_password) = match &cfg.turn {
            Some(turn) if turn.enabled => (
                Some(turn.server.clone()),
                Some(turn.username.clone()),
                Some(turn.password.clone()),
            ),
            _ => (None, None, None),
        };
        Self {
            stun_servers: cfg.stun_servers.clone(),
            turn_server,
            turn_username,
            turn_password,
            max_sessions: cfg.max_connections as usize,
        }
    }
}

/// Manager for WebRTC live streaming sessions
pub struct WebRtcLiveManager {
    config: WebRtcLiveConfig,
    sessions: DashMap<String, Arc<RwLock<WebRtcLiveSession>>>,
    /// Map from monitor_id to list of session IDs
    monitor_sessions: DashMap<u32, Vec<String>>,
}

/// The H.264 `profile-level-id` values to advertise in the SDP offer.
///
/// These are **not** the camera's native profile. The server is a pure
/// pass-through — it forwards the camera's real H.264 NAL units unchanged —
/// and in practice every WebRTC decoder (Safari's VideoToolbox, Chromium's
/// decoder) decodes whatever NAL units actually arrive regardless of the
/// negotiated `profile-level-id`. So the offer advertises only the profiles
/// every browser is guaranteed to accept:
///
/// * `42e01f` — Constrained Baseline, Level 3.1. The universal baseline; every
///   WebRTC implementation accepts it.
/// * `640c1f` — Constrained High, Level 3.1.
///
/// Safari accepts *only* these two. It rejects Main (`4d…`) outright, and
/// plain non-constrained High (`6400…`) is not the same as Constrained High
/// (`640c…`). Advertising the camera's native Main/High profile therefore
/// made Safari answer with a rejected `m=video 0` line, after which the server
/// errored "codec is not supported by remote" and dropped the connection.
///
/// `level-asymmetry-allowed=1` (set in the fmtp line) lets the bytes the
/// server actually sends stay at the camera's real level — 5.1 for 4K — even
/// though the negotiated level is the conservative 3.1. Profile is not subject
/// to level asymmetry, which is exactly why the offered *profiles* must be
/// ones the decoder accepts; it then decodes the real Main/High bitstream.
fn h264_offer_profile_level_ids() -> Vec<String> {
    vec!["42e01f".to_string(), "640c1f".to_string()]
}

impl WebRtcLiveManager {
    /// Create a new WebRTC live manager
    pub fn new(config: WebRtcLiveConfig) -> Self {
        Self {
            config,
            sessions: DashMap::new(),
            monitor_sessions: DashMap::new(),
        }
    }

    /// Create a new WebRTC session for a monitor
    ///
    /// Builds the webrtc-rs API with interceptor registry and setting engine,
    /// waits for ICE gathering to complete, and returns the full SDP offer
    /// with embedded ICE candidates.
    /// Create a new WebRTC session for a monitor.
    ///
    /// `profile_level_id` is the 6-hex-char value extracted from the H.264
    /// SPS NAL (e.g. `"4d0033"` for Main Profile Level 5.1).  When `None`,
    /// defaults to `"4d0033"` which covers most IP cameras.
    pub async fn create_session(
        &self,
        monitor_id: u32,
        codec: VideoCodec,
        profile_level_id: Option<&str>,
    ) -> Result<
        (
            String,
            RTCSessionDescription,
            tokio::sync::watch::Receiver<RTCPeerConnectionState>,
        ),
        WebRtcLiveError,
    > {
        // Check session limit
        if self.sessions.len() >= self.config.max_sessions {
            return Err(WebRtcLiveError::WebRtcError(
                "Maximum sessions reached".to_string(),
            ));
        }

        // Create media engine with codec support
        let mut media_engine = MediaEngine::default();

        // Build the codec capabilities to advertise. For H.264 the offer
        // advertises browser-universal pass-through profiles, *not* the
        // camera's native one — see `h264_offer_profile_level_ids`.
        let codec_capabilities: Vec<RTCRtpCodecCapability> = match codec {
            VideoCodec::H264 => {
                debug!(
                    "Monitor {monitor_id}: camera H.264 profile {}; advertising \
                     pass-through profiles 42e01f/640c1f for browser compatibility",
                    profile_level_id.unwrap_or("unknown")
                );
                h264_offer_profile_level_ids()
                    .into_iter()
                    .map(|variant| RTCRtpCodecCapability {
                        mime_type: "video/H264".to_string(),
                        clock_rate: 90000,
                        channels: 0,
                        sdp_fmtp_line: format!(
                            "level-asymmetry-allowed=1;packetization-mode=1;profile-level-id={variant}"
                        ),
                        rtcp_feedback: vec![],
                    })
                    .collect()
            }
            VideoCodec::H265 => vec![RTCRtpCodecCapability {
                mime_type: "video/H265".to_string(),
                clock_rate: 90000,
                channels: 0,
                sdp_fmtp_line: String::new(),
                rtcp_feedback: vec![],
            }],
            VideoCodec::Unknown => {
                return Err(WebRtcLiveError::UnsupportedCodec("Unknown".to_string()));
            }
        };

        // The pass-through track binds by mime type, so any advertised
        // capability works; use the first. Negotiation settles on one of the
        // offered profiles, but the bytes we send are always the camera's
        // real bitstream — the decoder handles it regardless of profile.
        let codec_capability = codec_capabilities[0].clone();

        // Payload types 96+ are the dynamic range; one per advertised codec.
        for (i, capability) in codec_capabilities.into_iter().enumerate() {
            media_engine
                .register_codec(
                    webrtc::rtp_transceiver::rtp_codec::RTCRtpCodecParameters {
                        capability,
                        payload_type: 96 + i as u8,
                        ..Default::default()
                    },
                    webrtc::rtp_transceiver::rtp_codec::RTPCodecType::Video,
                )
                .map_err(|e| WebRtcLiveError::WebRtcError(e.to_string()))?;
        }

        // Register default interceptors (RTCP feedback, NACK, etc.)
        let mut registry = Registry::new();
        registry = register_default_interceptors(registry, &mut media_engine)
            .map_err(|e| WebRtcLiveError::WebRtcError(e.to_string()))?;

        // Configure ICE/DTLS settings
        let mut setting_engine = SettingEngine::default();
        setting_engine.set_lite(false);
        setting_engine.set_ice_timeouts(
            Some(std::time::Duration::from_secs(5)),
            Some(std::time::Duration::from_secs(10)),
            Some(std::time::Duration::from_millis(200)),
        );

        // Build API with full interceptor + settings stack
        let api = APIBuilder::new()
            .with_media_engine(media_engine)
            .with_interceptor_registry(registry)
            .with_setting_engine(setting_engine)
            .build();

        // Create ICE servers configuration
        let mut ice_servers = vec![];

        for stun_url in &self.config.stun_servers {
            ice_servers.push(RTCIceServer {
                urls: vec![stun_url.clone()],
                ..Default::default()
            });
        }

        if let Some(turn_url) = &self.config.turn_server {
            ice_servers.push(RTCIceServer {
                urls: vec![turn_url.clone()],
                username: self.config.turn_username.clone().unwrap_or_default(),
                credential: self.config.turn_password.clone().unwrap_or_default(),
            });
        }

        let rtc_config = RTCConfiguration {
            ice_servers,
            ..Default::default()
        };

        // Create peer connection
        let peer_connection = api
            .new_peer_connection(rtc_config)
            .await
            .map_err(|e| WebRtcLiveError::WebRtcError(e.to_string()))?;

        // Watch channel for peer connection state — lets the handler wait for
        // DTLS completion ("connected" state) before streaming media.
        // Wrapped in Arc because both the peer connection state callback and
        // the DTLS transport state callback need to send on it.
        let (pc_state_tx, pc_state_rx) = tokio::sync::watch::channel(RTCPeerConnectionState::New);
        let pc_state_tx = Arc::new(pc_state_tx);

        // Register connection state callbacks for diagnostics + state channel
        let tx_for_pc = Arc::clone(&pc_state_tx);
        let pc_monitor_id = monitor_id;
        peer_connection.on_peer_connection_state_change(Box::new(move |state| {
            info!(
                monitor_id = pc_monitor_id,
                state = ?state,
                "WebRTC peer connection state changed"
            );
            let _ = tx_for_pc.send(state);
            Box::pin(async move {})
        }));

        // Workaround for webrtc-rs 0.12 bug: the library does NOT register a
        // DTLS transport state change callback to update the aggregate peer
        // connection state. When DTLS completes after ICE, the peer connection
        // state stays at Connecting forever. We register our own DTLS callback
        // and synthesize the Connected state (DTLS Connected implies ICE is
        // already connected since DTLS runs over ICE).
        let tx_for_dtls = Arc::clone(&pc_state_tx);
        let dtls_monitor_id = monitor_id;
        peer_connection
            .dtls_transport()
            .on_state_change(Box::new(move |state| {
                info!(
                    monitor_id = dtls_monitor_id,
                    state = ?state,
                    "WebRTC DTLS transport state changed"
                );
                match state {
                    RTCDtlsTransportState::Connected => {
                        let _ = tx_for_dtls.send(RTCPeerConnectionState::Connected);
                    }
                    RTCDtlsTransportState::Failed => {
                        let _ = tx_for_dtls.send(RTCPeerConnectionState::Failed);
                    }
                    _ => {}
                }
                Box::pin(async move {})
            }));

        let ice_monitor_id = monitor_id;
        peer_connection.on_ice_connection_state_change(Box::new(move |state| {
            info!(
                monitor_id = ice_monitor_id,
                state = ?state,
                "WebRTC ICE connection state changed"
            );
            Box::pin(async move {})
        }));

        // Create video track using TrackLocalStaticSample which handles
        // RTP packetization, SRTP, and interceptors automatically
        let video_track = Arc::new(TrackLocalStaticSample::new(
            codec_capability,
            "video".to_string(),
            format!("zm-live-{}", monitor_id),
        ));

        // Add track with sendonly direction (one-way surveillance stream)
        peer_connection
            .add_transceiver_from_track(
                Arc::clone(&video_track) as Arc<dyn TrackLocal + Send + Sync>,
                Some(RTCRtpTransceiverInit {
                    direction: RTCRtpTransceiverDirection::Sendonly,
                    send_encodings: vec![],
                }),
            )
            .await
            .map_err(|e| WebRtcLiveError::WebRtcError(e.to_string()))?;

        // Create offer
        let offer = peer_connection
            .create_offer(None)
            .await
            .map_err(|e| WebRtcLiveError::WebRtcError(e.to_string()))?;

        // Set local description (starts ICE gathering)
        peer_connection
            .set_local_description(offer)
            .await
            .map_err(|e| WebRtcLiveError::WebRtcError(e.to_string()))?;

        // Wait for ICE gathering to complete so the offer includes all candidates
        let mut gather_complete = peer_connection.gathering_complete_promise().await;
        let _ = gather_complete.recv().await;

        // Get the complete local description (with gathered ICE candidates)
        let complete_offer = peer_connection.local_description().await.ok_or_else(|| {
            WebRtcLiveError::WebRtcError(
                "No local description available after ICE gathering".to_string(),
            )
        })?;

        info!(
            "ICE gathering complete for monitor {}, offer has candidates",
            monitor_id
        );

        // Create session
        let session_id = Uuid::new_v4();
        let session = WebRtcLiveSession {
            id: session_id,
            monitor_id,
            state: RwLock::new(WebRtcSessionState::New),
            peer_connection: Arc::new(peer_connection),
            video_track,
            created_at: chrono::Utc::now(),
            packets_sent: AtomicU64::new(0),
            bytes_sent: AtomicU64::new(0),
        };

        let session_id_str = session_id.to_string();
        self.sessions
            .insert(session_id_str.clone(), Arc::new(RwLock::new(session)));

        // Track session by monitor
        self.monitor_sessions
            .entry(monitor_id)
            .or_default()
            .push(session_id_str.clone());

        info!(
            "Created WebRTC session {} for monitor {}",
            session_id_str, monitor_id
        );

        Ok((session_id_str, complete_offer, pc_state_rx))
    }

    /// Handle answer from client
    pub async fn set_answer(
        &self,
        session_id: &str,
        answer: RTCSessionDescription,
    ) -> Result<(), WebRtcLiveError> {
        let session_lock = self
            .sessions
            .get(session_id)
            .ok_or_else(|| WebRtcLiveError::SessionNotFound(session_id.to_string()))?;

        let session = session_lock.read().await;
        session
            .peer_connection
            .set_remote_description(answer)
            .await
            .map_err(|e| WebRtcLiveError::WebRtcError(e.to_string()))?;

        // Update state to connecting
        session.set_state(WebRtcSessionState::Connecting).await;

        info!("Set answer for WebRTC session {}", session_id);
        Ok(())
    }

    /// Add an ICE candidate to a session
    pub async fn add_ice_candidate(
        &self,
        session_id: &str,
        candidate: &str,
        sdp_mid: Option<String>,
        sdp_mline_index: Option<u16>,
    ) -> Result<(), WebRtcLiveError> {
        let session_lock = self
            .sessions
            .get(session_id)
            .ok_or_else(|| WebRtcLiveError::SessionNotFound(session_id.to_string()))?;

        let session = session_lock.read().await;

        let ice_candidate = RTCIceCandidateInit {
            candidate: candidate.to_string(),
            sdp_mid,
            sdp_mline_index,
            username_fragment: None,
        };

        session
            .peer_connection
            .add_ice_candidate(ice_candidate)
            .await
            .map_err(|e| WebRtcLiveError::WebRtcError(e.to_string()))?;

        debug!("Added ICE candidate for WebRTC session {}", session_id);
        Ok(())
    }

    /// Write a complete access unit to a session's video track
    pub async fn write_access_unit(
        &self,
        session_id: &str,
        au: &AssembledAccessUnit,
    ) -> Result<(), WebRtcLiveError> {
        let session_lock = self
            .sessions
            .get(session_id)
            .ok_or_else(|| WebRtcLiveError::SessionNotFound(session_id.to_string()))?;

        let session = session_lock.read().await;
        session.write_access_unit(au).await
    }

    /// Get stats for a session
    pub async fn get_session_stats(
        &self,
        session_id: &str,
    ) -> Result<WebRtcSessionStats, WebRtcLiveError> {
        let session_lock = self
            .sessions
            .get(session_id)
            .ok_or_else(|| WebRtcLiveError::SessionNotFound(session_id.to_string()))?;

        let session = session_lock.read().await;
        Ok(session.stats().await)
    }

    /// Get session by ID
    pub fn get_session(&self, session_id: &str) -> Option<Arc<RwLock<WebRtcLiveSession>>> {
        self.sessions.get(session_id).map(|s| s.clone())
    }

    /// Remove a session
    pub async fn remove_session(&self, session_id: &str) -> Result<(), WebRtcLiveError> {
        let session_lock = self
            .sessions
            .remove(session_id)
            .ok_or_else(|| WebRtcLiveError::SessionNotFound(session_id.to_string()))?;

        let session = session_lock.1.read().await;

        // Close peer connection
        let _ = session.peer_connection.close().await;

        // Remove from monitor sessions
        if let Some(mut sessions) = self.monitor_sessions.get_mut(&session.monitor_id) {
            sessions.retain(|id| id != session_id);
        }

        info!("Removed WebRTC session {}", session_id);
        Ok(())
    }

    /// Get all sessions for a monitor
    pub fn get_monitor_sessions(&self, monitor_id: u32) -> Vec<String> {
        self.monitor_sessions
            .get(&monitor_id)
            .map(|v| v.clone())
            .unwrap_or_default()
    }

    /// Get session count
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    /// List all sessions
    pub fn list_sessions(&self) -> Vec<String> {
        self.sessions.iter().map(|r| r.key().clone()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webrtc_live_config_default() {
        let config = WebRtcLiveConfig::default();
        assert_eq!(config.stun_servers.len(), 2);
        assert!(config.turn_server.is_none());
        assert_eq!(config.max_sessions, 100);
    }

    #[test]
    fn from_webrtc_config_honors_empty_stun_list() {
        // REVIEW_FIXES_PLAN §2.2: an empty STUN list must pass through verbatim
        // (LAN-only deployments rely on this to skip the STUN gather stall).
        let cfg = crate::configure::streaming::WebRtcConfig {
            stun_servers: vec![],
            max_connections: 7,
            turn: None,
            ..Default::default()
        };
        let live = WebRtcLiveConfig::from_webrtc_config(&cfg);
        assert!(live.stun_servers.is_empty());
        assert!(live.turn_server.is_none());
        assert_eq!(live.max_sessions, 7);
    }

    #[test]
    fn from_webrtc_config_maps_stun_and_enabled_turn() {
        let cfg = crate::configure::streaming::WebRtcConfig {
            stun_servers: vec!["stun:lan.local:3478".to_string()],
            turn: Some(crate::configure::streaming::TurnConfig {
                enabled: true,
                server: "turn:lan.local:3478".to_string(),
                username: "u".to_string(),
                password: "p".to_string(),
            }),
            ..Default::default()
        };
        let live = WebRtcLiveConfig::from_webrtc_config(&cfg);
        assert_eq!(live.stun_servers, vec!["stun:lan.local:3478".to_string()]);
        assert_eq!(live.turn_server.as_deref(), Some("turn:lan.local:3478"));
        assert_eq!(live.turn_username.as_deref(), Some("u"));
        assert_eq!(live.turn_password.as_deref(), Some("p"));
    }

    #[test]
    fn from_webrtc_config_ignores_disabled_turn() {
        // A present-but-disabled TURN block must not be advertised.
        let cfg = crate::configure::streaming::WebRtcConfig {
            turn: Some(crate::configure::streaming::TurnConfig {
                enabled: false,
                server: "turn:nope:3478".to_string(),
                username: "u".to_string(),
                password: "p".to_string(),
            }),
            ..Default::default()
        };
        let live = WebRtcLiveConfig::from_webrtc_config(&cfg);
        assert!(live.turn_server.is_none());
        assert!(live.turn_username.is_none());
        assert!(live.turn_password.is_none());
    }

    #[test]
    fn test_session_state_serialize() {
        let state = WebRtcSessionState::Connected;
        let json = serde_json::to_string(&state).unwrap();
        assert_eq!(json, "\"connected\"");
    }

    #[test]
    fn test_manager_creation() {
        let config = WebRtcLiveConfig::default();
        let manager = WebRtcLiveManager::new(config);
        assert_eq!(manager.session_count(), 0);
    }

    // --- AccessUnitAssembler tests ---

    /// Build a single-NAL `FifoPacket`. For VCL slice NAL types a slice-header
    /// byte with `first_mb_in_slice == 0` (MSB set) is inserted right after the
    /// NAL header, so the packet reads as the *start* of a coded picture. Use
    /// `make_continuation_slice` for slices that continue a multi-slice picture.
    fn make_nal(nal_type: u8, extra: &[u8]) -> FifoPacket {
        let raw = nal_type & 0x1F;
        let is_keyframe = raw == 5; // IDR
        let mut data = vec![0x00, 0x00, 0x00, 0x01, nal_type];
        if (1..=5).contains(&raw) {
            // first_mb_in_slice = 0 → Exp-Golomb single `1` bit (MSB set).
            data.push(0x80);
        }
        data.extend_from_slice(extra);
        FifoPacket {
            monitor_id: 1,
            timestamp_us: 0,
            data,
            is_keyframe,
            codec: VideoCodec::H264,
        }
    }

    /// Build a continuation slice of a multi-slice picture: a VCL NAL whose
    /// `first_mb_in_slice` is greater than 0 (MSB of the slice header clear).
    fn make_continuation_slice(nal_type: u8) -> FifoPacket {
        let raw = nal_type & 0x1F;
        let is_keyframe = raw == 5;
        // 0x00 slice-header byte → leading Exp-Golomb bit clear → MB index > 0.
        let data = vec![0x00, 0x00, 0x00, 0x01, nal_type, 0x00, 0xAA];
        FifoPacket {
            monitor_id: 1,
            timestamp_us: 0,
            data,
            is_keyframe,
            codec: VideoCodec::H264,
        }
    }

    /// Build a slice NAL whose slice header opens with `first_mb_in_slice`,
    /// Exp-Golomb (`ue(v)`) encoded as a real encoder would — so continuation
    /// slices carry the large macroblock indices a 4K picture produces.
    fn make_slice(nal_type: u8, first_mb_in_slice: u32) -> FifoPacket {
        let code = first_mb_in_slice as u64 + 1;
        let significant = 64 - code.leading_zeros();
        let leading_zeros = significant - 1;
        let mut bits: Vec<bool> = Vec::new();
        bits.extend(std::iter::repeat_n(false, leading_zeros as usize));
        for i in (0..significant).rev() {
            bits.push((code >> i) & 1 == 1);
        }
        while !bits.len().is_multiple_of(8) {
            bits.push(false);
        }
        let mut data = vec![0x00, 0x00, 0x00, 0x01, nal_type];
        for chunk in bits.chunks(8) {
            let mut byte = 0u8;
            for (i, &bit) in chunk.iter().enumerate() {
                if bit {
                    byte |= 1 << (7 - i);
                }
            }
            data.push(byte);
        }
        FifoPacket {
            monitor_id: 1,
            timestamp_us: 0,
            data,
            is_keyframe: (nal_type & 0x1F) == 5,
            codec: VideoCodec::H264,
        }
    }

    #[test]
    fn test_au_assembler_4k_many_slices_single_au() {
        // A 3840×2160 picture is 240×135 = 32400 macroblocks. Real 4K cameras
        // split it into many slices; only the first has first_mb_in_slice == 0.
        // Every continuation slice must stay in the SAME access unit so the
        // RTP packetizer gives the whole picture one timestamp + marker bit.
        const SLICES: u32 = 24;
        let mut asm = AccessUnitAssembler::new();

        asm.push(&make_nal(0x67, &[0x64, 0x00, 0x33])); // SPS
        asm.push(&make_nal(0x68, &[0xCE, 0x3C, 0x80])); // PPS

        // First IDR slice starts the picture; the rest continue it.
        assert!(asm.push(&make_slice(0x65, 0)).is_none());
        for s in 1..SLICES {
            assert!(
                asm.push(&make_slice(0x65, s * 1350)).is_none(),
                "continuation slice {s} must not flush the access unit"
            );
        }

        // The next picture's first slice flushes the 24-slice keyframe AU.
        let au = asm
            .push(&make_slice(0x41, 0))
            .expect("multi-slice keyframe AU expected");
        assert!(au.is_keyframe);
        // SPS + PPS + 24 IDR slices = 26 NAL start codes, all in one AU.
        let starts = au.data.windows(4).filter(|w| w == &[0, 0, 0, 1]).count();
        assert_eq!(
            starts, 26,
            "the whole 4K picture must emit as a single access unit"
        );
    }

    #[test]
    fn test_au_assembler_sps_pps_idr_grouped() {
        let mut asm = AccessUnitAssembler::new();

        // SPS (type 7) — non-VCL, buffered
        assert!(asm.push(&make_nal(0x67, &[0x4d, 0x00, 0x33])).is_none());
        // PPS (type 8) — non-VCL, buffered
        assert!(asm.push(&make_nal(0x68, &[0xCE, 0x3C, 0x80])).is_none());
        // IDR (type 5) — first VCL, buffered (no previous AU to flush)
        assert!(asm.push(&make_nal(0x65, &[0x88, 0x84])).is_none());

        // Next P-frame (type 1) triggers flush of the IDR AU
        let au = asm.push(&make_nal(0x41, &[0x9A, 0x21]));
        assert!(au.is_some());
        let au = au.unwrap();
        assert!(au.is_keyframe);
        // AU should contain SPS + PPS + IDR (3 start codes)
        let starts: Vec<_> = au
            .data
            .windows(4)
            .enumerate()
            .filter(|(_, w)| w == &[0, 0, 0, 1])
            .map(|(i, _)| i)
            .collect();
        assert_eq!(starts.len(), 3, "Expected 3 NAL units in keyframe AU");
    }

    #[test]
    fn test_au_assembler_p_frames_after_keyframe() {
        let mut asm = AccessUnitAssembler::new();

        // Prime with keyframe to clear needs_keyframe
        assert!(asm.push(&make_nal(0x65, &[0x88])).is_none()); // IDR buffered
        let au = asm.push(&make_nal(0x41, &[0x01])); // P flushes IDR
        assert!(au.is_some());
        assert!(au.unwrap().is_keyframe);

        // Now P-frames should emit normally
        let au = asm.push(&make_nal(0x41, &[0x02])); // flushes P1
        assert!(au.is_some());
        assert!(!au.unwrap().is_keyframe);

        let au = asm.push(&make_nal(0x41, &[0x03])); // flushes P2
        assert!(au.is_some());
        assert!(!au.unwrap().is_keyframe);
    }

    #[test]
    fn test_au_assembler_drops_until_keyframe() {
        let mut asm = AccessUnitAssembler::new();

        // P-frames before any keyframe should be dropped
        assert!(asm.push(&make_nal(0x41, &[0x01])).is_none()); // buffered
        assert!(asm.push(&make_nal(0x41, &[0x02])).is_none()); // flush P1 → dropped
        assert!(asm.push(&make_nal(0x41, &[0x03])).is_none()); // flush P2 → dropped

        // SPS arrives → flushes P3 (dropped), starts fresh AU
        assert!(asm.push(&make_nal(0x67, &[0x4d, 0x00, 0x33])).is_none());
        // PPS appended
        assert!(asm.push(&make_nal(0x68, &[0xCE, 0x3C, 0x80])).is_none());
        // IDR appended (first VCL in this AU)
        assert!(asm.push(&make_nal(0x65, &[0x88, 0x84])).is_none());

        // Next P triggers flush of keyframe AU [SPS+PPS+IDR]
        let au = asm.push(&make_nal(0x41, &[0x04]));
        assert!(au.is_some());
        let au = au.unwrap();
        assert!(au.is_keyframe);

        // Now P-frames flow normally
        let au = asm.push(&make_nal(0x41, &[0x05]));
        assert!(au.is_some());
        assert!(!au.unwrap().is_keyframe);
    }

    #[test]
    fn test_au_assembler_sps_boundary_separates_from_p_frame() {
        let mut asm = AccessUnitAssembler::new();

        // Prime with keyframe to clear needs_keyframe
        assert!(asm.push(&make_nal(0x65, &[0x88])).is_none());
        let au = asm.push(&make_nal(0x41, &[0x01]));
        assert!(au.unwrap().is_keyframe);

        // P2 flushes P1 (needs_keyframe is now cleared, so P-frames emit)
        let au = asm.push(&make_nal(0x41, &[0x02]));
        assert!(au.is_some());
        assert!(!au.unwrap().is_keyframe);

        // P3 buffered (P2 was just started in the buffer)
        // SPS arrives while P3 is buffered → flushes P3, starts fresh AU
        let au = asm.push(&make_nal(0x67, &[0x4d, 0x00, 0x33]));
        assert!(au.is_some());
        assert!(!au.unwrap().is_keyframe); // flushed AU was P2

        // PPS appended to SPS
        assert!(asm.push(&make_nal(0x68, &[0xCE, 0x3C, 0x80])).is_none());
        // IDR appended
        assert!(asm.push(&make_nal(0x65, &[0x88, 0x84])).is_none());

        // Next P flushes [SPS+PPS+IDR]
        let au = asm.push(&make_nal(0x41, &[0x03]));
        assert!(au.is_some());
        let au = au.unwrap();
        assert!(au.is_keyframe);
        // Verify 3 NALs in the keyframe AU
        let starts: Vec<_> = au
            .data
            .windows(4)
            .enumerate()
            .filter(|(_, w)| w == &[0, 0, 0, 1])
            .map(|(i, _)| i)
            .collect();
        assert_eq!(starts.len(), 3, "Expected SPS+PPS+IDR in keyframe AU");
    }

    #[test]
    fn test_au_assembler_multi_slice_keyframe_single_au() {
        // A 4K keyframe split across three slice NALs must emit as ONE AU so
        // the RTP packetizer gives the whole picture one timestamp + marker.
        let mut asm = AccessUnitAssembler::new();

        asm.push(&make_nal(0x67, &[0x4d, 0x00, 0x33])); // SPS
        asm.push(&make_nal(0x68, &[0xCE, 0x3C, 0x80])); // PPS
                                                        // First IDR slice (first_mb_in_slice == 0) starts the picture.
        assert!(asm.push(&make_nal(0x65, &[0x88])).is_none());
        // Two continuation IDR slices of the SAME picture — must NOT flush.
        assert!(asm.push(&make_continuation_slice(0x65)).is_none());
        assert!(asm.push(&make_continuation_slice(0x65)).is_none());

        // The next picture's first slice flushes the multi-slice keyframe AU.
        let au = asm.push(&make_nal(0x41, &[0x9A])).expect("AU expected");
        assert!(au.is_keyframe);
        // AU must contain SPS + PPS + 3 IDR slices = 5 NAL start codes.
        let starts = au.data.windows(4).filter(|w| w == &[0, 0, 0, 1]).count();
        assert_eq!(starts, 5, "multi-slice keyframe AU must hold all 5 NALs");
    }

    #[test]
    fn test_au_assembler_multi_slice_p_frame_single_au() {
        let mut asm = AccessUnitAssembler::new();

        // Prime with a keyframe to clear needs_keyframe.
        asm.push(&make_nal(0x65, &[0x88]));
        let au = asm.push(&make_nal(0x41, &[0x01])).expect("keyframe AU");
        assert!(au.is_keyframe);

        // The P slice just pushed is the current AU; add two continuation
        // slices of the same multi-slice P-frame.
        assert!(asm.push(&make_continuation_slice(0x41)).is_none());
        assert!(asm.push(&make_continuation_slice(0x41)).is_none());

        // The next picture's first slice flushes the 3-slice P-frame as one AU.
        let au = asm.push(&make_nal(0x41, &[0x02])).expect("P-frame AU");
        assert!(!au.is_keyframe);
        let starts = au.data.windows(4).filter(|w| w == &[0, 0, 0, 1]).count();
        assert_eq!(starts, 3, "multi-slice P-frame AU must hold all 3 slices");
    }

    #[test]
    fn test_au_assembler_multi_slice_keyframe_flag_from_first_slice() {
        // An AU is a keyframe when its first slice is an IDR slice, even when
        // later continuation slices are present.
        let mut asm = AccessUnitAssembler::new();
        asm.push(&make_nal(0x67, &[0x4d, 0x00, 0x33]));
        asm.push(&make_nal(0x68, &[0xCE]));
        asm.push(&make_nal(0x65, &[0x88])); // first IDR slice
        asm.push(&make_continuation_slice(0x65)); // continuation IDR slice
        let au = asm.push(&make_nal(0x41, &[0x9A])).expect("AU");
        assert!(au.is_keyframe);
    }

    #[test]
    fn test_au_assembler_aud_delimits_access_unit() {
        // An Access Unit Delimiter (NAL type 9) flushes the buffered picture.
        let mut asm = AccessUnitAssembler::new();
        asm.push(&make_nal(0x65, &[0x88])); // IDR slice buffered
        let au = asm
            .push(&make_nal(0x09, &[0xF0]))
            .expect("AUD flushes keyframe AU");
        assert!(au.is_keyframe);
    }

    #[test]
    fn test_au_assembler_empty_packet_ignored() {
        let mut asm = AccessUnitAssembler::new();
        let empty = FifoPacket {
            monitor_id: 1,
            timestamp_us: 0,
            data: vec![],
            is_keyframe: false,
            codec: VideoCodec::H264,
        };
        assert!(asm.push(&empty).is_none());
    }

    #[test]
    fn test_h264_nal_type_parsing() {
        // SPS
        assert_eq!(h264_nal_type(&[0, 0, 0, 1, 0x67]), Some(7));
        // PPS
        assert_eq!(h264_nal_type(&[0, 0, 0, 1, 0x68]), Some(8));
        // IDR
        assert_eq!(h264_nal_type(&[0, 0, 0, 1, 0x65]), Some(5));
        // Non-IDR slice
        assert_eq!(h264_nal_type(&[0, 0, 0, 1, 0x41]), Some(1));
        // 3-byte start code
        assert_eq!(h264_nal_type(&[0, 0, 1, 0x67]), Some(7));
        // No start code
        assert_eq!(h264_nal_type(&[0xFF, 0x67]), None);
    }

    #[test]
    fn test_extract_profile_level_id() {
        // Main Profile, Level 5.1: profile_idc=0x4D, constraints=0x00, level_idc=0x33
        let sps = vec![0x00, 0x00, 0x00, 0x01, 0x67, 0x4D, 0x00, 0x33, 0xFF];
        assert_eq!(extract_profile_level_id(&sps), Some("4d0033".to_string()));

        // Constrained Baseline, Level 3.1
        let sps = vec![0x00, 0x00, 0x00, 0x01, 0x67, 0x42, 0xE0, 0x1F];
        assert_eq!(extract_profile_level_id(&sps), Some("42e01f".to_string()));

        // High Profile, Level 4.0
        let sps = vec![0x00, 0x00, 0x01, 0x67, 0x64, 0x00, 0x28, 0xAA];
        assert_eq!(extract_profile_level_id(&sps), Some("640028".to_string()));

        // Not an SPS (PPS type 8)
        let pps = vec![0x00, 0x00, 0x00, 0x01, 0x68, 0xCE, 0x3C, 0x80];
        assert_eq!(extract_profile_level_id(&pps), None);

        // Too short
        let short = vec![0x00, 0x00, 0x00, 0x01, 0x67, 0x4D];
        assert_eq!(extract_profile_level_id(&short), None);

        // No start code
        let no_start = vec![0x67, 0x4D, 0x00, 0x33];
        assert_eq!(extract_profile_level_id(&no_start), None);
    }

    #[test]
    fn test_h264_offer_profile_level_ids() {
        // The offer advertises browser-universal pass-through profiles,
        // independent of the camera's native profile. Safari accepts only
        // Constrained Baseline (42e01f) and Constrained High (640c1f); both
        // are advertised so Safari can answer instead of rejecting the
        // m-line. Chrome accepts them too.
        assert_eq!(
            h264_offer_profile_level_ids(),
            vec!["42e01f".to_string(), "640c1f".to_string()]
        );

        // Constrained Baseline must always be offered — it is the one profile
        // every WebRTC implementation is guaranteed to support.
        assert!(h264_offer_profile_level_ids().contains(&"42e01f".to_string()));

        // The advertised levels are the conservative 3.1 (`…1f`); the real
        // 4K bitstream stays at its native level via level-asymmetry-allowed.
        for plid in h264_offer_profile_level_ids() {
            assert!(
                plid.ends_with("1f"),
                "offered profile must advertise level 3.1: {plid}"
            );
        }
    }

    #[test]
    fn test_clear_needs_keyframe() {
        let mut asm = AccessUnitAssembler::new();

        // P-frames should be dropped before any keyframe
        assert!(asm.push(&make_nal(0x41, &[0x01])).is_none()); // buffered
        assert!(asm.push(&make_nal(0x41, &[0x02])).is_none()); // flush P1 → dropped

        // Simulate external keyframe injection — clear needs_keyframe
        asm.clear_needs_keyframe();

        // Now P-frames should flow through (flushing the buffered P2)
        let au = asm.push(&make_nal(0x41, &[0x03])); // flush P2
        assert!(au.is_some());
        assert!(!au.unwrap().is_keyframe);
    }
}
