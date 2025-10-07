use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateServerStatRequest {
    #[schema(example = 1)]
    pub server_id: Option<u32>,
    #[schema(value_type = String, example = "1.5")]
    pub cpu_load: Option<String>,
    #[schema(value_type = String, example = "25.5")]
    pub cpu_user_percent: Option<String>,
    #[schema(value_type = String, example = "0.1")]
    pub cpu_nice_percent: Option<String>,
    #[schema(value_type = String, example = "5.2")]
    pub cpu_system_percent: Option<String>,
    #[schema(value_type = String, example = "69.2")]
    pub cpu_idle_percent: Option<String>,
    #[schema(value_type = String, example = "30.8")]
    pub cpu_usage_percent: Option<String>,
    #[schema(example = 8192000)]
    pub total_mem: Option<u64>,
    #[schema(example = 4096000)]
    pub free_mem: Option<u64>,
    #[schema(example = 2048000)]
    pub total_swap: Option<u64>,
    #[schema(example = 1024000)]
    pub free_swap: Option<u64>,
}
