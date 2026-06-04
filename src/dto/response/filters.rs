use crate::dto::request::filter_ast::FilterQuery;
use crate::dto::PaginatedResponse;
use crate::entity::sea_orm_active_enums::EmailFormat;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// String value ZoneMinder stores for an [`EmailFormat`] (`Individual` / `Summary`).
pub(crate) fn email_format_str(f: &EmailFormat) -> &'static str {
    match f {
        EmailFormat::Individual => "Individual",
        EmailFormat::Summary => "Summary",
    }
}

/// A saved event filter. Mirrors every column of the `Filters` table so the API
/// (and its OpenAPI schema) faithfully represents a ZoneMinder filter rather
/// than a lossy subset.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FilterResponse {
    pub id: u32,
    pub name: String,
    pub user_id: Option<u32>,
    pub execute_interval: u32,
    pub query_json: String,
    pub auto_archive: u8,
    pub auto_unarchive: u8,
    pub auto_video: u8,
    pub auto_upload: u8,
    pub auto_email: u8,
    pub email_to: Option<String>,
    pub email_subject: Option<String>,
    pub email_body: Option<String>,
    pub email_server: Option<String>,
    /// `Individual` or `Summary`.
    pub email_format: String,
    pub auto_message: u8,
    pub auto_execute: u8,
    pub auto_execute_cmd: Option<String>,
    pub auto_delete: u8,
    pub auto_move: u8,
    pub auto_move_to: u16,
    pub auto_copy: u8,
    pub auto_copy_to: u16,
    pub update_disk_space: u8,
    pub background: u8,
    pub concurrent: u8,
    pub lock_rows: u8,
    /// The stored `query_json` parsed into the structured AST, when it maps to
    /// our vocabulary. `None` for legacy/unmodelled filters. Populated by the
    /// service layer (the `From<&Model>` conversion leaves it `None`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<FilterQuery>,
}

impl From<&crate::entity::filters::Model> for FilterResponse {
    fn from(m: &crate::entity::filters::Model) -> Self {
        Self {
            id: m.id,
            name: m.name.clone(),
            user_id: m.user_id,
            execute_interval: m.execute_interval,
            query_json: m.query_json.clone(),
            auto_archive: m.auto_archive,
            auto_unarchive: m.auto_unarchive,
            auto_video: m.auto_video,
            auto_upload: m.auto_upload,
            auto_email: m.auto_email,
            email_to: m.email_to.clone(),
            email_subject: m.email_subject.clone(),
            email_body: m.email_body.clone(),
            email_server: m.email_server.clone(),
            email_format: email_format_str(&m.email_format).to_string(),
            auto_message: m.auto_message,
            auto_execute: m.auto_execute,
            auto_execute_cmd: m.auto_execute_cmd.clone(),
            auto_delete: m.auto_delete,
            auto_move: m.auto_move,
            auto_move_to: m.auto_move_to,
            auto_copy: m.auto_copy,
            auto_copy_to: m.auto_copy_to,
            update_disk_space: m.update_disk_space,
            background: m.background,
            concurrent: m.concurrent,
            lock_rows: m.lock_rows,
            filter: None,
        }
    }
}

/// Paginated response for filters
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct PaginatedFiltersResponse {
    pub items: Vec<FilterResponse>,
    pub total: u64,
    pub per_page: u64,
    pub current_page: u64,
    pub last_page: u64,
}

impl From<PaginatedResponse<FilterResponse>> for PaginatedFiltersResponse {
    fn from(r: PaginatedResponse<FilterResponse>) -> Self {
        Self {
            items: r.items,
            total: r.total,
            per_page: r.per_page,
            current_page: r.current_page,
            last_page: r.last_page,
        }
    }
}
