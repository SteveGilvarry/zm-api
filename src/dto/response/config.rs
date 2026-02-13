use crate::dto::PaginatedResponse;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ConfigResponse {
    pub id: u16,
    pub name: String,
    pub value: String,
    pub r#type: String,
    pub default_value: Option<String>,
    pub hint: Option<String>,
    pub pattern: Option<String>,
    pub format: Option<String>,
    pub prompt: Option<String>,
    pub help: Option<String>,
    pub category: String,
    pub readonly: u8,
    pub private: i8,
    pub system: i8,
}

/// Paginated response for configs
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct PaginatedConfigsResponse {
    pub items: Vec<ConfigResponse>,
    pub total: u64,
    pub per_page: u64,
    pub current_page: u64,
    pub last_page: u64,
}

impl From<PaginatedResponse<ConfigResponse>> for PaginatedConfigsResponse {
    fn from(r: PaginatedResponse<ConfigResponse>) -> Self {
        Self {
            items: r.items,
            total: r.total,
            per_page: r.per_page,
            current_page: r.current_page,
            last_page: r.last_page,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CategoryCountResponse {
    pub category: String,
    pub count: u64,
}
