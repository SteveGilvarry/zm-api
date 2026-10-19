use crate::dto::PaginatedResponse;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ZonePresetResponse {
    pub id: u32,
    pub name: String,
    pub r#type: String,
    pub units: String,
    pub check_method: String,
    /// The thresholds the preset applies, named as on `ZoneResponse` (#188).
    pub min_pixel_threshold: Option<u16>,
    pub max_pixel_threshold: Option<u16>,
    pub min_alarm_pixels: Option<f64>,
    pub max_alarm_pixels: Option<f64>,
    pub filter_x: Option<u8>,
    pub filter_y: Option<u8>,
    pub min_filter_pixels: Option<f64>,
    pub max_filter_pixels: Option<f64>,
    pub min_blob_pixels: Option<f64>,
    pub max_blob_pixels: Option<f64>,
    pub min_blobs: Option<u16>,
    pub max_blobs: Option<u16>,
    pub overload_frames: u16,
    pub extend_alarm_frames: u16,
}

impl From<&crate::entity::zone_presets::Model> for ZonePresetResponse {
    fn from(m: &crate::entity::zone_presets::Model) -> Self {
        use rust_decimal::prelude::ToPrimitive;
        let dec = |d: Option<rust_decimal::Decimal>| d.and_then(|v| v.to_f64());
        Self {
            id: m.id,
            name: m.name.clone(),
            r#type: format!("{:?}", m.r#type),
            units: format!("{:?}", m.units),
            check_method: format!("{:?}", m.check_method),
            min_pixel_threshold: m.min_pixel_threshold,
            max_pixel_threshold: m.max_pixel_threshold,
            min_alarm_pixels: dec(m.min_alarm_pixels),
            max_alarm_pixels: dec(m.max_alarm_pixels),
            filter_x: m.filter_x,
            filter_y: m.filter_y,
            min_filter_pixels: dec(m.min_filter_pixels),
            max_filter_pixels: dec(m.max_filter_pixels),
            min_blob_pixels: dec(m.min_blob_pixels),
            max_blob_pixels: dec(m.max_blob_pixels),
            min_blobs: m.min_blobs,
            max_blobs: m.max_blobs,
            overload_frames: m.overload_frames,
            extend_alarm_frames: m.extend_alarm_frames,
        }
    }
}

/// Paginated response for zone presets
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct PaginatedZonePresetsResponse {
    pub items: Vec<ZonePresetResponse>,
    pub total: u64,
    pub per_page: u64,
    pub current_page: u64,
    pub last_page: u64,
}

impl From<PaginatedResponse<ZonePresetResponse>> for PaginatedZonePresetsResponse {
    fn from(r: PaginatedResponse<ZonePresetResponse>) -> Self {
        Self {
            items: r.items,
            total: r.total,
            per_page: r.per_page,
            current_page: r.current_page,
            last_page: r.last_page,
        }
    }
}
