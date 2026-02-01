use crate::dto::PaginatedResponse;
use crate::entity::triggers_x10::Model as TriggerX10Model;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

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

/// Paginated response for X10 triggers
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct PaginatedTriggersX10Response {
    pub items: Vec<TriggerX10Response>,
    pub total: u64,
    pub per_page: u64,
    pub current_page: u64,
    pub last_page: u64,
}

impl From<PaginatedResponse<TriggerX10Response>> for PaginatedTriggersX10Response {
    fn from(r: PaginatedResponse<TriggerX10Response>) -> Self {
        Self {
            items: r.items,
            total: r.total,
            per_page: r.per_page,
            current_page: r.current_page,
            last_page: r.last_page,
        }
    }
}
