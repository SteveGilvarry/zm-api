use crate::dto::request::control_presets::{
    CreateControlPresetRequest, UpdateControlPresetRequest,
};
use crate::dto::response::control_presets::PaginatedControlPresetsResponse;
use crate::dto::response::ControlPresetResponse;
use crate::dto::PaginationParams;
use crate::error::AppResult;
use crate::server::state::AppState;
use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ControlPresetQuery {
    pub monitor_id: Option<u32>,
    #[serde(flatten)]
    pub pagination: PaginationParams,
}

/// List control presets with pagination; optionally filter by monitor id.
///
/// - Requires a valid JWT.
#[utoipa::path(
    get,
    path = "/api/v3/control_presets",
    params(
        ("monitor_id" = Option<u32>, Query, description = "Filter by monitor"),
        ("page" = Option<u64>, Query, description = "Page number (1-indexed)", example = 1),
        ("page_size" = Option<u64>, Query, description = "Items per page (max 1000)", example = 25)
    ),
    responses((status = 200, description = "Paginated list of control presets", body = PaginatedControlPresetsResponse)),
    tag = "Control Presets",
    security(("jwt" = []))
)]
pub async fn list_control_presets(
    State(state): State<AppState>,
    Query(q): Query<ControlPresetQuery>,
) -> AppResult<Json<PaginatedControlPresetsResponse>> {
    let result = if let Some(mid) = q.monitor_id {
        // For filtered results, get all matching and build paginated response
        let items = crate::service::control_presets::list_by_monitor(&state, mid).await?;
        let total = items.len() as u64;
        crate::dto::PaginatedResponse::from_params(items, total, &q.pagination)
    } else {
        crate::service::control_presets::list_paginated(&state, &q.pagination).await?
    };
    Ok(Json(PaginatedControlPresetsResponse::from(result)))
}

/// Get a control preset by monitor ID and preset number.
///
/// - Requires a valid JWT; responds 404 if not found.
#[utoipa::path(
    get,
    path = "/api/v3/control_presets/{monitor_id}/{preset}",
    params(
        ("monitor_id" = u32, Path, description = "Monitor ID"),
        ("preset" = u32, Path, description = "Preset number")
    ),
    responses((status = 200, description = "Control preset detail", body = ControlPresetResponse)),
    tag = "Control Presets",
    security(("jwt" = []))
)]
pub async fn get_control_preset(
    Path((monitor_id, preset)): Path<(u32, u32)>,
    State(state): State<AppState>,
) -> AppResult<Json<ControlPresetResponse>> {
    let item = crate::service::control_presets::get_by_id(&state, monitor_id, preset).await?;
    Ok(Json(item))
}

/// Create a new control preset entry.
///
/// - Requires a valid JWT.
#[utoipa::path(
    post,
    path = "/api/v3/control_presets",
    request_body = CreateControlPresetRequest,
    responses((status = 201, description = "Created control preset", body = ControlPresetResponse)),
    tag = "Control Presets",
    security(("jwt" = []))
)]
pub async fn create_control_preset(
    State(state): State<AppState>,
    Json(req): Json<CreateControlPresetRequest>,
) -> AppResult<(axum::http::StatusCode, Json<ControlPresetResponse>)> {
    let item = crate::service::control_presets::create(&state, req).await?;
    Ok((axum::http::StatusCode::CREATED, Json(item)))
}

/// Update a control preset entry.
///
/// - Partial update.
/// - Requires a valid JWT.
#[utoipa::path(
    patch,
    path = "/api/v3/control_presets/{monitor_id}/{preset}",
    params(
        ("monitor_id" = u32, Path, description = "Monitor ID"),
        ("preset" = u32, Path, description = "Preset number")
    ),
    request_body = UpdateControlPresetRequest,
    responses((status = 200, description = "Updated control preset", body = ControlPresetResponse)),
    tag = "Control Presets",
    security(("jwt" = []))
)]
pub async fn update_control_preset(
    Path((monitor_id, preset)): Path<(u32, u32)>,
    State(state): State<AppState>,
    Json(req): Json<UpdateControlPresetRequest>,
) -> AppResult<Json<ControlPresetResponse>> {
    let item = crate::service::control_presets::update(&state, monitor_id, preset, req).await?;
    Ok(Json(item))
}

/// Delete a control preset by monitor ID and preset number.
///
/// - Responds 204 on success, 404 if not found.
/// - Requires a valid JWT.
#[utoipa::path(
    delete,
    path = "/api/v3/control_presets/{monitor_id}/{preset}",
    params(
        ("monitor_id" = u32, Path, description = "Monitor ID"),
        ("preset" = u32, Path, description = "Preset number")
    ),
    responses((status = 204, description = "Deleted control preset")),
    tag = "Control Presets",
    security(("jwt" = []))
)]
pub async fn delete_control_preset(
    Path((monitor_id, preset)): Path<(u32, u32)>,
    State(state): State<AppState>,
) -> AppResult<axum::http::StatusCode> {
    crate::service::control_presets::delete(&state, monitor_id, preset).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
