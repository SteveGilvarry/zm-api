use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use sea_orm::prelude::DateTimeUtc;

use crate::dto::wrappers::DateTimeWrapper;
use crate::entity::events::Model as EventModel;
use crate::entity::events_tags::Model as EventTagModel;
use crate::entity::tags::Model as TagModel;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct EventTagResponse {
    #[schema(example = 1)]
    pub tag_id: u64,
    #[schema(example = 1)]
    pub event_id: u64,
    #[schema(value_type = Option<String>, example = "2025-01-15T10:30:00Z")]
    pub assigned_date: Option<DateTimeUtc>,
    #[schema(example = 1)]
    pub assigned_by: Option<u32>,
}

impl From<&EventTagModel> for EventTagResponse {
    fn from(model: &EventTagModel) -> Self {
        Self {
            tag_id: model.tag_id,
            event_id: model.event_id,
            assigned_date: model.assigned_date,
            assigned_by: model.assigned_by,
        }
    }
}

/// Lightweight tag summary for embedding in EventResponse
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TagSummary {
    #[schema(example = 1)]
    pub id: u64,
    #[schema(example = "Important")]
    pub name: String,
}

impl From<&TagModel> for TagSummary {
    fn from(model: &TagModel) -> Self {
        Self {
            id: model.id,
            name: model.name.clone(),
        }
    }
}

/// Lightweight event summary for embedding in TagDetailResponse
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EventSummary {
    #[schema(example = 1)]
    pub id: u64,
    #[schema(example = "Motion Detection")]
    pub name: String,
    #[schema(example = 1)]
    pub monitor_id: u32,
    #[schema(example = "2025-04-29T10:00:00Z")]
    pub start_date_time: Option<DateTimeWrapper>,
}

impl From<&EventModel> for EventSummary {
    fn from(model: &EventModel) -> Self {
        let to_utc = |ndt: NaiveDateTime| -> DateTime<Utc> {
            DateTime::<Utc>::from_naive_utc_and_offset(ndt, Utc)
        };

        Self {
            id: model.id,
            name: model.name.clone(),
            monitor_id: model.monitor_id,
            start_date_time: model.start_date_time.map(|ndt| DateTimeWrapper(to_utc(ndt))),
        }
    }
}

/// Tag detail response with paginated events
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TagDetailResponse {
    #[schema(example = 1)]
    pub id: u64,
    #[schema(example = "Important")]
    pub name: String,
    #[schema(value_type = Option<String>, example = "2025-01-15T10:30:00Z")]
    pub create_date: Option<DateTimeUtc>,
    pub events: Vec<EventSummary>,
    #[schema(example = 100)]
    pub total_events: u64,
    #[schema(example = 20)]
    pub per_page: u64,
    #[schema(example = 1)]
    pub current_page: u64,
    #[schema(example = 5)]
    pub last_page: u64,
}
