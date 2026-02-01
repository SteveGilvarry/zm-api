use garde::Validate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::dto::wrappers::{DateTimeWrapper, DecimalWrapper};
use crate::entity::sea_orm_active_enums::Orientation;

/// Field to sort events by
#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EventSortField {
    /// Sort by event start time (default)
    #[default]
    StartTime,
    /// Sort by event end time
    EndTime,
    /// Sort by number of alarm frames
    AlarmFrames,
    /// Sort by maximum score
    MaxScore,
    /// Sort by average score
    AvgScore,
    /// Sort by total score
    TotScore,
    /// Sort by event duration/length
    Length,
    /// Sort by event ID
    Id,
}

/// Sort direction
#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SortDirection {
    /// Ascending order (oldest first, lowest score first)
    Asc,
    /// Descending order (newest first, highest score first) - default
    #[default]
    Desc,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, Validate)]
pub struct EventQueryParams {
    #[schema(example = "1")]
    #[garde(range(min = 1))]
    pub page: Option<u64>,

    #[schema(example = "20")]
    #[garde(range(min = 1, max = 1000))]
    pub page_size: Option<u64>,

    #[schema(example = "1")]
    #[garde(range(min = 1, max = 1000000))]
    pub monitor_id: Option<u32>,

    #[schema(example = "2025-04-28T00:00:00Z")]
    #[garde(skip)]
    pub start_time: Option<DateTimeWrapper>,

    #[schema(example = "2025-04-29T23:59:59Z")]
    #[garde(skip)]
    pub end_time: Option<DateTimeWrapper>,

    /// Field to sort results by (default: start_time)
    #[schema(example = "start_time")]
    #[garde(skip)]
    pub sort: Option<EventSortField>,

    /// Sort direction (default: desc)
    #[schema(example = "desc")]
    #[garde(skip)]
    pub direction: Option<SortDirection>,

    /// Filter events with at least this many alarm frames
    #[schema(example = "5")]
    #[garde(range(min = 0))]
    pub alarm_frames_min: Option<u32>,

    /// Filter to show only archived events
    #[schema(example = "true")]
    #[garde(skip)]
    pub archived: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, Validate)]
pub struct EventCreateRequest {
    #[schema(example = 1)]
    #[garde(skip)]
    pub monitor_id: u32,

    #[schema(example = 1)]
    #[garde(skip)]
    pub storage_id: u16,

    #[schema(example = "null", nullable = true)]
    #[garde(skip)]
    pub secondary_storage_id: Option<u16>,

    #[schema(example = "Motion Detection")]
    #[garde(length(min = 1, max = 255))]
    pub name: String,

    #[schema(example = "Motion in zone 1", nullable = true)]
    #[garde(skip)]
    pub cause: Option<String>,

    #[schema(example = "2025-04-29T10:00:00Z", nullable = true)]
    #[garde(skip)]
    pub start_date_time: Option<DateTimeWrapper>,

    #[schema(example = "2025-04-29T10:02:00Z", nullable = true)]
    #[garde(skip)]
    pub end_date_time: Option<DateTimeWrapper>,

    #[schema(example = 1920)]
    #[garde(range(min = 320, max = 7680))]
    pub width: u16,

    #[schema(example = 1080)]
    #[garde(range(min = 240, max = 4320))]
    pub height: u16,

    #[schema(example = "2.00")]
    #[garde(skip)]
    pub length: DecimalWrapper,

    #[schema(example = "Important security event", nullable = true)]
    #[garde(length(min = 0, max = 1000))]
    pub notes: Option<String>,

    #[schema(example = "ROTATE_0")]
    #[garde(skip)]
    pub orientation: Orientation,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, Validate)]
pub struct EventUpdateRequest {
    #[schema(example = "Motion Detection")]
    #[garde(length(min = 1, max = 255))]
    pub name: Option<String>,

    #[schema(example = "Motion in zone 1", nullable = true)]
    #[garde(skip)]
    pub cause: Option<String>,

    #[schema(example = "Important security event", nullable = true)]
    #[garde(length(min = 0, max = 1000))]
    pub notes: Option<String>,

    #[schema(example = "ROTATE_0")]
    #[garde(skip)]
    pub orientation: Option<Orientation>,

    /// Mark event as archived (protected from auto-deletion)
    #[schema(example = true)]
    #[garde(skip)]
    pub archived: Option<bool>,

    /// Lock event to prevent deletion by filters
    #[schema(example = false)]
    #[garde(skip)]
    pub locked: Option<bool>,

    /// Mark event as having been emailed
    #[schema(example = false)]
    #[garde(skip)]
    pub emailed: Option<bool>,

    /// Mark event as having been sent via push notification
    #[schema(example = false)]
    #[garde(skip)]
    pub messaged: Option<bool>,

    /// Mark event as having been uploaded to external storage
    #[schema(example = false)]
    #[garde(skip)]
    pub uploaded: Option<bool>,

    /// Mark event as having triggered an external action
    #[schema(example = false)]
    #[garde(skip)]
    pub executed: Option<bool>,
}
