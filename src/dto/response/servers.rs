use crate::dto::PaginatedResponse;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ServerResponse {
    pub id: u32,
    pub name: String,
    pub hostname: Option<String>,
    pub port: Option<u32>,
    pub status: String,
}

impl From<&crate::entity::servers::Model> for ServerResponse {
    fn from(m: &crate::entity::servers::Model) -> Self {
        Self {
            id: m.id,
            name: m.name.clone(),
            hostname: m.hostname.clone(),
            port: m.port,
            status: m.status.to_string(),
        }
    }
}

/// Paginated response for servers
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct PaginatedServersResponse {
    pub items: Vec<ServerResponse>,
    pub total: u64,
    pub per_page: u64,
    pub current_page: u64,
    pub last_page: u64,
}

impl From<PaginatedResponse<ServerResponse>> for PaginatedServersResponse {
    fn from(r: PaginatedResponse<ServerResponse>) -> Self {
        Self {
            items: r.items,
            total: r.total,
            per_page: r.per_page,
            current_page: r.current_page,
            last_page: r.last_page,
        }
    }
}
