use crate::dto::PaginatedResponse;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ModelResponse {
    pub id: u32,
    pub name: String,
    pub manufacturer_id: Option<i32>,
}

impl From<&crate::entity::models::Model> for ModelResponse {
    fn from(m: &crate::entity::models::Model) -> Self {
        Self {
            id: m.id,
            name: m.name.clone(),
            manufacturer_id: m.manufacturer_id,
        }
    }
}

/// Paginated response for models
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct PaginatedModelsResponse {
    pub items: Vec<ModelResponse>,
    pub total: u64,
    pub per_page: u64,
    pub current_page: u64,
    pub last_page: u64,
}

impl From<PaginatedResponse<ModelResponse>> for PaginatedModelsResponse {
    fn from(r: PaginatedResponse<ModelResponse>) -> Self {
        Self {
            items: r.items,
            total: r.total,
            per_page: r.per_page,
            current_page: r.current_page,
            last_page: r.last_page,
        }
    }
}
