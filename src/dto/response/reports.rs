use crate::dto::PaginatedResponse;
use crate::entity::reports::Model as ReportModel;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
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

/// Paginated response for reports
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct PaginatedReportsResponse {
    pub items: Vec<ReportResponse>,
    pub total: u64,
    pub per_page: u64,
    pub current_page: u64,
    pub last_page: u64,
}

impl From<PaginatedResponse<ReportResponse>> for PaginatedReportsResponse {
    fn from(r: PaginatedResponse<ReportResponse>) -> Self {
        Self {
            items: r.items,
            total: r.total,
            per_page: r.per_page,
            current_page: r.current_page,
            last_page: r.last_page,
        }
    }
}
