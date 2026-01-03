use crate::entity::snapshots_events::Model as SnapshotEventModel;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SnapshotEventResponse {
    pub id: u32,
    pub snapshot_id: u32,
    pub event_id: u64,
}

impl From<&SnapshotEventModel> for SnapshotEventResponse {
    fn from(model: &SnapshotEventModel) -> Self {
        Self {
            id: model.id,
            snapshot_id: model.snapshot_id,
            event_id: model.event_id,
        }
    }
}
