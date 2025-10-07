use crate::entity::sea_orm_active_enums::FrameType;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct CreateFrameRequest {
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

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct UpdateFrameRequest {
    /// Frame type (Normal, Bulk, Alarm)
    #[schema(value_type = String)]
    pub r#type: Option<FrameType>,
    /// Motion detection score
    pub score: Option<u16>,
}
