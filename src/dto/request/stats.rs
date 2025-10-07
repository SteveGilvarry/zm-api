use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct CreateStatRequest {
    pub monitor_id: u32,
    pub zone_id: u32,
    pub event_id: u64,
    pub frame_id: u32,
    pub pixel_diff: u8,
    pub alarm_pixels: u32,
    pub filter_pixels: u32,
    pub blob_pixels: u32,
    pub blobs: u16,
    pub min_blob_size: u32,
    pub max_blob_size: u32,
    pub min_x: u16,
    pub max_x: u16,
    pub min_y: u16,
    pub max_y: u16,
    pub score: u16,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct UpdateStatRequest {
    pub monitor_id: Option<u32>,
    pub zone_id: Option<u32>,
    pub event_id: Option<u64>,
    pub frame_id: Option<u32>,
    pub pixel_diff: Option<u8>,
    pub alarm_pixels: Option<u32>,
    pub filter_pixels: Option<u32>,
    pub blob_pixels: Option<u32>,
    pub blobs: Option<u16>,
    pub min_blob_size: Option<u32>,
    pub max_blob_size: Option<u32>,
    pub min_x: Option<u16>,
    pub max_x: Option<u16>,
    pub min_y: Option<u16>,
    pub max_y: Option<u16>,
    pub score: Option<u16>,
}
