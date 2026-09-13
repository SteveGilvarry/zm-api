use garde::Validate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, Validate)]
pub struct CreateTriggerX10Request {
    #[garde(skip)]
    pub monitor_id: u32,
    #[garde(inner(length(chars, max = 32)))]
    pub activation: Option<String>,
    #[garde(inner(length(chars, max = 32)))]
    pub alarm_input: Option<String>,
    #[garde(inner(length(chars, max = 32)))]
    pub alarm_output: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, Validate)]
pub struct UpdateTriggerX10Request {
    #[garde(inner(length(chars, max = 32)))]
    pub activation: Option<String>,
    #[garde(inner(length(chars, max = 32)))]
    pub alarm_input: Option<String>,
    #[garde(inner(length(chars, max = 32)))]
    pub alarm_output: Option<String>,
}
