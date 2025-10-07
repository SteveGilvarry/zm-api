use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateSnapshotEventRequest {
    #[schema(example = 1)]
    pub snapshot_id: u32,
    #[schema(example = 1)]
    pub event_id: u64,
}
