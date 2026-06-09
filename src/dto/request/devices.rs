use crate::entity::sea_orm_active_enums::DeviceType;
use garde::Validate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// DB caps verified against the live ZoneMinder schema:
//   Devices.Name      → tinytext (255)
//   Devices.KeyString → varchar(32)

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, Validate)]
#[garde(allow_unvalidated)]
pub struct CreateDeviceRequest {
    #[garde(length(min = 1, max = 255))]
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: DeviceType,
    #[garde(length(max = 32))]
    pub key_string: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, Validate)]
#[garde(allow_unvalidated)]
pub struct UpdateDeviceRequest {
    #[garde(inner(length(min = 1, max = 255)))]
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub r#type: Option<DeviceType>,
    #[garde(inner(length(max = 32)))]
    pub key_string: Option<String>,
}
