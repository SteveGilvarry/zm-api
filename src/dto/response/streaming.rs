use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct StreamEndpoints {
    pub webrtc: String,      // HTML WebRTC player 
    pub webrtc_api: String,  // Direct WebRTC API endpoint
    pub hls: String,
    pub mjpeg: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct MonitorStreamingDetails {
    pub id: u32,
    pub name: String,
    pub user: String,
    pub pass: String,
    pub host: String,
    pub port: u16,
}