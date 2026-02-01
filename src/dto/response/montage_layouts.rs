use crate::dto::PaginatedResponse;
use crate::entity::montage_layouts::Model as MontageLayoutModel;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct MontageLayoutResponse {
    pub id: u32,
    pub name: String,
    pub user_id: u32,
    pub positions: Option<String>,
}

impl From<&MontageLayoutModel> for MontageLayoutResponse {
    fn from(model: &MontageLayoutModel) -> Self {
        Self {
            id: model.id,
            name: model.name.clone(),
            user_id: model.user_id,
            positions: model.positions.clone(),
        }
    }
}

/// Paginated response for montage layouts
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct PaginatedMontageLayoutsResponse {
    pub items: Vec<MontageLayoutResponse>,
    pub total: u64,
    pub per_page: u64,
    pub current_page: u64,
    pub last_page: u64,
}

impl From<PaginatedResponse<MontageLayoutResponse>> for PaginatedMontageLayoutsResponse {
    fn from(r: PaginatedResponse<MontageLayoutResponse>) -> Self {
        Self {
            items: r.items,
            total: r.total,
            per_page: r.per_page,
            current_page: r.current_page,
            last_page: r.last_page,
        }
    }
}
