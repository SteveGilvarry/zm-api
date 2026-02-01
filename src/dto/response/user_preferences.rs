use crate::dto::PaginatedResponse;
use crate::entity::user_preferences::Model as UserPreferenceModel;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct UserPreferenceResponse {
    pub id: u32,
    pub user_id: u32,
    pub name: Option<String>,
    pub value: Option<String>,
}

impl From<&UserPreferenceModel> for UserPreferenceResponse {
    fn from(model: &UserPreferenceModel) -> Self {
        Self {
            id: model.id,
            user_id: model.user_id,
            name: model.name.clone(),
            value: model.value.clone(),
        }
    }
}

/// Paginated response for user preferences
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct PaginatedUserPreferencesResponse {
    pub items: Vec<UserPreferenceResponse>,
    pub total: u64,
    pub per_page: u64,
    pub current_page: u64,
    pub last_page: u64,
}

impl From<PaginatedResponse<UserPreferenceResponse>> for PaginatedUserPreferencesResponse {
    fn from(r: PaginatedResponse<UserPreferenceResponse>) -> Self {
        Self {
            items: r.items,
            total: r.total,
            per_page: r.per_page,
            current_page: r.current_page,
            last_page: r.last_page,
        }
    }
}
