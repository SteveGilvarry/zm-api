use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct CreateTriggerX10Request {
    pub monitor_id: u32,
    pub activation: Option<String>,
    pub alarm_input: Option<String>,
    pub alarm_output: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct UpdateTriggerX10Request {
    pub activation: Option<String>,
    pub alarm_input: Option<String>,
    pub alarm_output: Option<String>,
}
