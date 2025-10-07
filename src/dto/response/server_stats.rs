use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::entity::server_stats::Model as ServerStatModel;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ServerStatResponse {
    pub id: u32,
    pub server_id: Option<u32>,
    pub time_stamp: String,
    pub cpu_load: Option<String>,
    pub cpu_user_percent: Option<String>,
    pub cpu_nice_percent: Option<String>,
    pub cpu_system_percent: Option<String>,
    pub cpu_idle_percent: Option<String>,
    pub cpu_usage_percent: Option<String>,
    pub total_mem: Option<u64>,
    pub free_mem: Option<u64>,
    pub total_swap: Option<u64>,
    pub free_swap: Option<u64>,
}

impl From<&ServerStatModel> for ServerStatResponse {
    fn from(model: &ServerStatModel) -> Self {
        Self {
            id: model.id,
            server_id: model.server_id,
            time_stamp: model.time_stamp.to_rfc3339(),
            cpu_load: model.cpu_load.map(|d| d.to_string()),
            cpu_user_percent: model.cpu_user_percent.map(|d| d.to_string()),
            cpu_nice_percent: model.cpu_nice_percent.map(|d| d.to_string()),
            cpu_system_percent: model.cpu_system_percent.map(|d| d.to_string()),
            cpu_idle_percent: model.cpu_idle_percent.map(|d| d.to_string()),
            cpu_usage_percent: model.cpu_usage_percent.map(|d| d.to_string()),
            total_mem: model.total_mem,
            free_mem: model.free_mem,
            total_swap: model.total_swap,
            free_swap: model.free_swap,
        }
    }
}
