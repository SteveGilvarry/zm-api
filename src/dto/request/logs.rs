use garde::Validate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// A ZoneMinder log severity, named instead of numeric.
///
/// ZoneMinder's `Logs.Level` scale is inverted — more negative is more severe
/// (`0`=Info, `-1`=Warning, `-2`=Error, `-3`=Fatal; positive values are debug
/// levels). Clients should filter by name via `min_level` rather than reasoning
/// about the numeric direction.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Fatal,
    Error,
    Warning,
    Info,
    Debug,
}

impl LogLevel {
    /// The `Logs.Level` value for this severity.
    pub fn threshold(self) -> i8 {
        match self {
            LogLevel::Fatal => -3,
            LogLevel::Error => -2,
            LogLevel::Warning => -1,
            LogLevel::Info => 0,
            LogLevel::Debug => 1,
        }
    }
}

/// Sort direction for the log time (`TimeKey`).
#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum LogSort {
    /// Oldest first.
    Asc,
    /// Newest first (default).
    #[default]
    Desc,
}

/// Query parameters for listing (and clearing) logs.
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

    /// Filter to this severity **or worse** (`fatal` < `error` < `warning` <
    /// `info` < `debug`). E.g. `min_level=error` returns errors and fatals.
    /// Preferred over the raw numeric `level`.
    #[garde(skip)]
    pub min_level: Option<LogLevel>,

    /// Exact `Logs.Level` value match (ZoneMinder's inverted numeric scale:
    /// 0=Info, -1=Warning, -2=Error, -3=Fatal, positive=debug). For "this
    /// severe or worse", use `min_level` instead.
    #[schema(example = -2)]
    #[garde(range(min = -128, max = 127))]
    pub level: Option<i8>,

    /// Case-insensitive substring match on the log message.
    #[schema(example = "connection")]
    #[garde(skip)]
    pub search: Option<String>,

    /// Only logs at or after this Unix time (seconds; `TimeKey`).
    #[schema(example = 1_714_300_000.0)]
    #[garde(skip)]
    pub start: Option<f64>,

    /// Only logs at or before this Unix time (seconds; `TimeKey`).
    #[schema(example = 1_714_400_000.0)]
    #[garde(skip)]
    pub end: Option<f64>,

    /// Sort by time: `asc` (oldest first) or `desc` (newest first, default).
    #[garde(skip)]
    pub sort: Option<LogSort>,

    /// Filter by server ID
    #[schema(example = 1)]
    #[garde(skip)]
    pub server_id: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::LogLevel;

    #[test]
    fn severity_thresholds_follow_zoneminders_inverted_scale() {
        // More severe = more negative.
        assert_eq!(LogLevel::Fatal.threshold(), -3);
        assert_eq!(LogLevel::Error.threshold(), -2);
        assert_eq!(LogLevel::Warning.threshold(), -1);
        assert_eq!(LogLevel::Info.threshold(), 0);
        assert!(LogLevel::Error.threshold() < LogLevel::Warning.threshold());
    }

    #[test]
    fn deserializes_from_lowercase_name() {
        let l: LogLevel = serde_json::from_value(serde_json::json!("error")).unwrap();
        assert_eq!(l, LogLevel::Error);
    }
}
