use crate::entity::stats::Model as StatModel;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct StatResponse {
    pub id: u32,
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

impl From<&StatModel> for StatResponse {
    fn from(model: &StatModel) -> Self {
        Self {
            id: model.id,
            monitor_id: model.monitor_id,
            zone_id: model.zone_id,
            event_id: model.event_id,
            frame_id: model.frame_id,
            pixel_diff: model.pixel_diff,
            alarm_pixels: model.alarm_pixels,
            filter_pixels: model.filter_pixels,
            blob_pixels: model.blob_pixels,
            blobs: model.blobs,
            min_blob_size: model.min_blob_size,
            max_blob_size: model.max_blob_size,
            min_x: model.min_x,
            max_x: model.max_x,
            min_y: model.min_y,
            max_y: model.max_y,
            score: model.score,
        }
    }
}
