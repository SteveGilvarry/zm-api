use crate::dto::request::stats::{CreateStatRequest, UpdateStatRequest};
use crate::dto::response::stats::PaginatedStatsResponse;
use crate::dto::response::StatResponse;
use crate::dto::PaginationParams;
use crate::error::AppResult;
use crate::server::state::AppState;
use axum::{
    extract::{Path, Query, State},
    Json,
};

/// List all stats with pagination.
///
/// - Requires a valid JWT.
#[utoipa::path(
    get,
    path = "/api/v3/stats",
    params(
        ("page" = Option<u64>, Query, description = "Page number (1-indexed)", example = 1),
        ("page_size" = Option<u64>, Query, description = "Items per page (max 1000)", example = 25)
    ),
    responses((status = 200, description = "Paginated list of stats", body = PaginatedStatsResponse)),
    tag = "Stats",
    security(("jwt" = []))
)]
pub async fn list_stats(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> AppResult<Json<PaginatedStatsResponse>> {
    let result = crate::service::stats::list_paginated(&state, &params).await?;
    Ok(Json(PaginatedStatsResponse::from(result)))
}

/// Get a stat by id.
///
/// - Requires a valid JWT; responds 404 if not found.
#[utoipa::path(
    get,
    path = "/api/v3/stats/{id}",
    params(("id" = u32, Path, description = "Stat ID")),
    responses((status = 200, description = "Stat detail", body = StatResponse)),
    tag = "Stats",
    security(("jwt" = []))
)]
pub async fn get_stat(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<Json<StatResponse>> {
    let item = crate::service::stats::get_by_id(&state, id).await?;
    Ok(Json(item))
}

/// Create a new stat entry.
///
/// - Requires a valid JWT.
#[utoipa::path(
    post,
    path = "/api/v3/stats",
    request_body = CreateStatRequest,
    responses((status = 201, description = "Created stat", body = StatResponse)),
    tag = "Stats",
    security(("jwt" = []))
)]
pub async fn create_stat(
    State(state): State<AppState>,
    Json(req): Json<CreateStatRequest>,
) -> AppResult<(axum::http::StatusCode, Json<StatResponse>)> {
    let item = crate::service::stats::create(&state, req).await?;
    Ok((axum::http::StatusCode::CREATED, Json(item)))
}

/// Update a stat entry.
///
/// - Partial update.
/// - Requires a valid JWT.
#[utoipa::path(
    patch,
    path = "/api/v3/stats/{id}",
    params(("id" = u32, Path, description = "Stat ID")),
    request_body = UpdateStatRequest,
    responses((status = 200, description = "Updated stat", body = StatResponse)),
    tag = "Stats",
    security(("jwt" = []))
)]
pub async fn update_stat(
    Path(id): Path<u32>,
    State(state): State<AppState>,
    Json(req): Json<UpdateStatRequest>,
) -> AppResult<Json<StatResponse>> {
    let item = crate::service::stats::update(&state, id, req).await?;
    Ok(Json(item))
}

/// Delete a stat by id.
///
/// - Responds 204 on success, 404 if not found.
/// - Requires a valid JWT.
#[utoipa::path(
    delete,
    path = "/api/v3/stats/{id}",
    params(("id" = u32, Path, description = "Stat ID")),
    responses((status = 204, description = "Deleted stat")),
    tag = "Stats",
    security(("jwt" = []))
)]
pub async fn delete_stat(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<axum::http::StatusCode> {
    crate::service::stats::delete(&state, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
