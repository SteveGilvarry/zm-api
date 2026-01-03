use crate::entity::devices::Model as DeviceModel;
use crate::entity::sea_orm_active_enums::DeviceType;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct DeviceResponse {
    pub id: u32,
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: DeviceType,
    pub key_string: String,
}

impl From<&DeviceModel> for DeviceResponse {
    fn from(model: &DeviceModel) -> Self {
        Self {
            id: model.id,
            name: model.name.clone(),
            r#type: model.r#type.clone(),
            key_string: model.key_string.clone(),
        }
    }
}
