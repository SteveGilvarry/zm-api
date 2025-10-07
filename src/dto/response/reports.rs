use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::entity::reports::Model as ReportModel;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ReportResponse {
    pub id: u32,
    pub name: Option<String>,
    pub filter_id: Option<u32>,
    pub start_date_time: Option<String>,
    pub end_date_time: Option<String>,
    pub interval: Option<u32>,
}

impl From<&ReportModel> for ReportResponse {
    fn from(model: &ReportModel) -> Self {
        Self {
            id: model.id,
            name: model.name.clone(),
            filter_id: model.filter_id,
            start_date_time: model.start_date_time.map(|dt| dt.and_utc().to_rfc3339()),
            end_date_time: model.end_date_time.map(|dt| dt.and_utc().to_rfc3339()),
            interval: model.interval,
        }
    }
}
