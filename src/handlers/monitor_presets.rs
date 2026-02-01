use crate::dto::request::monitor_presets::{
    CreateMonitorPresetRequest, UpdateMonitorPresetRequest,
};
use crate::dto::response::monitor_presets::PaginatedMonitorPresetsResponse;
use crate::dto::response::MonitorPresetResponse;
use crate::dto::PaginationParams;
use crate::error::AppResult;
use crate::server::state::AppState;
use axum::{
    extract::{Path, Query, State},
    Json,
};

/// List all monitor presets with pagination.
///
/// - Requires a valid JWT.
#[utoipa::path(
    get,
    path = "/api/v3/monitor_presets",
    params(
        ("page" = Option<u64>, Query, description = "Page number (1-indexed)", example = 1),
        ("page_size" = Option<u64>, Query, description = "Items per page (max 1000)", example = 25)
    ),
    responses((status = 200, description = "Paginated list of monitor presets", body = PaginatedMonitorPresetsResponse)),
    tag = "Monitor Presets",
    security(("jwt" = []))
)]
pub async fn list_monitor_presets(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> AppResult<Json<PaginatedMonitorPresetsResponse>> {
    let result = crate::service::monitor_presets::list_paginated(&state, &params).await?;
    Ok(Json(PaginatedMonitorPresetsResponse::from(result)))
}

/// Get a monitor preset by id.
///
/// - Requires a valid JWT; responds 404 if not found.
#[utoipa::path(
    get,
    path = "/api/v3/monitor_presets/{id}",
    params(("id" = u32, Path, description = "Monitor Preset ID")),
    responses((status = 200, description = "Monitor preset detail", body = MonitorPresetResponse)),
    tag = "Monitor Presets",
    security(("jwt" = []))
)]
pub async fn get_monitor_preset(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<Json<MonitorPresetResponse>> {
    let item = crate::service::monitor_presets::get_by_id(&state, id).await?;
    Ok(Json(item))
}

/// Create a new monitor preset entry.
///
/// - Requires a valid JWT.
#[utoipa::path(
    post,
    path = "/api/v3/monitor_presets",
    request_body = CreateMonitorPresetRequest,
    responses((status = 201, description = "Created monitor preset", body = MonitorPresetResponse)),
    tag = "Monitor Presets",
    security(("jwt" = []))
)]
pub async fn create_monitor_preset(
    State(state): State<AppState>,
    Json(req): Json<CreateMonitorPresetRequest>,
) -> AppResult<(axum::http::StatusCode, Json<MonitorPresetResponse>)> {
    let item = crate::service::monitor_presets::create(&state, req).await?;
    Ok((axum::http::StatusCode::CREATED, Json(item)))
}

/// Update a monitor preset entry.
///
/// - Partial update.
/// - Requires a valid JWT.
#[utoipa::path(
    patch,
    path = "/api/v3/monitor_presets/{id}",
    params(("id" = u32, Path, description = "Monitor Preset ID")),
    request_body = UpdateMonitorPresetRequest,
    responses((status = 200, description = "Updated monitor preset", body = MonitorPresetResponse)),
    tag = "Monitor Presets",
    security(("jwt" = []))
)]
pub async fn update_monitor_preset(
    Path(id): Path<u32>,
    State(state): State<AppState>,
    Json(req): Json<UpdateMonitorPresetRequest>,
) -> AppResult<Json<MonitorPresetResponse>> {
    let item = crate::service::monitor_presets::update(&state, id, req).await?;
    Ok(Json(item))
}

/// Delete a monitor preset by id.
///
/// - Responds 204 on success, 404 if not found.
/// - Requires a valid JWT.
#[utoipa::path(
    delete,
    path = "/api/v3/monitor_presets/{id}",
    params(("id" = u32, Path, description = "Monitor Preset ID")),
    responses((status = 204, description = "Deleted monitor preset")),
    tag = "Monitor Presets",
    security(("jwt" = []))
)]
pub async fn delete_monitor_preset(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<axum::http::StatusCode> {
    crate::service::monitor_presets::delete(&state, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
