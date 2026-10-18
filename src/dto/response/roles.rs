use crate::dto::PaginatedResponse;
use crate::entity::user_roles::Model as RoleModel;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// A ZoneMinder 1.39 role. A user holding it (`role_id`) gets, per feature,
/// the higher of their own level and the role's.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RoleResponse {
    pub id: u32,
    pub name: String,
    pub description: Option<String>,
    pub stream: String,
    pub events: String,
    pub control: String,
    pub monitors: String,
    pub groups: String,
    pub devices: String,
    pub snapshots: String,
    pub system: String,
}

impl From<&RoleModel> for RoleResponse {
    fn from(m: &RoleModel) -> Self {
        Self {
            id: m.id,
            name: m.name.clone(),
            description: m.description.clone(),
            stream: format!("{:?}", m.stream),
            events: format!("{:?}", m.events),
            control: format!("{:?}", m.control),
            monitors: format!("{:?}", m.monitors),
            groups: format!("{:?}", m.groups),
            devices: format!("{:?}", m.devices),
            snapshots: format!("{:?}", m.snapshots),
            system: format!("{:?}", m.system),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PaginatedRolesResponse {
    pub items: Vec<RoleResponse>,
    pub total: u64,
    pub per_page: u64,
    pub current_page: u64,
    pub last_page: u64,
}

impl From<PaginatedResponse<RoleResponse>> for PaginatedRolesResponse {
    fn from(r: PaginatedResponse<RoleResponse>) -> Self {
        Self {
            items: r.items,
            total: r.total,
            per_page: r.per_page,
            current_page: r.current_page,
            last_page: r.last_page,
        }
    }
}
