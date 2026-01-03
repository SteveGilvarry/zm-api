use crate::entity::snapshots::Model as SnapshotModel;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct SnapshotResponse {
    pub id: u32,
    pub name: Option<String>,
    pub description: Option<String>,
    pub created_by: Option<i32>,
    #[schema(value_type = Option<String>, example = "2025-01-15T10:30:00")]
    pub created_on: Option<NaiveDateTime>,
}

impl From<&SnapshotModel> for SnapshotResponse {
    fn from(model: &SnapshotModel) -> Self {
        Self {
            id: model.id,
            name: model.name.clone(),
            description: model.description.clone(),
            created_by: model.created_by,
            created_on: model.created_on,
        }
    }
}
