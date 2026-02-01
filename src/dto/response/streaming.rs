use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct StreamEndpoints {
    pub webrtc: String,     // HTML WebRTC player
    pub webrtc_api: String, // Direct WebRTC API endpoint
    pub hls: String,
    pub mjpeg: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct MonitorStreamingDetails {
    pub id: u32,
    pub name: String,
    /// The camera's RTSP URL (from monitor's Path field)
    /// This is the actual source URL to use for streaming
    pub rtsp_url: Option<String>,
    /// Secondary stream URL if available (from monitor's SecondPath field)
    pub rtsp_url_secondary: Option<String>,
    /// RTSP username (from monitor's User field)
    pub user: String,
    /// RTSP password (from monitor's Pass field)
    pub pass: String,
    /// Host extracted from Path URL (for reference)
    pub host: String,
    /// Port extracted from Path URL (for reference)
    pub port: u16,
}
