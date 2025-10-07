use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateObjectTypeRequest {
    #[schema(example = "person")]
    pub name: Option<String>,
    #[schema(example = "Person")]
    pub human: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateObjectTypeRequest {
    #[schema(example = "person")]
    pub name: Option<String>,
    #[schema(example = "Person")]
    pub human: Option<String>,
}
