use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::entity::triggers_x10::Model as TriggerX10Model;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct TriggerX10Response {
    pub monitor_id: u32,
    pub activation: Option<String>,
    pub alarm_input: Option<String>,
    pub alarm_output: Option<String>,
}

impl From<&TriggerX10Model> for TriggerX10Response {
    fn from(model: &TriggerX10Model) -> Self {
        Self {
            monitor_id: model.monitor_id,
            activation: model.activation.clone(),
            alarm_input: model.alarm_input.clone(),
            alarm_output: model.alarm_output.clone(),
        }
    }
}
