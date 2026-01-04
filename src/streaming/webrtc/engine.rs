use interceptor::registry::Registry;
use std::sync::Arc;
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::setting_engine::SettingEngine;
use webrtc::api::APIBuilder;
use webrtc::api::API;
use webrtc::ice_transport::ice_candidate::RTCIceCandidateInit;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::peer_connection::RTCPeerConnection;
use webrtc::rtp_transceiver::rtp_codec::{
    RTCRtpCodecCapability, RTCRtpCodecParameters, RTPCodecType,
};
use webrtc::track::track_local::track_local_static_rtp::TrackLocalStaticRTP;
use webrtc::track::track_local::TrackLocal;

use crate::configure::streaming::WebRtcConfig;

/// WebRTC engine for creating and managing peer connections
pub struct WebRtcEngine {
    api: API,
    config: WebRtcConfig,
    ice_servers: Vec<RTCIceServer>,
}

/// Parameters for creating a new peer connection
pub struct PeerConnectionParams {
    pub monitor_id: u32,
    pub enable_audio: bool,
}

/// Result of creating a peer connection
pub struct PeerConnectionResult {
    pub peer_connection: Arc<RTCPeerConnection>,
    pub video_track: Arc<TrackLocalStaticRTP>,
    pub audio_track: Option<Arc<TrackLocalStaticRTP>>,
}

#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("Failed to create media engine: {0}")]
    MediaEngine(String),
    #[error("Failed to create peer connection: {0}")]
    PeerConnection(String),
    #[error("Failed to create offer: {0}")]
    CreateOffer(String),
    #[error("Failed to set local description: {0}")]
    SetLocalDescription(String),
    #[error("Failed to set remote description: {0}")]
    SetRemoteDescription(String),
    #[error("Failed to add ICE candidate: {0}")]
    AddIceCandidate(String),
    #[error("Invalid SDP: {0}")]
    InvalidSdp(String),
}

impl WebRtcEngine {
    /// Create a new WebRTC engine with the given configuration
    pub fn new(config: WebRtcConfig) -> Result<Self, EngineError> {
        tracing::info!("Initializing WebRTC engine");

        // Configure media engine with H.264 support
        let media_engine = configure_media_engine()?;

        // Create setting engine
        let mut setting_engine = SettingEngine::default();

        // Configure ICE settings for optimal performance
        setting_engine.set_lite(false);
        setting_engine.set_ice_timeouts(
            Some(std::time::Duration::from_secs(5)),
            Some(std::time::Duration::from_secs(10)),
            Some(std::time::Duration::from_millis(200)),
        );

        // Build API with interceptors
        let mut media_engine_mut = media_engine;
        let mut registry = Registry::new();
        registry = register_default_interceptors(registry, &mut media_engine_mut)
            .map_err(|e| EngineError::MediaEngine(e.to_string()))?;

        let api = APIBuilder::new()
            .with_media_engine(media_engine_mut)
            .with_setting_engine(setting_engine)
            .with_interceptor_registry(registry)
            .build();

        // Parse ICE servers from config
        let ice_servers = Self::parse_ice_servers(&config)?;

        tracing::info!(
            stun_servers = ?config.stun_servers,
            turn_enabled = config.turn.as_ref().map(|t| t.enabled).unwrap_or(false),
            "WebRTC engine initialized with {} ICE servers",
            ice_servers.len()
        );

        Ok(Self {
            api,
            config,
            ice_servers,
        })
    }

    /// Parse ICE servers from configuration
    fn parse_ice_servers(config: &WebRtcConfig) -> Result<Vec<RTCIceServer>, EngineError> {
        let mut ice_servers = Vec::new();

        // Add STUN servers
        if !config.stun_servers.is_empty() {
            ice_servers.push(RTCIceServer {
                urls: config.stun_servers.clone(),
                username: String::new(),
                credential: String::new(),
            });
        }

        // Add TURN server if configured
        if let Some(turn_config) = &config.turn {
            if turn_config.enabled {
                ice_servers.push(RTCIceServer {
                    urls: vec![turn_config.server.clone()],
                    username: turn_config.username.clone(),
                    credential: turn_config.password.clone(),
                });
            }
        }

        if ice_servers.is_empty() {
            tracing::warn!(
                "No ICE servers configured, peer connections may fail for non-local networks"
            );
        }

        Ok(ice_servers)
    }

    /// Create a new peer connection for streaming
    pub async fn create_peer_connection(
        &self,
        params: PeerConnectionParams,
    ) -> Result<PeerConnectionResult, EngineError> {
        tracing::info!(
            monitor_id = params.monitor_id,
            enable_audio = params.enable_audio,
            "Creating peer connection"
        );

        // Create RTCConfiguration
        let rtc_config = RTCConfiguration {
            ice_servers: self.ice_servers.clone(),
            ..Default::default()
        };

        // Create peer connection
        let peer_connection = Arc::new(
            self.api
                .new_peer_connection(rtc_config)
                .await
                .map_err(|e| EngineError::PeerConnection(e.to_string()))?,
        );

        // Create video track
        let video_track = Arc::new(TrackLocalStaticRTP::new(
            RTCRtpCodecCapability {
                mime_type: "video/H264".to_owned(),
                clock_rate: 90000,
                channels: 0,
                sdp_fmtp_line:
                    "level-asymmetry-allowed=1;packetization-mode=1;profile-level-id=42e01f"
                        .to_owned(),
                rtcp_feedback: vec![],
            },
            format!("video-{}", params.monitor_id),
            format!("zm-monitor-{}", params.monitor_id),
        ));

        // Add video track to peer connection
        let _rtp_sender = peer_connection
            .add_track(Arc::clone(&video_track) as Arc<dyn TrackLocal + Send + Sync>)
            .await
            .map_err(|e| {
                EngineError::PeerConnection(format!("Failed to add video track: {}", e))
            })?;

        tracing::debug!("Video track added to peer connection");

        // Create audio track if requested
        let audio_track = if params.enable_audio {
            let track = Arc::new(TrackLocalStaticRTP::new(
                RTCRtpCodecCapability {
                    mime_type: "audio/opus".to_owned(),
                    clock_rate: 48000,
                    channels: 2,
                    sdp_fmtp_line: "".to_owned(),
                    rtcp_feedback: vec![],
                },
                format!("audio-{}", params.monitor_id),
                format!("zm-monitor-{}-audio", params.monitor_id),
            ));

            // Add audio track to peer connection
            let _rtp_sender = peer_connection
                .add_track(Arc::clone(&track) as Arc<dyn TrackLocal + Send + Sync>)
                .await
                .map_err(|e| {
                    EngineError::PeerConnection(format!("Failed to add audio track: {}", e))
                })?;

            tracing::debug!("Audio track added to peer connection");
            Some(track)
        } else {
            None
        };

        // Set up connection state change handler
        let monitor_id_1 = params.monitor_id;
        peer_connection.on_peer_connection_state_change(Box::new(move |state| {
            tracing::info!(
                monitor_id = monitor_id_1,
                state = ?state,
                "Peer connection state changed"
            );
            Box::pin(async move {})
        }));

        // Set up ICE connection state change handler
        let monitor_id_2 = params.monitor_id;
        peer_connection.on_ice_connection_state_change(Box::new(move |state| {
            tracing::debug!(
                monitor_id = monitor_id_2,
                state = ?state,
                "ICE connection state changed"
            );
            Box::pin(async move {})
        }));

        // Set up ICE gathering state change handler
        let monitor_id_3 = params.monitor_id;
        peer_connection.on_ice_gathering_state_change(Box::new(move |state| {
            tracing::debug!(
                monitor_id = monitor_id_3,
                state = ?state,
                "ICE gathering state changed"
            );
            Box::pin(async move {})
        }));

        tracing::info!(
            monitor_id = params.monitor_id,
            has_audio = audio_track.is_some(),
            "Peer connection created successfully"
        );

        Ok(PeerConnectionResult {
            peer_connection,
            video_track,
            audio_track,
        })
    }

    /// Process an SDP offer and return an SDP answer
    pub async fn process_offer(
        &self,
        pc: &RTCPeerConnection,
        offer_sdp: &str,
    ) -> Result<String, EngineError> {
        tracing::debug!("Processing SDP offer");

        // Parse the offer SDP
        let offer = RTCSessionDescription::offer(offer_sdp.to_owned())
            .map_err(|e| EngineError::InvalidSdp(e.to_string()))?;

        // Set remote description
        pc.set_remote_description(offer)
            .await
            .map_err(|e| EngineError::SetRemoteDescription(e.to_string()))?;

        tracing::debug!("Remote description set");

        // Create answer
        let answer = pc
            .create_answer(None)
            .await
            .map_err(|e| EngineError::CreateOffer(e.to_string()))?;

        // Set local description
        pc.set_local_description(answer.clone())
            .await
            .map_err(|e| EngineError::SetLocalDescription(e.to_string()))?;

        tracing::debug!("Answer created and local description set");

        Ok(answer.sdp)
    }

    /// Create an SDP offer (for server-initiated connections)
    pub async fn create_offer(&self, pc: &RTCPeerConnection) -> Result<String, EngineError> {
        tracing::debug!("Creating SDP offer");

        // Create offer
        let offer = pc
            .create_offer(None)
            .await
            .map_err(|e| EngineError::CreateOffer(e.to_string()))?;

        // Set local description
        pc.set_local_description(offer.clone())
            .await
            .map_err(|e| EngineError::SetLocalDescription(e.to_string()))?;

        tracing::debug!("Offer created and local description set");

        Ok(offer.sdp)
    }

    /// Process an SDP answer
    pub async fn process_answer(
        &self,
        pc: &RTCPeerConnection,
        answer_sdp: &str,
    ) -> Result<(), EngineError> {
        tracing::debug!("Processing SDP answer");

        // Parse the answer SDP
        let answer = RTCSessionDescription::answer(answer_sdp.to_owned())
            .map_err(|e| EngineError::InvalidSdp(e.to_string()))?;

        // Set remote description
        pc.set_remote_description(answer)
            .await
            .map_err(|e| EngineError::SetRemoteDescription(e.to_string()))?;

        tracing::debug!("Remote description set from answer");

        Ok(())
    }

    /// Add an ICE candidate to the peer connection
    pub async fn add_ice_candidate(
        &self,
        pc: &RTCPeerConnection,
        candidate: &str,
        sdp_mid: Option<&str>,
        sdp_mline_index: Option<u16>,
    ) -> Result<(), EngineError> {
        tracing::debug!(
            candidate = candidate,
            sdp_mid = ?sdp_mid,
            sdp_mline_index = ?sdp_mline_index,
            "Adding ICE candidate"
        );

        let ice_candidate = RTCIceCandidateInit {
            candidate: candidate.to_owned(),
            sdp_mid: sdp_mid.map(|s| s.to_owned()),
            sdp_mline_index,
            username_fragment: None,
        };

        pc.add_ice_candidate(ice_candidate)
            .await
            .map_err(|e| EngineError::AddIceCandidate(e.to_string()))?;

        tracing::debug!("ICE candidate added successfully");

        Ok(())
    }
}

/// Configure the media engine with H.264 support
fn configure_media_engine() -> Result<MediaEngine, EngineError> {
    let mut media_engine = MediaEngine::default();

    // Register H.264 video codec
    // Using baseline profile (42e01f) for better compatibility with surveillance systems
    media_engine
        .register_codec(
            RTCRtpCodecParameters {
                capability: RTCRtpCodecCapability {
                    mime_type: "video/H264".to_owned(),
                    clock_rate: 90000,
                    channels: 0,
                    sdp_fmtp_line:
                        "level-asymmetry-allowed=1;packetization-mode=1;profile-level-id=42e01f"
                            .to_owned(),
                    rtcp_feedback: vec![],
                },
                payload_type: 96,
                ..Default::default()
            },
            RTPCodecType::Video,
        )
        .map_err(|e| EngineError::MediaEngine(format!("Failed to register H.264 codec: {}", e)))?;

    // Register Opus audio codec (for future audio support)
    media_engine
        .register_codec(
            RTCRtpCodecParameters {
                capability: RTCRtpCodecCapability {
                    mime_type: "audio/opus".to_owned(),
                    clock_rate: 48000,
                    channels: 2,
                    sdp_fmtp_line: "".to_owned(),
                    rtcp_feedback: vec![],
                },
                payload_type: 111,
                ..Default::default()
            },
            RTPCodecType::Audio,
        )
        .map_err(|e| EngineError::MediaEngine(format!("Failed to register Opus codec: {}", e)))?;

    // Register PCMU (G.711 μ-law) for compatibility
    media_engine
        .register_codec(
            RTCRtpCodecParameters {
                capability: RTCRtpCodecCapability {
                    mime_type: "audio/PCMU".to_owned(),
                    clock_rate: 8000,
                    channels: 1,
                    sdp_fmtp_line: "".to_owned(),
                    rtcp_feedback: vec![],
                },
                payload_type: 0,
                ..Default::default()
            },
            RTPCodecType::Audio,
        )
        .map_err(|e| EngineError::MediaEngine(format!("Failed to register PCMU codec: {}", e)))?;

    tracing::debug!("Media engine configured with H.264, Opus, and PCMU codecs");

    Ok(media_engine)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ice_servers_with_stun() {
        let config = WebRtcConfig {
            enabled: true,
            mode: "native".to_string(),
            max_connections: 100,
            connection_timeout_seconds: 30,
            stun_servers: vec![
                "stun:stun.l.google.com:19302".to_string(),
                "stun:stun1.l.google.com:19302".to_string(),
            ],
            turn: None,
        };

        let ice_servers = WebRtcEngine::parse_ice_servers(&config).unwrap();
        assert_eq!(ice_servers.len(), 1);
        assert_eq!(ice_servers[0].urls.len(), 2);
        assert_eq!(ice_servers[0].urls[0], "stun:stun.l.google.com:19302");
    }

    #[test]
    fn test_parse_ice_servers_with_turn() {
        use crate::configure::streaming::TurnConfig;

        let config = WebRtcConfig {
            enabled: true,
            mode: "native".to_string(),
            max_connections: 100,
            connection_timeout_seconds: 30,
            stun_servers: vec!["stun:stun.l.google.com:19302".to_string()],
            turn: Some(TurnConfig {
                enabled: true,
                server: "turn:turn.example.com:3478".to_string(),
                username: "user".to_string(),
                password: "pass".to_string(),
            }),
        };

        let ice_servers = WebRtcEngine::parse_ice_servers(&config).unwrap();
        assert_eq!(ice_servers.len(), 2);
        assert_eq!(ice_servers[1].urls[0], "turn:turn.example.com:3478");
        assert_eq!(ice_servers[1].username, "user");
        assert_eq!(ice_servers[1].credential, "pass");
    }

    #[test]
    fn test_parse_ice_servers_empty() {
        let config = WebRtcConfig {
            enabled: true,
            mode: "native".to_string(),
            max_connections: 100,
            connection_timeout_seconds: 30,
            stun_servers: vec![],
            turn: None,
        };

        let ice_servers = WebRtcEngine::parse_ice_servers(&config).unwrap();
        assert_eq!(ice_servers.len(), 0);
    }

    #[test]
    fn test_configure_media_engine() {
        let result = configure_media_engine();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_engine() {
        let config = WebRtcConfig::default();
        let result = WebRtcEngine::new(config);
        assert!(result.is_ok());
    }
}
