use crate::dto::PaginatedResponse;
use crate::entity::frames::Model as FrameModel;
use crate::entity::sea_orm_active_enums::FrameType;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct FrameResponse {
    /// Frame ID
    pub id: u64,
    /// Event ID this frame belongs to
    pub event_id: u64,
    /// Frame sequence number within the event
    pub frame_id: u32,
    /// Frame type (Normal, Bulk, Alarm)
    #[schema(value_type = String)]
    pub r#type: FrameType,
    /// Timestamp of the frame
    #[schema(value_type = String, example = "2025-01-15T10:30:00Z")]
    pub time_stamp: String,
    /// Time delta from previous frame
    #[schema(value_type = String, example = "0.05")]
    pub delta: Decimal,
    /// Motion detection score
    pub score: u16,
}

impl From<&FrameModel> for FrameResponse {
    fn from(model: &FrameModel) -> Self {
        Self {
            id: model.id,
            event_id: model.event_id,
            frame_id: model.frame_id,
            r#type: model.r#type.clone(),
            time_stamp: model.time_stamp.to_string(),
            delta: model.delta,
            score: model.score,
        }
    }
}

/// Paginated response for frames
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct PaginatedFramesResponse {
    pub items: Vec<FrameResponse>,
    pub total: u64,
    pub per_page: u64,
    pub current_page: u64,
    pub last_page: u64,
}

impl From<PaginatedResponse<FrameResponse>> for PaginatedFramesResponse {
    fn from(r: PaginatedResponse<FrameResponse>) -> Self {
        Self {
            items: r.items,
            total: r.total,
            per_page: r.per_page,
            current_page: r.current_page,
            last_page: r.last_page,
        }
    }
}
