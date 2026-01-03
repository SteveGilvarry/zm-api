use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use rust_decimal::Decimal;
use crate::entity::monitor_presets::Model as MonitorPresetModel;
use crate::entity::sea_orm_active_enums::MonitorType;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct MonitorPresetResponse {
    pub id: u32,
    pub model_id: Option<u32>,
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: MonitorType,
    pub device: Option<String>,
    pub channel: Option<u8>,
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
    pub controllable: u8,
    pub control_id: Option<String>,
    pub control_device: Option<String>,
    pub control_address: Option<String>,
    pub default_rate: u16,
    pub default_scale: u16,
}

impl From<&MonitorPresetModel> for MonitorPresetResponse {
    fn from(model: &MonitorPresetModel) -> Self {
        Self {
            id: model.id,
            model_id: model.model_id,
            name: model.name.clone(),
            r#type: model.r#type.clone(),
            device: model.device.clone(),
            channel: model.channel.clone(),
            format: model.format,
            protocol: model.protocol.clone(),
            method: model.method.clone(),
            host: model.host.clone(),
            port: model.port.clone(),
            path: model.path.clone(),
            sub_path: model.sub_path.clone(),
            width: model.width,
            height: model.height,
            palette: model.palette,
            max_fps: model.max_fps,
            controllable: model.controllable,
            control_id: model.control_id.clone(),
            control_device: model.control_device.clone(),
            control_address: model.control_address.clone(),
            default_rate: model.default_rate,
            default_scale: model.default_scale,
        }
    }
}
