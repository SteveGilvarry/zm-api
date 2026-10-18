use crate::dto::PaginatedResponse;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ConfigResponse {
    pub id: u16,
    pub name: String,
    pub value: String,
    pub r#type: String,
    pub default_value: Option<String>,
    pub hint: Option<String>,
    pub pattern: Option<String>,
    pub format: Option<String>,
    pub prompt: Option<String>,
    pub help: Option<String>,
    pub category: String,
    pub readonly: u8,
    pub private: i8,
    pub system: i8,
    /// `Config.Requires`, raw: clauses like `ZM_OPT_USE_AUTH=1;ZM_AUTH_RELAY=hashed`
    /// that must all hold for the option to apply (#183).
    pub requires: Option<String>,
    /// `db`, or `file` when zm.conf or a conf.d file sets the option. A file
    /// wins everywhere, so `value` is then the file's (blank for a private
    /// one) and the option can't be changed through the API.
    pub source: String,
    /// The file that sets it, for `source: "file"`.
    pub source_file: Option<String>,
}

/// Paginated response for configs
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct PaginatedConfigsResponse {
    pub items: Vec<ConfigResponse>,
    pub total: u64,
    pub per_page: u64,
    pub current_page: u64,
    pub last_page: u64,
}

impl From<PaginatedResponse<ConfigResponse>> for PaginatedConfigsResponse {
    fn from(r: PaginatedResponse<ConfigResponse>) -> Self {
        Self {
            items: r.items,
            total: r.total,
            per_page: r.per_page,
            current_page: r.current_page,
            last_page: r.last_page,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CategoryCountResponse {
    pub category: String,
    pub count: u64,
}
