use crate::dto::PaginatedResponse;
use crate::entity::snapshots_events::Model as SnapshotEventModel;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
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

/// Paginated response for snapshot-event associations
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct PaginatedSnapshotEventsResponse {
    pub items: Vec<SnapshotEventResponse>,
    pub total: u64,
    pub per_page: u64,
    pub current_page: u64,
    pub last_page: u64,
}

impl From<PaginatedResponse<SnapshotEventResponse>> for PaginatedSnapshotEventsResponse {
    fn from(r: PaginatedResponse<SnapshotEventResponse>) -> Self {
        Self {
            items: r.items,
            total: r.total,
            per_page: r.per_page,
            current_page: r.current_page,
            last_page: r.last_page,
        }
    }
}
