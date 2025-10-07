use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use rust_decimal::Decimal;
use crate::entity::sea_orm_active_enums::MonitorType;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct CreateMonitorPresetRequest {
    pub model_id: Option<u32>,
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: MonitorType,
    pub device: Option<String>,
    pub channel: Option<String>,
    pub format: Option<u32>,
    pub protocol: Option<String>,
    pub method: Option<String>,
    pub host: Option<String>,
    pub port: Option<String>,
    pub path: Option<String>,
    pub sub_path: Option<String>,
    pub width: Option<u16>,
    pub height: Option<u16>,
    pub palette: Option<u32>,
    #[schema(value_type = Option<f64>)]
    pub max_fps: Option<Decimal>,
    pub controllable: Option<u8>,
    pub control_id: Option<String>,
    pub control_device: Option<String>,
    pub control_address: Option<String>,
    pub default_rate: Option<u16>,
    pub default_scale: Option<u16>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct UpdateMonitorPresetRequest {
    pub model_id: Option<u32>,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub r#type: Option<MonitorType>,
    pub device: Option<String>,
    pub channel: Option<String>,
    pub format: Option<u32>,
    pub protocol: Option<String>,
    pub method: Option<String>,
    pub host: Option<String>,
    pub port: Option<String>,
    pub path: Option<String>,
    pub sub_path: Option<String>,
    pub width: Option<u16>,
    pub height: Option<u16>,
    pub palette: Option<u32>,
    #[schema(value_type = Option<f64>)]
    pub max_fps: Option<Decimal>,
    pub controllable: Option<u8>,
    pub control_id: Option<String>,
    pub control_device: Option<String>,
    pub control_address: Option<String>,
    pub default_rate: Option<u16>,
    pub default_scale: Option<u16>,
}
