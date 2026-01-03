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
