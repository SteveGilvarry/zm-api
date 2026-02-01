use crate::dto::PaginatedResponse;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GroupResponse {
    pub id: u32,
    pub name: String,
    pub parent_id: Option<u32>,
}

impl From<&crate::entity::groups::Model> for GroupResponse {
    fn from(m: &crate::entity::groups::Model) -> Self {
        Self {
            id: m.id,
            name: m.name.clone(),
            parent_id: m.parent_id,
        }
    }
}

/// Paginated response for groups
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct PaginatedGroupsResponse {
    pub items: Vec<GroupResponse>,
    pub total: u64,
    pub per_page: u64,
    pub current_page: u64,
    pub last_page: u64,
}

impl From<PaginatedResponse<GroupResponse>> for PaginatedGroupsResponse {
    fn from(r: PaginatedResponse<GroupResponse>) -> Self {
        Self {
            items: r.items,
            total: r.total,
            per_page: r.per_page,
            current_page: r.current_page,
            last_page: r.last_page,
        }
    }
}
