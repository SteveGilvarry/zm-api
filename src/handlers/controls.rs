use crate::dto::request::controls::{CreateControlRequest, UpdateControlRequest};
use crate::dto::response::controls::PaginatedControlsResponse;
use crate::dto::response::ControlResponse;
use crate::dto::PaginationParams;
use crate::error::AppResult;
use crate::server::state::AppState;
use axum::{
    extract::{Path, Query, State},
    Json,
};

/// List all camera controls with pagination.
///
/// - Requires a valid JWT.
#[utoipa::path(
    get,
    path = "/api/v3/controls",
    params(
        ("page" = Option<u64>, Query, description = "Page number (1-indexed)", example = 1),
        ("page_size" = Option<u64>, Query, description = "Items per page (max 1000)", example = 25)
    ),
    responses((status = 200, description = "Paginated list of controls", body = PaginatedControlsResponse)),
    tag = "Controls",
    security(("jwt" = []))
)]
pub async fn list_controls(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> AppResult<Json<PaginatedControlsResponse>> {
    let result = crate::service::controls::list_paginated(&state, &params).await?;
    Ok(Json(PaginatedControlsResponse::from(result)))
}

/// Get control by id.
///
/// - Requires a valid JWT; responds 404 if not found.
#[utoipa::path(
    get,
    path = "/api/v3/controls/{id}",
    params(("id" = u32, Path, description = "Control ID")),
    responses((status = 200, description = "Control detail", body = ControlResponse)),
    tag = "Controls",
    security(("jwt" = []))
)]
pub async fn get_control(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<Json<ControlResponse>> {
    let item = crate::service::controls::get_by_id(&state, id).await?;
    Ok(Json(item))
}

/// Create a new control record.
///
/// - Requires a valid JWT.
#[utoipa::path(
    post,
    path = "/api/v3/controls",
    request_body = CreateControlRequest,
    responses((status = 201, description = "Created control", body = ControlResponse)),
    tag = "Controls",
    security(("jwt" = []))
)]
pub async fn create_control(
    State(state): State<AppState>,
    Json(req): Json<CreateControlRequest>,
) -> AppResult<(axum::http::StatusCode, Json<ControlResponse>)> {
    let item = crate::service::controls::create(&state, req).await?;
    Ok((axum::http::StatusCode::CREATED, Json(item)))
}

/// Update a control.
///
/// - Partial update.
/// - Requires a valid JWT.
#[utoipa::path(
    patch,
    path = "/api/v3/controls/{id}",
    params(("id" = u32, Path, description = "Control ID")),
    request_body = UpdateControlRequest,
    responses((status = 200, description = "Updated control", body = ControlResponse)),
    tag = "Controls",
    security(("jwt" = []))
)]
pub async fn update_control(
    Path(id): Path<u32>,
    State(state): State<AppState>,
    Json(req): Json<UpdateControlRequest>,
) -> AppResult<Json<ControlResponse>> {
    let item = crate::service::controls::update(&state, id, req).await?;
    Ok(Json(item))
}

/// Delete a control by id.
///
/// - Responds 204 on success, 404 if not found.
/// - Requires a valid JWT.
#[utoipa::path(
    delete,
    path = "/api/v3/controls/{id}",
    params(("id" = u32, Path, description = "Control ID")),
    responses((status = 204, description = "Deleted control")),
    tag = "Controls",
    security(("jwt" = []))
)]
pub async fn delete_control(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<axum::http::StatusCode> {
    crate::service::controls::delete(&state, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
