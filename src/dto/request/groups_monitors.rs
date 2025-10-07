use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateGroupMonitorRequest {
    #[schema(example = 1)]
    pub group_id: u32,
    #[schema(example = 1)]
    pub monitor_id: u32,
}
