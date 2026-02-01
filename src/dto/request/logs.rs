use garde::Validate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Query parameters for listing logs
#[derive(Debug, Deserialize, Serialize, ToSchema, Validate)]
pub struct LogQueryParams {
    /// Page number (1-indexed)
    #[schema(example = 1)]
    #[garde(range(min = 1))]
    pub page: Option<u64>,

    /// Number of items per page (max 1000)
    #[schema(example = 50)]
    #[garde(range(min = 1, max = 1000))]
    pub page_size: Option<u64>,

    /// Filter by component name (e.g., "zmc", "zma", "zmdc", "web")
    #[schema(example = "zmc")]
    #[garde(skip)]
    pub component: Option<String>,

    /// Filter by minimum log level (-3=Debug, 0=Info, 1=Warning, 2=Error, 3=Fatal)
    #[schema(example = 0)]
    #[garde(range(min = -3, max = 3))]
    pub level: Option<i8>,

    /// Filter by server ID
    #[schema(example = 1)]
    #[garde(skip)]
    pub server_id: Option<u32>,
}
