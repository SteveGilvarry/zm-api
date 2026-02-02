//! WebRTC live streaming from FIFO sources
//!
//! This module provides WebRTC streaming using the webrtc-rs crate.
//! It packetizes H.264/H.265 NAL units into RTP packets and sends them
//! through WebRTC peer connections.

use std::sync::atomic::{AtomicU16, AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;

use dashmap::DashMap;
use tokio::sync::RwLock;
use tracing::{debug, info};
use uuid::Uuid;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::APIBuilder;
use webrtc::ice_transport::ice_candidate::RTCIceCandidateInit;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::peer_connection::RTCPeerConnection;
use webrtc::rtp_transceiver::rtp_codec::RTCRtpCodecCapability;
use webrtc::track::track_local::track_local_static_rtp::TrackLocalStaticRTP;
use webrtc::track::track_local::{TrackLocal, TrackLocalWriter};

use crate::streaming::source::{FifoPacket, VideoCodec};

/// RTP payload type for H.264
const RTP_PAYLOAD_TYPE_H264: u8 = 96;
/// RTP clock rate (90kHz for video)
const RTP_CLOCK_RATE: u32 = 90000;
/// Maximum RTP packet size (MTU - IP/UDP headers)
const MAX_RTP_PACKET_SIZE: usize = 1400;

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
    pub video_track: Arc<TrackLocalStaticRTP>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    packets_sent: AtomicU64,
    bytes_sent: AtomicU64,
    /// RTP sequence number
    rtp_sequence: AtomicU16,
    /// RTP timestamp
    rtp_timestamp: AtomicU32,
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

    /// Write a video packet (NAL unit) to the WebRTC track
    ///
    /// This method handles RTP packetization of H.264 NAL units,
    /// including fragmentation for NAL units larger than MTU.
    pub async fn write_packet(&self, packet: &FifoPacket) -> Result<(), WebRtcLiveError> {
        if packet.data.is_empty() {
            return Ok(());
        }

        // Convert timestamp from microseconds to RTP timestamp (90kHz clock)
        let rtp_ts = ((packet.timestamp_us as u64) * RTP_CLOCK_RATE as u64 / 1_000_000) as u32;
        self.rtp_timestamp.store(rtp_ts, Ordering::Relaxed);

        // Process NAL units from the packet
        // The data might contain multiple NAL units with start codes
        let nals = parse_nal_units(&packet.data);

        for nal in nals {
            if nal.is_empty() {
                continue;
            }

            // Packetize and send the NAL unit
            self.send_nal_unit(nal, rtp_ts).await?;
        }

        Ok(())
    }

    /// Send a single NAL unit, fragmenting if necessary
    async fn send_nal_unit(&self, nal: &[u8], rtp_timestamp: u32) -> Result<(), WebRtcLiveError> {
        if nal.is_empty() {
            return Ok(());
        }

        // Check if NAL fits in single packet
        if nal.len() <= MAX_RTP_PACKET_SIZE {
            // Single NAL unit packet
            self.send_rtp_packet(nal, rtp_timestamp, true).await?;
        } else {
            // Need to fragment using FU-A (Fragmentation Unit type A)
            self.send_fragmented_nal(nal, rtp_timestamp).await?;
        }

        Ok(())
    }

    /// Send a NAL unit as FU-A fragments
    async fn send_fragmented_nal(
        &self,
        nal: &[u8],
        rtp_timestamp: u32,
    ) -> Result<(), WebRtcLiveError> {
        if nal.is_empty() {
            return Ok(());
        }

        let nal_header = nal[0];
        let nal_type = nal_header & 0x1F;
        let nri = nal_header & 0x60;

        // FU indicator: same NRI as original, type = 28 (FU-A)
        let fu_indicator = nri | 28;

        // Fragment the NAL unit (skip the NAL header byte)
        let payload = &nal[1..];
        let max_payload = MAX_RTP_PACKET_SIZE - 2; // Account for FU indicator and header

        let mut offset = 0;
        let mut is_first = true;

        while offset < payload.len() {
            let remaining = payload.len() - offset;
            let fragment_size = remaining.min(max_payload);
            let is_last = offset + fragment_size >= payload.len();

            // FU header
            let fu_header = if is_first {
                0x80 | nal_type // Start bit set
            } else if is_last {
                0x40 | nal_type // End bit set
            } else {
                nal_type // Neither start nor end
            };

            // Build FU-A packet
            let mut fu_packet = Vec::with_capacity(2 + fragment_size);
            fu_packet.push(fu_indicator);
            fu_packet.push(fu_header);
            fu_packet.extend_from_slice(&payload[offset..offset + fragment_size]);

            self.send_rtp_packet(&fu_packet, rtp_timestamp, is_last)
                .await?;

            offset += fragment_size;
            is_first = false;
        }

        Ok(())
    }

    /// Send an RTP packet to the video track
    async fn send_rtp_packet(
        &self,
        payload: &[u8],
        timestamp: u32,
        marker: bool,
    ) -> Result<(), WebRtcLiveError> {
        // Get next sequence number
        let sequence = self.rtp_sequence.fetch_add(1, Ordering::Relaxed);

        // Build RTP header
        let mut rtp_packet = Vec::with_capacity(12 + payload.len());

        // Version (2), Padding (0), Extension (0), CSRC count (0)
        rtp_packet.push(0x80);
        // Marker bit and payload type
        rtp_packet.push(if marker {
            0x80 | RTP_PAYLOAD_TYPE_H264
        } else {
            RTP_PAYLOAD_TYPE_H264
        });
        // Sequence number (big endian)
        rtp_packet.extend_from_slice(&sequence.to_be_bytes());
        // Timestamp (big endian)
        rtp_packet.extend_from_slice(&timestamp.to_be_bytes());
        // SSRC (use monitor_id as SSRC)
        rtp_packet.extend_from_slice(&self.monitor_id.to_be_bytes());
        // Payload
        rtp_packet.extend_from_slice(payload);

        // Write to track using the TrackLocalWriter trait
        TrackLocalWriter::write(&*self.video_track, &rtp_packet)
            .await
            .map_err(|e| {
                WebRtcLiveError::WebRtcError(format!("Failed to write RTP packet: {}", e))
            })?;

        // Update stats
        self.packets_sent.fetch_add(1, Ordering::Relaxed);
        self.bytes_sent
            .fetch_add(rtp_packet.len() as u64, Ordering::Relaxed);

        Ok(())
    }
}

/// Parse NAL units from a byte buffer
///
/// Handles both Annex B format (start codes) and raw NAL units
fn parse_nal_units(data: &[u8]) -> Vec<&[u8]> {
    let mut nals = Vec::new();
    let mut i = 0;

    while i < data.len() {
        // Look for start code (0x00 0x00 0x01 or 0x00 0x00 0x00 0x01)
        let start_code_len = if i + 4 <= data.len()
            && data[i] == 0x00
            && data[i + 1] == 0x00
            && data[i + 2] == 0x00
            && data[i + 3] == 0x01
        {
            4
        } else if i + 3 <= data.len()
            && data[i] == 0x00
            && data[i + 1] == 0x00
            && data[i + 2] == 0x01
        {
            3
        } else {
            // No start code found at current position
            // If we're at the start, assume raw NAL unit
            if i == 0 && nals.is_empty() {
                nals.push(data);
                return nals;
            }
            i += 1;
            continue;
        };

        let nal_start = i + start_code_len;

        // Find end of this NAL (next start code or end of data)
        let mut nal_end = data.len();
        let mut j = nal_start;

        while j < data.len() - 2 {
            if data[j] == 0x00
                && data[j + 1] == 0x00
                && ((j + 2 < data.len() && data[j + 2] == 0x01)
                    || (j + 3 < data.len() && data[j + 2] == 0x00 && data[j + 3] == 0x01))
            {
                nal_end = j;
                break;
            }
            j += 1;
        }

        if nal_start < nal_end {
            nals.push(&data[nal_start..nal_end]);
        }

        i = nal_end;
    }

    nals
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

/// Manager for WebRTC live streaming sessions
pub struct WebRtcLiveManager {
    config: WebRtcLiveConfig,
    sessions: DashMap<String, Arc<RwLock<WebRtcLiveSession>>>,
    /// Map from monitor_id to list of session IDs
    monitor_sessions: DashMap<u32, Vec<String>>,
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
    pub async fn create_session(
        &self,
        monitor_id: u32,
        codec: VideoCodec,
    ) -> Result<(String, RTCSessionDescription), WebRtcLiveError> {
        // Check session limit
        if self.sessions.len() >= self.config.max_sessions {
            return Err(WebRtcLiveError::WebRtcError(
                "Maximum sessions reached".to_string(),
            ));
        }

        // Create media engine with codec support
        let mut media_engine = MediaEngine::default();

        let codec_capability = match codec {
            VideoCodec::H264 => RTCRtpCodecCapability {
                mime_type: "video/H264".to_string(),
                clock_rate: 90000,
                channels: 0,
                sdp_fmtp_line:
                    "level-asymmetry-allowed=1;packetization-mode=1;profile-level-id=42001f"
                        .to_string(),
                rtcp_feedback: vec![],
            },
            VideoCodec::H265 => RTCRtpCodecCapability {
                mime_type: "video/H265".to_string(),
                clock_rate: 90000,
                channels: 0,
                sdp_fmtp_line: String::new(),
                rtcp_feedback: vec![],
            },
            VideoCodec::Unknown => {
                return Err(WebRtcLiveError::UnsupportedCodec("Unknown".to_string()));
            }
        };

        media_engine
            .register_codec(
                webrtc::rtp_transceiver::rtp_codec::RTCRtpCodecParameters {
                    capability: codec_capability.clone(),
                    payload_type: 96,
                    ..Default::default()
                },
                webrtc::rtp_transceiver::rtp_codec::RTPCodecType::Video,
            )
            .map_err(|e| WebRtcLiveError::WebRtcError(e.to_string()))?;

        // Build API
        let api = APIBuilder::new().with_media_engine(media_engine).build();

        // Create ICE servers configuration
        let mut ice_servers = vec![];

        for stun_url in &self.config.stun_servers {
            ice_servers.push(RTCIceServer {
                urls: vec![stun_url.clone()],
                ..Default::default()
            });
        }

        #[allow(clippy::needless_update)]
        if let Some(turn_url) = &self.config.turn_server {
            ice_servers.push(RTCIceServer {
                urls: vec![turn_url.clone()],
                username: self.config.turn_username.clone().unwrap_or_default(),
                credential: self.config.turn_password.clone().unwrap_or_default(),
                ..Default::default()
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

        // Create video track
        let video_track = Arc::new(TrackLocalStaticRTP::new(
            codec_capability,
            "video".to_string(),
            format!("zm-live-{}", monitor_id),
        ));

        // Add track to peer connection
        peer_connection
            .add_track(Arc::clone(&video_track) as Arc<dyn TrackLocal + Send + Sync>)
            .await
            .map_err(|e| WebRtcLiveError::WebRtcError(e.to_string()))?;

        // Create offer
        let offer = peer_connection
            .create_offer(None)
            .await
            .map_err(|e| WebRtcLiveError::WebRtcError(e.to_string()))?;

        // Set local description
        peer_connection
            .set_local_description(offer.clone())
            .await
            .map_err(|e| WebRtcLiveError::WebRtcError(e.to_string()))?;

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
            rtp_sequence: AtomicU16::new(0),
            rtp_timestamp: AtomicU32::new(0),
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

        Ok((session_id_str, offer))
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

    /// Write a packet to a session's video track
    pub async fn write_packet(
        &self,
        session_id: &str,
        packet: &FifoPacket,
    ) -> Result<(), WebRtcLiveError> {
        let session_lock = self
            .sessions
            .get(session_id)
            .ok_or_else(|| WebRtcLiveError::SessionNotFound(session_id.to_string()))?;

        let session = session_lock.read().await;
        session.write_packet(packet).await
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
}
