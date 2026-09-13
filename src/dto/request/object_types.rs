use garde::Validate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateObjectTypeRequest {
    #[schema(example = "person")]
    #[garde(inner(length(chars, max = 32)))]
    pub name: Option<String>,
    #[schema(example = "Person")]
    #[garde(skip)]
    pub human: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateObjectTypeRequest {
    #[schema(example = "person")]
    #[garde(inner(length(chars, max = 32)))]
    pub name: Option<String>,
    #[schema(example = "Person")]
    #[garde(skip)]
    pub human: Option<String>,
}
