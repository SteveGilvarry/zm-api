use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::entity::sessions::Model as SessionModel;

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
