use crate::dto::request::filter_ast::FilterQuery;
use crate::entity::sea_orm_active_enums::EmailFormat;
use garde::Validate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Parse the textual `email_format` accepted by the API into the entity enum.
/// Unknown values fall back to `Summary`.
pub fn parse_email_format(s: &str) -> EmailFormat {
    match s.to_lowercase().as_str() {
        "individual" => EmailFormat::Individual,
        _ => EmailFormat::Summary,
    }
}

/// Create a saved event filter. `name` and `query_json` are required; every
/// other column of the `Filters` table is optional and falls back to the
/// schema default when omitted (flags `0`, `execute_interval` `60`,
/// `email_format` `Individual`).
#[derive(Debug, Default, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateFilterRequest {
    #[garde(length(chars, max = 64))]
    pub name: String,
    /// Raw ZoneMinder `query_json`. Optional when `filter` (the structured AST)
    /// is supplied — in that case the AST is translated and wins.
    #[serde(default)]
    #[garde(skip)]
    pub query_json: String,
    /// Structured, SQLi-safe filter AST. When present it is validated and
    /// translated to ZoneMinder's flat `query_json` for storage.
    #[serde(default)]
    #[garde(skip)]
    pub filter: Option<FilterQuery>,
    #[garde(skip)]
    pub user_id: Option<u32>,
    #[garde(skip)]
    pub execute_interval: Option<u32>,
    /// `Individual` or `Summary` (default `Individual`).
    #[garde(skip)]
    pub email_format: Option<String>,
    #[garde(skip)]
    pub auto_archive: Option<u8>,
    #[garde(skip)]
    pub auto_unarchive: Option<u8>,
    #[garde(skip)]
    pub auto_video: Option<u8>,
    #[garde(skip)]
    pub auto_upload: Option<u8>,
    #[garde(skip)]
    pub auto_email: Option<u8>,
    #[garde(skip)]
    pub email_to: Option<String>,
    #[garde(skip)]
    pub email_subject: Option<String>,
    #[garde(skip)]
    pub email_body: Option<String>,
    #[garde(skip)]
    pub email_server: Option<String>,
    #[garde(skip)]
    pub auto_message: Option<u8>,
    #[garde(skip)]
    pub auto_execute: Option<u8>,
    #[garde(inner(length(max = 255)))]
    pub auto_execute_cmd: Option<String>,
    #[garde(skip)]
    pub auto_delete: Option<u8>,
    #[garde(skip)]
    pub auto_move: Option<u8>,
    #[garde(skip)]
    pub auto_move_to: Option<u16>,
    #[garde(skip)]
    pub auto_copy: Option<u8>,
    #[garde(skip)]
    pub auto_copy_to: Option<u16>,
    #[garde(skip)]
    pub update_disk_space: Option<u8>,
    #[garde(skip)]
    pub background: Option<u8>,
    #[garde(skip)]
    pub concurrent: Option<u8>,
    #[garde(skip)]
    pub lock_rows: Option<u8>,
}

/// Patch a saved event filter. Every field is optional; only the ones provided
/// are changed.
#[derive(Debug, Default, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateFilterRequest {
    #[garde(inner(length(chars, max = 64)))]
    pub name: Option<String>,
    #[garde(skip)]
    pub query_json: Option<String>,
    /// Structured filter AST; when present it is translated to `query_json`.
    #[serde(default)]
    #[garde(skip)]
    pub filter: Option<FilterQuery>,
    #[garde(skip)]
    pub user_id: Option<u32>,
    #[garde(skip)]
    pub execute_interval: Option<u32>,
    /// `Individual` or `Summary`.
    #[garde(skip)]
    pub email_format: Option<String>,
    #[garde(skip)]
    pub auto_archive: Option<u8>,
    #[garde(skip)]
    pub auto_unarchive: Option<u8>,
    #[garde(skip)]
    pub auto_video: Option<u8>,
    #[garde(skip)]
    pub auto_upload: Option<u8>,
    #[garde(skip)]
    pub auto_email: Option<u8>,
    #[garde(skip)]
    pub email_to: Option<String>,
    #[garde(skip)]
    pub email_subject: Option<String>,
    #[garde(skip)]
    pub email_body: Option<String>,
    #[garde(skip)]
    pub email_server: Option<String>,
    #[garde(skip)]
    pub auto_message: Option<u8>,
    #[garde(skip)]
    pub auto_execute: Option<u8>,
    #[garde(inner(length(max = 255)))]
    pub auto_execute_cmd: Option<String>,
    #[garde(skip)]
    pub auto_delete: Option<u8>,
    #[garde(skip)]
    pub auto_move: Option<u8>,
    #[garde(skip)]
    pub auto_move_to: Option<u16>,
    #[garde(skip)]
    pub auto_copy: Option<u8>,
    #[garde(skip)]
    pub auto_copy_to: Option<u16>,
    #[garde(skip)]
    pub update_disk_space: Option<u8>,
    #[garde(skip)]
    pub background: Option<u8>,
    #[garde(skip)]
    pub concurrent: Option<u8>,
    #[garde(skip)]
    pub lock_rows: Option<u8>,
}
