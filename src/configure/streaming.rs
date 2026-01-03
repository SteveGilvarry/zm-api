use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct StreamingConfig {
    pub enabled: bool,
    pub default_protocol: StreamingProtocol,
    pub source: SourceConfig,
    pub zoneminder: ZoneMinderConfig,
    pub go2rtc: Go2RtcConfig,
    pub webrtc: WebRtcConfig,
    pub hls: HlsConfig,
    pub mse: MseConfig,
    pub rtsp_proxy: RtspProxyConfig,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_protocol: StreamingProtocol::default(),
            source: SourceConfig::default(),
            zoneminder: ZoneMinderConfig::default(),
            go2rtc: Go2RtcConfig::default(),
            webrtc: WebRtcConfig::default(),
            hls: HlsConfig::default(),
            mse: MseConfig::default(),
            rtsp_proxy: RtspProxyConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum StreamingProtocol {
    #[default]
    Auto,
    Webrtc,
    Hls,
    Mse,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct SourceConfig {
    pub priority: Vec<String>, // ["fifo", "rtsp", "go2rtc"]
    pub prefer_direct_rtsp: bool,
    pub fallback_to_go2rtc: bool,
    pub cache_sdp_seconds: u32,
}

impl Default for SourceConfig {
    fn default() -> Self {
        Self {
            priority: vec!["fifo".to_string(), "rtsp".to_string(), "go2rtc".to_string()],
            prefer_direct_rtsp: false,
            fallback_to_go2rtc: true,
            cache_sdp_seconds: 300,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ZoneMinderConfig {
    pub enabled: bool,
    pub fifo_base_path: String,
    pub video_fifo_suffix: String,
    pub audio_fifo_suffix: String,
    pub fifo_read_timeout_ms: u64,
    pub reconnect_delay_ms: u64,
}

impl Default for ZoneMinderConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            fifo_base_path: "/dev/shm".to_string(),
            video_fifo_suffix: "-v.fifo".to_string(),
            audio_fifo_suffix: "-a.fifo".to_string(),
            fifo_read_timeout_ms: 5000,
            reconnect_delay_ms: 1000,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Go2RtcConfig {
    pub enabled: bool,
    pub base_url: String,
    pub timeout_seconds: u64,
    pub auto_register: bool,
    pub health_check_interval_seconds: u64,
    pub retry_attempts: u32,
}

impl Default for Go2RtcConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            base_url: "http://localhost:1984".to_string(),
            timeout_seconds: 10,
            auto_register: true,
            health_check_interval_seconds: 30,
            retry_attempts: 3,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct WebRtcConfig {
    pub enabled: bool,
    pub mode: String, // "native" | "plugin" | "go2rtc"
    pub max_connections: u32,
    pub connection_timeout_seconds: u64,
    pub stun_servers: Vec<String>,
    pub turn: Option<TurnConfig>,
}

impl Default for WebRtcConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            mode: "go2rtc".to_string(),
            max_connections: 100,
            connection_timeout_seconds: 30,
            stun_servers: vec![
                "stun:stun.l.google.com:19302".to_string(),
                "stun:stun1.l.google.com:19302".to_string(),
            ],
            turn: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct TurnConfig {
    pub enabled: bool,
    pub server: String,
    pub username: String,
    pub password: String,
}

impl Default for TurnConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            server: "turn:turn.example.com:3478".to_string(),
            username: String::new(),
            password: String::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct HlsConfig {
    pub enabled: bool,
    pub segment_duration_seconds: u32,
    pub playlist_size: u32,
    pub ll_hls_enabled: bool,
    pub partial_segment_ms: u32,
    pub storage: HlsStorageConfig,
}

impl Default for HlsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            segment_duration_seconds: 2,
            playlist_size: 10,
            ll_hls_enabled: false,
            partial_segment_ms: 200,
            storage: HlsStorageConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct HlsStorageConfig {
    pub path: String,
    pub retention_minutes: u32,
}

impl Default for HlsStorageConfig {
    fn default() -> Self {
        Self {
            path: "/tmp/hls".to_string(),
            retention_minutes: 60,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct MseConfig {
    pub enabled: bool,
    pub mode: String, // "native" | "plugin"
    pub max_buffer_segments: u32,
    pub segment_duration_ms: u32,
}

impl Default for MseConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            mode: "native".to_string(),
            max_buffer_segments: 20,
            segment_duration_ms: 1000,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct RtspProxyConfig {
    pub enabled: bool,
    pub port: u16,
    pub rtp_port_range_start: u16,
    pub rtp_port_range_end: u16,
    pub max_sessions: u32,
    pub transport: String, // "udp" | "tcp" | "auto"
}

impl Default for RtspProxyConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            port: 8554,
            rtp_port_range_start: 20000,
            rtp_port_range_end: 30000,
            max_sessions: 100,
            transport: "auto".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_streaming_config_default() {
        let config = StreamingConfig::default();
        assert!(config.enabled);
        assert_eq!(config.source.priority.len(), 3);
        assert_eq!(config.source.priority[0], "fifo");
    }

    #[test]
    fn test_streaming_protocol_default() {
        let protocol = StreamingProtocol::default();
        assert!(matches!(protocol, StreamingProtocol::Auto));
    }

    #[test]
    fn test_webrtc_config_default() {
        let config = WebRtcConfig::default();
        assert!(config.enabled);
        assert_eq!(config.mode, "go2rtc");
        assert_eq!(config.stun_servers.len(), 2);
        assert!(config.turn.is_none());
    }

    #[test]
    fn test_go2rtc_config_default() {
        let config = Go2RtcConfig::default();
        assert!(!config.enabled); // Disabled by default
        assert_eq!(config.base_url, "http://localhost:1984");
        assert!(config.auto_register);
    }

    #[test]
    fn test_hls_config_default() {
        let config = HlsConfig::default();
        assert!(config.enabled);
        assert_eq!(config.segment_duration_seconds, 2);
        assert!(!config.ll_hls_enabled);
    }

    #[test]
    fn test_rtsp_proxy_config_default() {
        let config = RtspProxyConfig::default();
        assert!(!config.enabled); // Disabled by default
        assert_eq!(config.port, 8554);
        assert_eq!(config.rtp_port_range_start, 20000);
        assert_eq!(config.rtp_port_range_end, 30000);
    }
}
