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
/// Returns paginated log entries, optionally filtered by component, level, or server.
/// Logs are ordered by time (newest first).
#[utoipa::path(
    get,
    path = "/api/v3/logs",
    operation_id = "listLogs",
    tag = "Logs",
    params(
        ("page" = Option<u64>, Query, description = "Page number (1-indexed)", example = 1),
        ("page_size" = Option<u64>, Query, description = "Number of items per page (max 1000)", example = 50),
        ("component" = Option<String>, Query, description = "Filter by component (e.g., 'zmc', 'zma', 'zmdc', 'web')", example = "zmc"),
        ("level" = Option<i8>, Query, description = "Filter by minimum log level (-3=Debug, 0=Info, 1=Warning, 2=Error, 3=Fatal)", example = 0),
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
