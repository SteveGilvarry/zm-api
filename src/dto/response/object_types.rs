use crate::dto::PaginatedResponse;
use crate::entity::object_types::Model as ObjectTypeModel;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ObjectTypeResponse {
    pub id: i32,
    pub name: Option<String>,
    pub human: Option<String>,
}

impl From<&ObjectTypeModel> for ObjectTypeResponse {
    fn from(model: &ObjectTypeModel) -> Self {
        Self {
            id: model.id,
            name: model.name.clone(),
            human: model.human.clone(),
        }
    }
}

/// Paginated response for object types
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct PaginatedObjectTypesResponse {
    pub items: Vec<ObjectTypeResponse>,
    pub total: u64,
    pub per_page: u64,
    pub current_page: u64,
    pub last_page: u64,
}

impl From<PaginatedResponse<ObjectTypeResponse>> for PaginatedObjectTypesResponse {
    fn from(r: PaginatedResponse<ObjectTypeResponse>) -> Self {
        Self {
            items: r.items,
            total: r.total,
            per_page: r.per_page,
            current_page: r.current_page,
            last_page: r.last_page,
        }
    }
}
