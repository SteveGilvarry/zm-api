use crate::dto::request::reports::{CreateReportRequest, UpdateReportRequest};
use crate::dto::response::reports::PaginatedReportsResponse;
use crate::dto::response::ReportResponse;
use crate::dto::PaginationParams;
use crate::error::AppResult;
use crate::server::state::AppState;
use axum::{
    extract::{Path, Query, State},
    Json,
};

/// List all reports with pagination.
///
/// - Requires a valid JWT.
#[utoipa::path(
    get,
    path = "/api/v3/reports",
    params(
        ("page" = Option<u64>, Query, description = "Page number (1-indexed)", example = 1),
        ("page_size" = Option<u64>, Query, description = "Items per page (max 1000)", example = 25)
    ),
    responses((status = 200, description = "Paginated list of reports", body = PaginatedReportsResponse)),
    tag = "Reports",
    security(("jwt" = []))
)]
pub async fn list_reports(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> AppResult<Json<PaginatedReportsResponse>> {
    let result = crate::service::reports::list_paginated(&state, &params).await?;
    Ok(Json(PaginatedReportsResponse::from(result)))
}

/// Get a report by id.
#[utoipa::path(
    get,
    path = "/api/v3/reports/{id}",
    params(("id" = u32, Path, description = "Report ID")),
    responses((status = 200, description = "Report detail", body = ReportResponse)),
    tag = "Reports",
    security(("jwt" = []))
)]
pub async fn get_report(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<Json<ReportResponse>> {
    let item = crate::service::reports::get_by_id(&state, id).await?;
    Ok(Json(item))
}

/// Create a new report.
#[utoipa::path(
    post,
    path = "/api/v3/reports",
    request_body = CreateReportRequest,
    responses((status = 201, description = "Created report", body = ReportResponse)),
    tag = "Reports",
    security(("jwt" = []))
)]
pub async fn create_report(
    State(state): State<AppState>,
    Json(req): Json<CreateReportRequest>,
) -> AppResult<(axum::http::StatusCode, Json<ReportResponse>)> {
    let item = crate::service::reports::create(&state, req).await?;
    Ok((axum::http::StatusCode::CREATED, Json(item)))
}

/// Update a report.
#[utoipa::path(
    patch,
    path = "/api/v3/reports/{id}",
    params(("id" = u32, Path, description = "Report ID")),
    request_body = UpdateReportRequest,
    responses((status = 200, description = "Updated report", body = ReportResponse)),
    tag = "Reports",
    security(("jwt" = []))
)]
pub async fn update_report(
    Path(id): Path<u32>,
    State(state): State<AppState>,
    Json(req): Json<UpdateReportRequest>,
) -> AppResult<Json<ReportResponse>> {
    let item = crate::service::reports::update(&state, id, req).await?;
    Ok(Json(item))
}

/// Delete a report by id.
#[utoipa::path(
    delete,
    path = "/api/v3/reports/{id}",
    params(("id" = u32, Path, description = "Report ID")),
    responses((status = 204, description = "Deleted report")),
    tag = "Reports",
    security(("jwt" = []))
)]
pub async fn delete_report(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<axum::http::StatusCode> {
    crate::service::reports::delete(&state, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
