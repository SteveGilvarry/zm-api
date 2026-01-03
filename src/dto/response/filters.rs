use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct FilterResponse {
    pub id: u32,
    pub name: String,
    pub user_id: Option<u32>,
    pub execute_interval: u32,
    pub query_json: String,
    pub auto_archive: u8,
    pub auto_delete: u8,
}

impl From<&crate::entity::filters::Model> for FilterResponse {
    fn from(m: &crate::entity::filters::Model) -> Self {
        Self {
            id: m.id,
            name: m.name.clone(),
            user_id: m.user_id,
            execute_interval: m.execute_interval,
            query_json: m.query_json.clone(),
            auto_archive: m.auto_archive,
            auto_delete: m.auto_delete,
        }
    }
}
