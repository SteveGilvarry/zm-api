use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateModelRequest {
    pub name: String,
    pub manufacturer_id: Option<i32>,
}
