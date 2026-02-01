use crate::dto::PaginatedResponse;
use crate::entity::states::Model as StateModel;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct StateResponse {
    pub id: u32,
    pub name: String,
    pub definition: String,
    pub is_active: u8,
}

impl From<&StateModel> for StateResponse {
    fn from(model: &StateModel) -> Self {
        Self {
            id: model.id,
            name: model.name.clone(),
            definition: model.definition.clone(),
            is_active: model.is_active,
        }
    }
}

/// Paginated response for states
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct PaginatedStatesResponse {
    pub items: Vec<StateResponse>,
    pub total: u64,
    pub per_page: u64,
    pub current_page: u64,
    pub last_page: u64,
}

impl From<PaginatedResponse<StateResponse>> for PaginatedStatesResponse {
    fn from(r: PaginatedResponse<StateResponse>) -> Self {
        Self {
            items: r.items,
            total: r.total,
            per_page: r.per_page,
            current_page: r.current_page,
            last_page: r.last_page,
        }
    }
}
