use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateEventTagRequest {
    #[schema(example = 1)]
    pub event_id: u64,
    #[schema(example = 1)]
    pub tag_id: u64,
    #[schema(example = 1)]
    pub assigned_by: Option<u32>,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct EventTagQuery {
    pub event_id: Option<u64>,
    pub tag_id: Option<u64>,
}
