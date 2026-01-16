use crate::dto::request::PaginationParams;
use crate::dto::response::{LogResponse, PaginatedResponse};
use crate::error::AppResult;
use crate::server::state::AppState;
use axum::{
    extract::{Path, Query, State},
    Json,
};

/// List log entries with pagination, ordered by newest first.
///
/// - Requires a valid JWT.
#[utoipa::path(
    get,
    path = "/api/v3/logs",
    params(
        ("page" = Option<u64>, Query, description = "Page number (1-indexed)", example = 1),
        ("page_size" = Option<u64>, Query, description = "Number of items per page (max 1000)", example = 20)
    ),
    responses((status = 200, description = "Paginated list of logs", body = PaginatedResponse<LogResponse>)),
    tag = "Logs",
    security(("jwt" = []))
)]
pub async fn list_logs(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> AppResult<Json<PaginatedResponse<LogResponse>>> {
    let items = crate::service::logs::list_all_paginated(&state, &params).await?;
    Ok(Json(items))
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
