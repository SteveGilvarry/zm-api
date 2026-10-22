use crate::dto::PaginationParams;
use garde::Validate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Query parameters for listing stats.
#[derive(Debug, Default, Deserialize, Serialize, ToSchema, Validate)]
pub struct StatQueryParams {
    /// Page number (1-indexed)
    #[schema(example = 1)]
    #[garde(range(min = 1))]
    pub page: Option<u64>,

    /// Number of items per page (max 1000)
    #[schema(example = 25)]
    #[garde(range(min = 1, max = 1000))]
    pub page_size: Option<u64>,

    /// Only stats of this event.
    #[garde(skip)]
    pub event_id: Option<u64>,

    /// Only stats of this frame. `FrameId` restarts at 1 in every event, so
    /// this needs `event_id` alongside it.
    #[garde(custom(frame_needs_event(&self.event_id)))]
    pub frame_id: Option<u32>,

    /// Only stats of this zone.
    #[garde(skip)]
    pub zone_id: Option<u32>,
}

fn frame_needs_event(
    event_id: &Option<u64>,
) -> impl FnOnce(&Option<u32>, &()) -> garde::Result + '_ {
    move |frame_id, _| match (frame_id, event_id) {
        (Some(_), None) => Err(garde::Error::new("frame_id needs event_id")),
        _ => Ok(()),
    }
}

impl StatQueryParams {
    pub fn pagination(&self) -> PaginationParams {
        PaginationParams {
            page: self.page,
            page_size: self.page_size,
        }
    }
}

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

#[derive(Debug, Clone, Default, Deserialize, Serialize, ToSchema)]
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
