use crate::dto::PaginatedResponse;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
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

/// Paginated response for users
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct PaginatedUsersResponse {
    pub items: Vec<UserResponse>,
    pub total: u64,
    pub per_page: u64,
    pub current_page: u64,
    pub last_page: u64,
}

impl From<PaginatedResponse<UserResponse>> for PaginatedUsersResponse {
    fn from(r: PaginatedResponse<UserResponse>) -> Self {
        Self {
            items: r.items,
            total: r.total,
            per_page: r.per_page,
            current_page: r.current_page,
            last_page: r.last_page,
        }
    }
}
