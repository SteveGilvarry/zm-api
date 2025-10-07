use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use sea_orm::prelude::DateTimeUtc;
use crate::entity::tags::Model as TagModel;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct TagResponse {
    pub id: u64,
    pub name: String,
    #[schema(value_type = Option<String>, example = "2025-01-15T10:30:00Z")]
    pub create_date: Option<DateTimeUtc>,
}

impl From<&TagModel> for TagResponse {
    fn from(model: &TagModel) -> Self {
        Self {
            id: model.id,
            name: model.name.clone(),
            create_date: model.create_date,
        }
    }
}
