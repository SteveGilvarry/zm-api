use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::dto::PaginatedResponse;
use crate::entity::event_summaries::Model as EventSummaryModel;

/// Event counts and disk space summary for a monitor
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct EventSummaryResponse {
    /// Monitor ID
    #[schema(example = 1)]
    pub monitor_id: u32,

    /// Total event count (all time)
    #[schema(example = 1250)]
    pub total_events: i32,

    /// Total disk space used by events (bytes)
    #[schema(example = 536870912_i64)]
    pub total_event_disk_space: i64,

    /// Events in the last hour
    #[schema(example = 5)]
    pub hour_events: i32,

    /// Disk space used by events in the last hour (bytes)
    #[schema(example = 52428800_i64)]
    pub hour_event_disk_space: i64,

    /// Events in the last day
    #[schema(example = 42)]
    pub day_events: i32,

    /// Disk space used by events in the last day (bytes)
    #[schema(example = 524288000_i64)]
    pub day_event_disk_space: i64,

    /// Events in the last week
    #[schema(example = 287)]
    pub week_events: i32,

    /// Disk space used by events in the last week (bytes)
    #[schema(example = 1073741824_i64)]
    pub week_event_disk_space: i64,

    /// Events in the last month
    #[schema(example = 1100)]
    pub month_events: i32,

    /// Disk space used by events in the last month (bytes)
    #[schema(example = 2147483648_i64)]
    pub month_event_disk_space: i64,

    /// Archived event count
    #[schema(example = 25)]
    pub archived_events: i32,

    /// Disk space used by archived events (bytes)
    #[schema(example = 268435456_i64)]
    pub archived_event_disk_space: i64,
}

impl From<EventSummaryModel> for EventSummaryResponse {
    fn from(model: EventSummaryModel) -> Self {
        Self {
            monitor_id: model.monitor_id,
            total_events: model.total_events.unwrap_or(0),
            total_event_disk_space: model.total_event_disk_space.unwrap_or(0),
            hour_events: model.hour_events.unwrap_or(0),
            hour_event_disk_space: model.hour_event_disk_space.unwrap_or(0),
            day_events: model.day_events.unwrap_or(0),
            day_event_disk_space: model.day_event_disk_space.unwrap_or(0),
            week_events: model.week_events.unwrap_or(0),
            week_event_disk_space: model.week_event_disk_space.unwrap_or(0),
            month_events: model.month_events.unwrap_or(0),
            month_event_disk_space: model.month_event_disk_space.unwrap_or(0),
            archived_events: model.archived_events.unwrap_or(0),
            archived_event_disk_space: model.archived_event_disk_space.unwrap_or(0),
        }
    }
}

/// Paginated response for event summaries
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct PaginatedEventSummariesResponse {
    pub items: Vec<EventSummaryResponse>,
    pub total: u64,
    pub per_page: u64,
    pub current_page: u64,
    pub last_page: u64,
}

impl From<PaginatedResponse<EventSummaryResponse>> for PaginatedEventSummariesResponse {
    fn from(r: PaginatedResponse<EventSummaryResponse>) -> Self {
        Self {
            items: r.items,
            total: r.total,
            per_page: r.per_page,
            current_page: r.current_page,
            last_page: r.last_page,
        }
    }
}
