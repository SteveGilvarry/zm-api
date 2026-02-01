use crate::dto::PaginatedResponse;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FilterResponse {
    pub id: u32,
    pub name: String,
    pub user_id: Option<u32>,
    pub execute_interval: u32,
    pub query_json: String,
    pub auto_archive: u8,
    pub auto_delete: u8,
}

impl From<&crate::entity::filters::Model> for FilterResponse {
    fn from(m: &crate::entity::filters::Model) -> Self {
        Self {
            id: m.id,
            name: m.name.clone(),
            user_id: m.user_id,
            execute_interval: m.execute_interval,
            query_json: m.query_json.clone(),
            auto_archive: m.auto_archive,
            auto_delete: m.auto_delete,
        }
    }
}

/// Paginated response for filters
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct PaginatedFiltersResponse {
    pub items: Vec<FilterResponse>,
    pub total: u64,
    pub per_page: u64,
    pub current_page: u64,
    pub last_page: u64,
}

impl From<PaginatedResponse<FilterResponse>> for PaginatedFiltersResponse {
    fn from(r: PaginatedResponse<FilterResponse>) -> Self {
        Self {
            items: r.items,
            total: r.total,
            per_page: r.per_page,
            current_page: r.current_page,
            last_page: r.last_page,
        }
    }
}
