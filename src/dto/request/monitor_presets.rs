use crate::entity::sea_orm_active_enums::MonitorType;
use garde::Validate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, Validate)]
pub struct CreateMonitorPresetRequest {
    #[garde(skip)]
    pub model_id: Option<u32>,
    #[garde(length(chars, max = 64))]
    pub name: String,
    #[serde(rename = "type")]
    #[garde(skip)]
    pub r#type: MonitorType,
    #[garde(inner(length(max = 255)))]
    pub device: Option<String>,
    #[garde(skip)]
    pub channel: Option<u8>,
    #[garde(skip)]
    pub format: Option<u32>,
    #[garde(inner(length(chars, max = 16)))]
    pub protocol: Option<String>,
    #[garde(inner(length(chars, max = 16)))]
    pub method: Option<String>,
    #[garde(inner(length(chars, max = 64)))]
    pub host: Option<String>,
    #[garde(inner(length(chars, max = 8)))]
    pub port: Option<String>,
    #[garde(inner(length(chars, max = 255)))]
    pub path: Option<String>,
    #[garde(inner(length(chars, max = 64)))]
    pub sub_path: Option<String>,
    #[garde(skip)]
    pub width: Option<u16>,
    #[garde(skip)]
    pub height: Option<u16>,
    #[garde(skip)]
    pub palette: Option<u32>,
    #[schema(value_type = Option<f64>)]
    #[garde(skip)]
    pub max_fps: Option<Decimal>,
    #[garde(skip)]
    pub controllable: Option<u8>,
    #[garde(inner(length(chars, max = 16)))]
    pub control_id: Option<String>,
    #[garde(inner(length(chars, max = 255)))]
    pub control_device: Option<String>,
    #[garde(inner(length(chars, max = 255)))]
    pub control_address: Option<String>,
    #[garde(skip)]
    pub default_rate: Option<u16>,
    #[garde(inner(length(chars, max = 16)))]
    pub default_scale: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, Validate)]
pub struct UpdateMonitorPresetRequest {
    #[garde(skip)]
    pub model_id: Option<u32>,
    #[garde(inner(length(chars, max = 64)))]
    pub name: Option<String>,
    #[serde(rename = "type")]
    #[garde(skip)]
    pub r#type: Option<MonitorType>,
    #[garde(inner(length(max = 255)))]
    pub device: Option<String>,
    #[garde(skip)]
    pub channel: Option<u8>,
    #[garde(skip)]
    pub format: Option<u32>,
    #[garde(inner(length(chars, max = 16)))]
    pub protocol: Option<String>,
    #[garde(inner(length(chars, max = 16)))]
    pub method: Option<String>,
    #[garde(inner(length(chars, max = 64)))]
    pub host: Option<String>,
    #[garde(inner(length(chars, max = 8)))]
    pub port: Option<String>,
    #[garde(inner(length(chars, max = 255)))]
    pub path: Option<String>,
    #[garde(inner(length(chars, max = 64)))]
    pub sub_path: Option<String>,
    #[garde(skip)]
    pub width: Option<u16>,
    #[garde(skip)]
    pub height: Option<u16>,
    #[garde(skip)]
    pub palette: Option<u32>,
    #[schema(value_type = Option<f64>)]
    #[garde(skip)]
    pub max_fps: Option<Decimal>,
    #[garde(skip)]
    pub controllable: Option<u8>,
    #[garde(inner(length(chars, max = 16)))]
    pub control_id: Option<String>,
    #[garde(inner(length(chars, max = 255)))]
    pub control_device: Option<String>,
    #[garde(inner(length(chars, max = 255)))]
    pub control_address: Option<String>,
    #[garde(skip)]
    pub default_rate: Option<u16>,
    #[garde(inner(length(chars, max = 16)))]
    pub default_scale: Option<String>,
}
