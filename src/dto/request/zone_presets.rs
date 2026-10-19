use garde::Validate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateZonePresetRequest {
    #[garde(length(chars, max = 64))]
    pub name: String,
    #[garde(skip)]
    pub r#type: String,
    #[garde(skip)]
    pub units: String,
    #[garde(skip)]
    pub check_method: String,
    /// The thresholds the preset applies to a zone (#188).
    #[serde(flatten)]
    #[garde(skip)]
    pub thresholds: ZonePresetThresholds,
}

/// A zone preset's thresholds, named as on `ZoneResponse` so a client can copy
/// them to a zone one to one. On update, an omitted field is left alone.
#[derive(Debug, Default, Clone, Serialize, Deserialize, ToSchema)]
pub struct ZonePresetThresholds {
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
    pub overload_frames: Option<u16>,
    pub extend_alarm_frames: Option<u16>,
}

impl ZonePresetThresholds {
    /// Set every provided threshold on the active model.
    pub fn apply(&self, am: &mut crate::entity::zone_presets::ActiveModel) {
        use rust_decimal::prelude::FromPrimitive;
        use rust_decimal::Decimal;
        use sea_orm::Set;
        let dec = |v: f64| Decimal::from_f64(v);
        macro_rules! set {
            ($f:ident) => {
                if let Some(v) = self.$f {
                    am.$f = Set(Some(v));
                }
            };
            ($f:ident, dec) => {
                if let Some(v) = self.$f {
                    am.$f = Set(dec(v));
                }
            };
        }
        set!(min_pixel_threshold);
        set!(max_pixel_threshold);
        set!(min_alarm_pixels, dec);
        set!(max_alarm_pixels, dec);
        set!(filter_x);
        set!(filter_y);
        set!(min_filter_pixels, dec);
        set!(max_filter_pixels, dec);
        set!(min_blob_pixels, dec);
        set!(max_blob_pixels, dec);
        set!(min_blobs);
        set!(max_blobs);
        if let Some(v) = self.overload_frames {
            am.overload_frames = Set(v);
        }
        if let Some(v) = self.extend_alarm_frames {
            am.extend_alarm_frames = Set(v);
        }
    }
}
