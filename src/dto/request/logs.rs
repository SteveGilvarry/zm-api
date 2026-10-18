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

/// Column the log list is ordered by.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum LogSortField {
    /// `TimeKey` (default).
    #[default]
    Time,
    Id,
    Component,
    ServerId,
    Pid,
    Level,
    Code,
    Message,
    File,
    Line,
}

/// Parse a comma-separated query value (`"1,2,3"`). An empty list or any
/// element that doesn't parse is an error, never "no filter": on `DELETE /logs`
/// a filter that silently vanished would clear the whole table.
pub fn parse_comma_list<T: std::str::FromStr>(raw: &str) -> Result<Vec<T>, garde::Error> {
    let items: Result<Vec<T>, _> = raw.split(',').map(|s| s.trim().parse::<T>()).collect();
    match items {
        Ok(v) if !v.is_empty() => Ok(v),
        _ => Err(garde::Error::new(
            "must be a comma-separated list of integers",
        )),
    }
}

fn comma_list_of<T: std::str::FromStr>(value: &Option<String>, _: &()) -> garde::Result {
    match value {
        Some(raw) => parse_comma_list::<T>(raw).map(|_| ()),
        None => Ok(()),
    }
}

fn level_list(value: &Option<String>, ctx: &()) -> garde::Result {
    comma_list_of::<i8>(value, ctx)
}

fn id_list(value: &Option<String>, ctx: &()) -> garde::Result {
    comma_list_of::<u32>(value, ctx)
}

/// Sort direction.
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

    /// Exact component name, or a comma list of them (`zmc,zma`).
    #[schema(example = "zmc,zma")]
    #[garde(skip)]
    pub component: Option<String>,

    /// Filter to this severity **or worse** (`fatal` < `error` < `warning` <
    /// `info` < `debug`). E.g. `min_level=error` returns errors and fatals.
    /// Preferred over the raw numeric `level`.
    #[garde(skip)]
    pub min_level: Option<LogLevel>,

    /// Exact `Logs.Level` match, one value or a comma list (`-1,-5`), on
    /// ZoneMinder's inverted numeric scale: 0=Info, -1=Warning, -2=Error,
    /// -3=Fatal, -4=Panic, -5=Audit, positive=debug. For "this severe or
    /// worse", use `min_level` instead.
    #[schema(example = "-2")]
    #[garde(custom(level_list))]
    pub level: Option<String>,

    /// Only these log ids, as a comma list. On `DELETE` this clears the rows
    /// an operator selected.
    #[schema(example = "101,102")]
    #[garde(custom(id_list))]
    pub ids: Option<String>,

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

    /// Sort direction: `asc` or `desc` (default). With the default `sort_by`
    /// that is oldest or newest first.
    #[garde(skip)]
    pub sort: Option<LogSort>,

    /// Column to sort on (default `time`).
    #[garde(skip)]
    pub sort_by: Option<LogSortField>,

    /// Also return `total_unfiltered`, the row count of the whole table. Off
    /// by default: it is a second `COUNT(*)` over `Logs` on every request.
    #[serde(default)]
    #[garde(skip)]
    pub unfiltered_total: bool,

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
