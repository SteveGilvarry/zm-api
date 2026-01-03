use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ConfigResponse {
    pub id: u16,
    pub name: String,
    pub value: String,
    pub r#type: String,
    pub default_value: Option<String>,
    pub hint: Option<String>,
    pub pattern: Option<String>,
    pub format: Option<String>,
    pub prompt: Option<String>,
    pub help: Option<String>,
    pub category: String,
    pub readonly: u8,
    pub private: i8,
    pub system: i8,
}
