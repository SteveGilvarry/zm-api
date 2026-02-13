use garde::Validate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::dto::pagination::{DEFAULT_PAGE_SIZE, MAX_PAGE_SIZE};
use crate::dto::PaginationParams;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateConfigRequest {
    pub value: String,
}

#[derive(Debug, Default, Deserialize, Serialize, ToSchema, Validate)]
pub struct ConfigQueryParams {
    #[schema(example = 1, minimum = 1)]
    #[garde(range(min = 1))]
    pub page: Option<u64>,

    #[schema(example = 25, minimum = 1, maximum = 1000)]
    #[garde(range(min = 1, max = 1000))]
    pub page_size: Option<u64>,

    /// Exact match on Category column
    #[garde(skip)]
    pub category: Option<String>,

    /// Substring match on Name column
    #[garde(skip)]
    pub search: Option<String>,
}

impl ConfigQueryParams {
    pub fn page(&self) -> u64 {
        self.page.unwrap_or(1).max(1)
    }

    pub fn page_size(&self) -> u64 {
        self.page_size
            .unwrap_or(DEFAULT_PAGE_SIZE)
            .clamp(1, MAX_PAGE_SIZE)
    }

    pub fn as_pagination(&self) -> PaginationParams {
        PaginationParams {
            page: Some(self.page()),
            page_size: Some(self.page_size()),
        }
    }
}
