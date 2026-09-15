use garde::Validate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::dto::serde_helpers::double_option;

/// Largest value a `decimal(10,2) unsigned` zone column holds.
const DECIMAL_10_2_MAX: f64 = 99_999_999.99;

#[derive(Debug, Default, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateZoneRequest {
    #[garde(length(min = 1, max = 64))]
    pub name: String,
    /// `Active`, `Inclusive`, `Exclusive`, `Preclusive`, `Inactive` or `Privacy`.
    #[garde(length(max = 32))]
    pub r#type: String,
    /// `Pixels` or `Percent`.
    #[garde(length(max = 32))]
    pub units: String,
    /// Polygon, `"x1,y1 x2,y2 ..."`. `Zones.Coords` is TINYTEXT (255 bytes).
    #[garde(length(max = 255))]
    pub coords: String,
    /// Ignored: the count is taken from `coords`. Kept so existing clients
    /// that send it still validate.
    #[garde(skip)]
    #[serde(default)]
    pub num_coords: u8,
    /// `AlarmedPixels` (default), `FilteredPixels` or `Blobs`.
    #[garde(inner(length(max = 32)))]
    #[serde(default)]
    pub check_method: Option<String>,
    #[garde(skip)]
    #[serde(default)]
    pub alarm_rgb: Option<u32>,
    /// Pixel difference, 0-255.
    #[garde(inner(range(max = 255)))]
    #[serde(default)]
    pub min_pixel_threshold: Option<u16>,
    #[garde(inner(range(max = 255)))]
    #[serde(default)]
    pub max_pixel_threshold: Option<u16>,
    #[garde(inner(range(min = 0.0, max = DECIMAL_10_2_MAX)))]
    #[serde(default)]
    pub min_alarm_pixels: Option<f64>,
    #[garde(inner(range(min = 0.0, max = DECIMAL_10_2_MAX)))]
    #[serde(default)]
    pub max_alarm_pixels: Option<f64>,
    #[garde(skip)]
    #[serde(default)]
    pub filter_x: Option<u8>,
    #[garde(skip)]
    #[serde(default)]
    pub filter_y: Option<u8>,
    #[garde(inner(range(min = 0.0, max = DECIMAL_10_2_MAX)))]
    #[serde(default)]
    pub min_filter_pixels: Option<f64>,
    #[garde(inner(range(min = 0.0, max = DECIMAL_10_2_MAX)))]
    #[serde(default)]
    pub max_filter_pixels: Option<f64>,
    #[garde(inner(range(min = 0.0, max = DECIMAL_10_2_MAX)))]
    #[serde(default)]
    pub min_blob_pixels: Option<f64>,
    #[garde(inner(range(min = 0.0, max = DECIMAL_10_2_MAX)))]
    #[serde(default)]
    pub max_blob_pixels: Option<f64>,
    #[garde(skip)]
    #[serde(default)]
    pub min_blobs: Option<u16>,
    #[garde(skip)]
    #[serde(default)]
    pub max_blobs: Option<u16>,
    #[garde(skip)]
    #[serde(default)]
    pub overload_frames: Option<u16>,
    #[garde(skip)]
    #[serde(default)]
    pub extend_alarm_frames: Option<u16>,
}

/// Partial update: a field left out is unchanged; `null` clears a nullable one.
#[derive(Debug, Default, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateZoneRequest {
    #[garde(inner(length(min = 1, max = 64)))]
    #[serde(default)]
    pub name: Option<String>,
    #[garde(inner(length(max = 32)))]
    #[serde(default)]
    pub r#type: Option<String>,
    #[garde(inner(length(max = 32)))]
    #[serde(default)]
    pub units: Option<String>,
    /// New polygon; `num_coords` and `area` are recomputed from it.
    #[garde(inner(length(max = 255)))]
    #[serde(default)]
    pub coords: Option<String>,
    /// Earlier name for `coords`, still accepted.
    #[garde(inner(length(max = 255)))]
    #[serde(default)]
    pub polygon: Option<String>,
    #[garde(inner(length(max = 32)))]
    #[serde(default)]
    pub check_method: Option<String>,
    #[garde(skip)]
    #[serde(default, deserialize_with = "double_option")]
    #[schema(value_type = Option<u32>)]
    pub alarm_rgb: Option<Option<u32>>,
    #[garde(inner(inner(range(max = 255))))]
    #[serde(default, deserialize_with = "double_option")]
    #[schema(value_type = Option<u16>)]
    pub min_pixel_threshold: Option<Option<u16>>,
    #[garde(inner(inner(range(max = 255))))]
    #[serde(default, deserialize_with = "double_option")]
    #[schema(value_type = Option<u16>)]
    pub max_pixel_threshold: Option<Option<u16>>,
    #[garde(inner(inner(range(min = 0.0, max = DECIMAL_10_2_MAX))))]
    #[serde(default, deserialize_with = "double_option")]
    #[schema(value_type = Option<f64>)]
    pub min_alarm_pixels: Option<Option<f64>>,
    #[garde(inner(inner(range(min = 0.0, max = DECIMAL_10_2_MAX))))]
    #[serde(default, deserialize_with = "double_option")]
    #[schema(value_type = Option<f64>)]
    pub max_alarm_pixels: Option<Option<f64>>,
    #[garde(skip)]
    #[serde(default, deserialize_with = "double_option")]
    #[schema(value_type = Option<u8>)]
    pub filter_x: Option<Option<u8>>,
    #[garde(skip)]
    #[serde(default, deserialize_with = "double_option")]
    #[schema(value_type = Option<u8>)]
    pub filter_y: Option<Option<u8>>,
    #[garde(inner(inner(range(min = 0.0, max = DECIMAL_10_2_MAX))))]
    #[serde(default, deserialize_with = "double_option")]
    #[schema(value_type = Option<f64>)]
    pub min_filter_pixels: Option<Option<f64>>,
    #[garde(inner(inner(range(min = 0.0, max = DECIMAL_10_2_MAX))))]
    #[serde(default, deserialize_with = "double_option")]
    #[schema(value_type = Option<f64>)]
    pub max_filter_pixels: Option<Option<f64>>,
    #[garde(inner(inner(range(min = 0.0, max = DECIMAL_10_2_MAX))))]
    #[serde(default, deserialize_with = "double_option")]
    #[schema(value_type = Option<f64>)]
    pub min_blob_pixels: Option<Option<f64>>,
    #[garde(inner(inner(range(min = 0.0, max = DECIMAL_10_2_MAX))))]
    #[serde(default, deserialize_with = "double_option")]
    #[schema(value_type = Option<f64>)]
    pub max_blob_pixels: Option<Option<f64>>,
    #[garde(skip)]
    #[serde(default, deserialize_with = "double_option")]
    #[schema(value_type = Option<u16>)]
    pub min_blobs: Option<Option<u16>>,
    #[garde(skip)]
    #[serde(default, deserialize_with = "double_option")]
    #[schema(value_type = Option<u16>)]
    pub max_blobs: Option<Option<u16>>,
    #[garde(skip)]
    #[serde(default)]
    pub overload_frames: Option<u16>,
    #[garde(skip)]
    #[serde(default)]
    pub extend_alarm_frames: Option<u16>,
}
