use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LogResponse {
    pub id: u32,
    pub component: String,
    pub level: i8,
    pub code: String,
    pub message: String,
}

impl From<&crate::entity::logs::Model> for LogResponse {
    fn from(m: &crate::entity::logs::Model) -> Self {
        Self {
            id: m.id,
            component: m.component.clone(),
            level: m.level,
            code: m.code.clone(),
            message: m.message.clone(),
        }
    }
}

