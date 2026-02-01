use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LogResponse {
    /// Log entry ID
    #[schema(example = 12345)]
    pub id: u32,

    /// Timestamp as Unix epoch with microseconds (e.g., 1234567890.123456)
    #[schema(example = "1704067200.123456")]
    pub time_key: String,

    /// Component that generated the log (e.g., "zmc", "zma", "zmdc", "web")
    #[schema(example = "zmc")]
    pub component: String,

    /// Server ID (for multi-server setups)
    #[schema(example = 1)]
    pub server_id: Option<u32>,

    /// Process ID that generated the log
    #[schema(example = 12345)]
    pub pid: Option<i32>,

    /// Log level: -3=Debug, -2=Debug, -1=Debug, 0=Info, 1=Warning, 2=Error, 3=Fatal
    #[schema(example = 0)]
    pub level: i8,

    /// Log code (e.g., "WAR", "ERR", "INF")
    #[schema(example = "INF")]
    pub code: String,

    /// Log message
    #[schema(example = "Starting capture daemon")]
    pub message: String,

    /// Source file that generated the log
    #[schema(example = "zm_monitor.cpp")]
    pub file: Option<String>,

    /// Line number in the source file
    #[schema(example = 123)]
    pub line: Option<u16>,
}

impl From<&crate::entity::logs::Model> for LogResponse {
    fn from(m: &crate::entity::logs::Model) -> Self {
        Self {
            id: m.id,
            time_key: format_time_key(&m.time_key),
            component: m.component.clone(),
            server_id: m.server_id,
            pid: m.pid,
            level: m.level,
            code: m.code.clone(),
            message: m.message.clone(),
            file: m.file.clone(),
            line: m.line,
        }
    }
}

/// Format time_key decimal as a string with microsecond precision
fn format_time_key(time_key: &Decimal) -> String {
    format!("{:.6}", time_key)
}

/// Paginated response for log entries
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PaginatedLogsResponse {
    /// List of log entries
    pub logs: Vec<LogResponse>,
    /// Total number of matching logs
    pub total: u64,
    /// Number of logs per page
    pub per_page: u64,
    /// Current page number (1-indexed)
    pub current_page: u64,
    /// Last page number
    pub last_page: u64,
}
