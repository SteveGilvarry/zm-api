use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MessageResponse {
    message: String,
}

impl MessageResponse {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ServiceStatusResponse {
    status: String,
}

impl ServiceStatusResponse {
    pub fn new(status: &str) -> Self {
        Self {
            status: status.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct VersionResponse {
    /// ZoneMinder version
    pub version: String,
    /// API version
    pub api_version: String,
    /// Database version
    pub db_version: String,
}

/// Server locale: the effective timezone + ZoneMinder's date/time format
/// patterns, so clients can render server-local time consistently with the
/// legacy web UI (GH #33). Pairs with event timestamps now being true UTC.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LocaleResponse {
    /// Effective IANA timezone name (from ZoneMinder's `ZM_TIMEZONE`, or the
    /// host OS zone when that is empty). May be `null` if it can't be resolved.
    #[schema(example = "Australia/Melbourne", nullable = true)]
    pub timezone: Option<String>,
    /// Current UTC offset of the server, e.g. `+10:00`.
    #[schema(example = "+10:00")]
    pub utc_offset: String,
    /// Current UTC offset in seconds (e.g. 36000 for +10:00).
    #[schema(example = 36000)]
    pub utc_offset_seconds: i32,
    /// ZoneMinder `ZM_DATE_FORMAT_PATTERN`.
    pub date_format: Option<String>,
    /// ZoneMinder `ZM_DATETIME_FORMAT_PATTERN`.
    pub datetime_format: Option<String>,
    /// ZoneMinder `ZM_TIME_FORMAT_PATTERN`.
    pub time_format: Option<String>,
}
