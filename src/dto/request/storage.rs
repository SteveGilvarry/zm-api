use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateStorageRequest {
    pub name: String,
    pub path: String,
    pub r#type: String,
    pub enabled: i8,
    pub scheme: Option<String>,
    pub server_id: Option<u32>,
    pub url: Option<String>,
    /// Whether deleting an event may remove its media from this storage.
    /// Defaults to `1`, matching the column and ZoneMinder's own UI — a
    /// storage created with `0` cannot be reclaimed by retention.
    pub do_delete: Option<i8>,
}
