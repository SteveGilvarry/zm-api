use crate::dto::PaginatedResponse;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ManufacturerResponse {
    pub id: u32,
    pub name: String,
}

impl From<&crate::entity::manufacturers::Model> for ManufacturerResponse {
    fn from(m: &crate::entity::manufacturers::Model) -> Self {
        Self {
            id: m.id,
            name: m.name.clone(),
        }
    }
}

/// Paginated response for manufacturers
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct PaginatedManufacturersResponse {
    pub items: Vec<ManufacturerResponse>,
    pub total: u64,
    pub per_page: u64,
    pub current_page: u64,
    pub last_page: u64,
}

impl From<PaginatedResponse<ManufacturerResponse>> for PaginatedManufacturersResponse {
    fn from(r: PaginatedResponse<ManufacturerResponse>) -> Self {
        Self {
            items: r.items,
            total: r.total,
            per_page: r.per_page,
            current_page: r.current_page,
            last_page: r.last_page,
        }
    }
}
