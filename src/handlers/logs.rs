use crate::dto::request::logs::LogQueryParams;
use crate::dto::response::logs::{LogResponse, PaginatedLogsResponse};
use crate::error::{AppResponseError, AppResult};
use crate::server::state::AppState;
use axum::{
    extract::{Path, Query, State},
    Json,
};
use tracing::{info, instrument};

/// List log entries with pagination and filtering.
///
/// Filter by `component`, `min_level` (severity name — this severe or worse),
/// exact `level`, `search` (message substring), `start`/`end` (Unix seconds),
/// `server_id`, and `sort` (asc/desc by time; default newest-first).
#[utoipa::path(
    get,
    path = "/api/v3/logs",
    operation_id = "listLogs",
    tag = "Logs",
    params(
        ("page" = Option<u64>, Query, description = "Page number (1-indexed)", example = 1),
        ("page_size" = Option<u64>, Query, description = "Number of items per page (max 1000)", example = 50),
        ("component" = Option<String>, Query, description = "Filter by component (e.g., 'zmc', 'zma', 'zmdc', 'web')", example = "zmc"),
        ("min_level" = Option<String>, Query, description = "Severity threshold: fatal|error|warning|info|debug, returns that severity or worse", example = "error"),
        ("level" = Option<i8>, Query, description = "Exact Logs.Level match (inverted scale: 0=Info, -1=Warning, -2=Error, -3=Fatal, positive=debug)", example = -2),
        ("search" = Option<String>, Query, description = "Case-insensitive substring match on the message", example = "connection"),
        ("start" = Option<f64>, Query, description = "Only logs at or after this Unix time (seconds)"),
        ("end" = Option<f64>, Query, description = "Only logs at or before this Unix time (seconds)"),
        ("sort" = Option<String>, Query, description = "asc (oldest first) or desc (newest first, default)", example = "desc"),
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
#[instrument(skip(state))]
pub async fn list_logs(
    State(state): State<AppState>,
    Query(params): Query<LogQueryParams>,
) -> AppResult<Json<PaginatedLogsResponse>> {
    info!("Listing logs with params: {:?}", params);

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
        ("component" = Option<String>, Query, description = "Only clear logs from this component"),
        ("min_level" = Option<String>, Query, description = "Only clear this severity or worse (fatal|error|warning|info|debug)"),
        ("level" = Option<i8>, Query, description = "Only clear this exact Logs.Level"),
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
#[instrument(skip(state))]
pub async fn clear_logs(
    State(state): State<AppState>,
    Query(params): Query<LogQueryParams>,
) -> AppResult<Json<crate::dto::response::MessageResponse>> {
    let deleted = crate::service::logs::delete(&state, &params).await?;
    info!("Cleared {deleted} log rows");
    Ok(Json(crate::dto::response::MessageResponse::new(&format!(
        "Deleted {deleted} log entries"
    ))))
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
