use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Legacy's "Audit Events Report": per monitor, the events overlapping a
/// window, the gaps between them, and those whose media is missing or empty.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EventAuditResponse {
    #[schema(example = "2026-10-18T09:00:00Z")]
    pub start: String,
    #[schema(example = "2026-10-18T10:00:00Z")]
    pub end: String,
    pub monitors: Vec<MonitorEventAudit>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MonitorEventAudit {
    pub monitor_id: u32,
    pub event_count: u64,
    pub first_event_id: u64,
    pub last_event_id: u64,
    /// Start of the first event in the window.
    pub first_start: Option<String>,
    /// End of the last event in the window.
    pub last_end: Option<String>,
    /// Smallest and largest gap, in seconds, from one event's end to the
    /// next one's start: negative where events overlap. Null with fewer than
    /// two events.
    pub min_gap_seconds: Option<i64>,
    pub max_gap_seconds: Option<i64>,
    /// Events whose video (or, without one, whose directory) isn't on disk.
    pub missing_files: Vec<u64>,
    /// Events whose video file is empty.
    pub zero_size: Vec<u64>,
}
