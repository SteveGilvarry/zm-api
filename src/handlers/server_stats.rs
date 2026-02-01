use crate::dto::request::server_stats::CreateServerStatRequest;
use crate::dto::response::server_stats::PaginatedServerStatsResponse;
use crate::dto::response::ServerStatResponse;
use crate::dto::PaginationParams;
use crate::error::AppResult;
use crate::server::state::AppState;
use axum::{
    extract::{Path, Query, State},
    Json,
};

/// List all server stats with pagination.
#[utoipa::path(
    get,
    path = "/api/v3/server-stats",
    params(
        ("page" = Option<u64>, Query, description = "Page number (1-indexed)", example = 1),
        ("page_size" = Option<u64>, Query, description = "Items per page (max 1000)", example = 25)
    ),
    responses((status = 200, description = "Paginated list of server stats", body = PaginatedServerStatsResponse)),
    tag = "Server Stats",
    security(("jwt" = []))
)]
pub async fn list_server_stats(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> AppResult<Json<PaginatedServerStatsResponse>> {
    let result = crate::service::server_stats::list_paginated(&state, &params).await?;
    Ok(Json(PaginatedServerStatsResponse::from(result)))
}

/// Get a server stat by id.
#[utoipa::path(
    get,
    path = "/api/v3/server-stats/{id}",
    params(("id" = u32, Path, description = "Server Stat ID")),
    responses((status = 200, description = "Server stat detail", body = ServerStatResponse)),
    tag = "Server Stats",
    security(("jwt" = []))
)]
pub async fn get_server_stat(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<Json<ServerStatResponse>> {
    let item = crate::service::server_stats::get_by_id(&state, id).await?;
    Ok(Json(item))
}

/// Create a new server stat entry.
#[utoipa::path(
    post,
    path = "/api/v3/server-stats",
    request_body = CreateServerStatRequest,
    responses((status = 201, description = "Created server stat", body = ServerStatResponse)),
    tag = "Server Stats",
    security(("jwt" = []))
)]
pub async fn create_server_stat(
    State(state): State<AppState>,
    Json(req): Json<CreateServerStatRequest>,
) -> AppResult<(axum::http::StatusCode, Json<ServerStatResponse>)> {
    let item = crate::service::server_stats::create(&state, req).await?;
    Ok((axum::http::StatusCode::CREATED, Json(item)))
}

/// Delete a server stat by id.
#[utoipa::path(
    delete,
    path = "/api/v3/server-stats/{id}",
    params(("id" = u32, Path, description = "Server Stat ID")),
    responses((status = 204, description = "Deleted server stat")),
    tag = "Server Stats",
    security(("jwt" = []))
)]
pub async fn delete_server_stat(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<axum::http::StatusCode> {
    crate::service::server_stats::delete(&state, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
