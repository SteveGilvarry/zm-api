use crate::entity::sea_orm_active_enums::DeviceType;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct CreateDeviceRequest {
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: DeviceType,
    pub key_string: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct UpdateDeviceRequest {
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub r#type: Option<DeviceType>,
    pub key_string: Option<String>,
}
