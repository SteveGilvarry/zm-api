use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserResponse {
    pub id: u32,
    pub username: String,
    pub name: String,
    pub email: String,
    pub enabled: u8,
    pub system: String,
    pub stream: String,
    pub events: String,
    pub control: String,
    pub monitors: String,
    pub groups: String,
    pub devices: String,
    pub snapshots: String,
}

impl From<&crate::entity::users::Model> for UserResponse {
    fn from(m: &crate::entity::users::Model) -> Self {
        Self {
            id: m.id,
            username: m.username.clone(),
            name: m.name.clone(),
            email: m.email.clone(),
            enabled: m.enabled,
            system: format!("{:?}", m.system),
            stream: format!("{:?}", m.stream),
            events: format!("{:?}", m.events),
            control: format!("{:?}", m.control),
            monitors: format!("{:?}", m.monitors),
            groups: format!("{:?}", m.groups),
            devices: format!("{:?}", m.devices),
            snapshots: format!("{:?}", m.snapshots),
        }
    }
}
