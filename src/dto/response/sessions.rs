use crate::dto::PaginatedResponse;
use crate::entity::sessions::Model as SessionModel;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct SessionResponse {
    pub id: String,
    pub access: Option<u32>,
    pub data: Option<String>,
}

impl From<&SessionModel> for SessionResponse {
    fn from(model: &SessionModel) -> Self {
        Self {
            id: model.id.clone(),
            access: model.access,
            data: model.data.clone(),
        }
    }
}

/// Paginated response for sessions
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct PaginatedSessionsResponse {
    pub items: Vec<SessionResponse>,
    pub total: u64,
    pub per_page: u64,
    pub current_page: u64,
    pub last_page: u64,
}

impl From<PaginatedResponse<SessionResponse>> for PaginatedSessionsResponse {
    fn from(r: PaginatedResponse<SessionResponse>) -> Self {
        Self {
            items: r.items,
            total: r.total,
            per_page: r.per_page,
            current_page: r.current_page,
            last_page: r.last_page,
        }
    }
}
