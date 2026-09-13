use garde::Validate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateStorageRequest {
    #[garde(length(chars, max = 64))]
    pub name: String,
    #[garde(length(chars, max = 64))]
    pub path: String,
    #[garde(skip)]
    pub r#type: String,
    #[garde(skip)]
    pub enabled: i8,
    #[garde(skip)]
    pub scheme: Option<String>,
    #[garde(skip)]
    pub server_id: Option<u32>,
    #[garde(inner(length(chars, max = 255)))]
    pub url: Option<String>,
    /// Whether deleting an event may remove its media from this storage.
    /// Defaults to `1`, matching the column and ZoneMinder's own UI — a
    /// storage created with `0` cannot be reclaimed by retention.
    #[garde(skip)]
    pub do_delete: Option<i8>,
}
