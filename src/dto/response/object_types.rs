use crate::entity::object_types::Model as ObjectTypeModel;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ObjectTypeResponse {
    pub id: i32,
    pub name: Option<String>,
    pub human: Option<String>,
}

impl From<&ObjectTypeModel> for ObjectTypeResponse {
    fn from(model: &ObjectTypeModel) -> Self {
        Self {
            id: model.id,
            name: model.name.clone(),
            human: model.human.clone(),
        }
    }
}
