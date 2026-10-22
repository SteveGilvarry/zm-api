use crate::dto::request::logs::LogQueryParams;
use crate::dto::response::logs::{LogResponse, PaginatedLogsResponse};
use crate::error::{AppError, AppResponseError, AppResult};
use crate::server::state::AppState;
use axum::{
    extract::{Path, Query, State},
    Json,
};
use garde::Validate;
use tracing::{info, instrument};

/// List log entries with pagination and filtering.
///
/// Filter by `component`, `min_level` (severity name — this severe or worse),
/// exact `level` (one value or a comma list), `ids`, `search` (message
/// substring), `start`/`end` (Unix seconds) and `server_id`. Order with
/// `sort_by` (default `time`) and `sort` (asc/desc; default newest-first).
#[utoipa::path(
    get,
    path = "/api/v3/logs",
    operation_id = "listLogs",
    tag = "Logs",
    params(
        ("page" = Option<u64>, Query, description = "Page number (1-indexed)", example = 1),
        ("page_size" = Option<u64>, Query, description = "Number of items per page (max 1000)", example = 50),
        ("component" = Option<String>, Query, description = "Component name, or a comma list of them (e.g. 'zmc,zma')", example = "zmc,zma"),
        ("min_level" = Option<String>, Query, description = "Severity threshold: fatal|error|warning|info|debug, returns that severity or worse", example = "error"),
        ("level" = Option<String>, Query, description = "Exact Logs.Level match, one value or a comma list (inverted scale: 0=Info, -1=Warning, -2=Error, -3=Fatal, -4=Panic, -5=Audit, positive=debug)", example = "-1,-5"),
        ("ids" = Option<String>, Query, description = "Only these log ids, comma-separated", example = "101,102"),
        ("search" = Option<String>, Query, description = "Case-insensitive substring match on the message", example = "connection"),
        ("start" = Option<f64>, Query, description = "Only logs at or after this Unix time (seconds)"),
        ("end" = Option<f64>, Query, description = "Only logs at or before this Unix time (seconds)"),
        ("sort" = Option<String>, Query, description = "asc or desc (default)", example = "desc"),
        ("sort_by" = Option<String>, Query, description = "Column to sort on: time (default)|id|component|server_id|pid|level|code|message|file|line", example = "time"),
        ("unfiltered_total" = Option<bool>, Query, description = "Also return total_unfiltered, the whole table's row count. Costs a second COUNT(*)."),
        ("server_id" = Option<u32>, Query, description = "Filter by server ID", example = 1)
    ),
    responses(
        (status = 200, description = "Paginated list of logs", body = PaginatedLogsResponse),
        (status = 400, description = "Bad request", body = AppResponseError),
        (status = 401, description = "Unauthorized", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(("jwt" = []))
)]
#[instrument(level = "debug", skip(state))]
pub async fn list_logs(
    State(state): State<AppState>,
    Query(params): Query<LogQueryParams>,
) -> AppResult<Json<PaginatedLogsResponse>> {
    tracing::debug!("Listing logs with params: {:?}", params);
    // Without this the declared bounds are inert: page_size=0 reaches
    // `div_ceil` and SeaORM's `paginate`, both of which panic on zero.
    params.validate().map_err(AppError::InvalidInputError)?;

    let result = crate::service::logs::list(&state, &params).await?;
    Ok(Json(result))
}

/// Clear logs (legacy "Clear Logs"), optionally scoped by the same filters as
/// the list endpoint. Returns the number of rows deleted. Requires System
/// (admin) permission.
#[utoipa::path(
    delete,
    path = "/api/v3/logs",
    operation_id = "clearLogs",
    tag = "Logs",
    params(
        ("component" = Option<String>, Query, description = "Only clear logs from this component, or a comma list of them"),
        ("min_level" = Option<String>, Query, description = "Only clear this severity or worse (fatal|error|warning|info|debug)"),
        ("level" = Option<String>, Query, description = "Only clear these exact Logs.Level values (one or a comma list)"),
        ("ids" = Option<String>, Query, description = "Only clear these log ids, comma-separated. An empty or unparseable list is a 400, never \"clear everything\"."),
        ("search" = Option<String>, Query, description = "Only clear messages containing this substring"),
        ("start" = Option<f64>, Query, description = "Only clear logs at or after this Unix time (seconds)"),
        ("end" = Option<f64>, Query, description = "Only clear logs at or before this Unix time (seconds)"),
        ("server_id" = Option<u32>, Query, description = "Only clear logs for this server")
    ),
    responses(
        (status = 200, description = "Number of log rows deleted", body = crate::dto::response::MessageResponse),
        (status = 401, description = "Unauthorized", body = AppResponseError),
        (status = 403, description = "Forbidden (System permission required)", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(("jwt" = []))
)]
#[instrument(level = "debug", skip(state))]
pub async fn clear_logs(
    State(state): State<AppState>,
    Query(params): Query<LogQueryParams>,
) -> AppResult<Json<crate::dto::response::MessageResponse>> {
    params.validate().map_err(AppError::InvalidInputError)?;
    let deleted = crate::service::logs::delete(&state, &params).await?;
    info!("Cleared {deleted} log rows");
    Ok(Json(crate::dto::response::MessageResponse::new(&format!(
        "Deleted {deleted} log entries"
    ))))
}

/// List every component that has written a log row, sorted.
///
/// Feeds the log page's Component filter, so it shows what is in the table
/// instead of a guessed list.
#[utoipa::path(
    get,
    path = "/api/v3/logs/components",
    operation_id = "listLogComponents",
    tag = "Logs",
    responses(
        (status = 200, description = "Distinct component names, sorted", body = Vec<String>),
        (status = 401, description = "Unauthorized", body = AppResponseError)
    ),
    security(("jwt" = []))
)]
pub async fn list_log_components(State(state): State<AppState>) -> AppResult<Json<Vec<String>>> {
    Ok(Json(crate::service::logs::components(&state).await?))
}

/// Get a single log entry by id.
///
/// - Requires a valid JWT.
#[utoipa::path(
    get,
    path = "/api/v3/logs/{id}",
    params(("id" = u32, Path, description = "Log ID")),
    responses((status = 200, description = "Log detail", body = serde_json::Value)),
    tag = "Logs",
    security(("jwt" = []))
)]
pub async fn get_log(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<Json<LogResponse>> {
    let item = crate::service::logs::get_by_id(&state, id).await?;
    Ok(Json(item))
}

// No POST for logs; logs are system-generated.
