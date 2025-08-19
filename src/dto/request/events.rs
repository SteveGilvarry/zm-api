use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use chrono::{DateTime, Utc, NaiveDateTime};
use garde::Validate;

use crate::dto::wrappers::{DateTimeWrapper, DecimalWrapper, SchemeWrapper, NaiveDateTimeWrapper};
use crate::entity::sea_orm_active_enums::{Orientation, Scheme};

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
    pub end_time: Option<DateTimeWrapper>
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
}