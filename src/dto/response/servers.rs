use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
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
