use axum::{extract::{Path, State, Query}, Json};
use serde::Deserialize;
use crate::dto::response::MonitorPresetResponse;
use crate::dto::request::monitor_presets::{CreateMonitorPresetRequest, UpdateMonitorPresetRequest};
use crate::error::AppResult;
use crate::server::state::AppState;

#[derive(Debug, Deserialize)]
pub struct MonitorPresetQuery { pub model_id: Option<u32> }

/// List monitor presets; optionally filter by model id.
///
/// - Requires a valid JWT.
#[utoipa::path(
    get,
    path = "/api/v3/monitor_presets",
    params(("model_id" = Option<u32>, Query, description = "Filter by model")),
    responses((status = 200, description = "List monitor presets", body = [MonitorPresetResponse])),
    tag = "Monitor Presets",
    security(("jwt" = []))
)]
pub async fn list_monitor_presets(State(state): State<AppState>, Query(q): Query<MonitorPresetQuery>) -> AppResult<Json<Vec<MonitorPresetResponse>>> {
    let items = crate::service::monitor_presets::list_all(&state, q.model_id).await?;
    Ok(Json(items))
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
pub async fn get_monitor_preset(Path(id): Path<u32>, State(state): State<AppState>) -> AppResult<Json<MonitorPresetResponse>> {
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
pub async fn create_monitor_preset(State(state): State<AppState>, Json(req): Json<CreateMonitorPresetRequest>) -> AppResult<(axum::http::StatusCode, Json<MonitorPresetResponse>)> {
    let item = crate::service::monitor_presets::create(&state, req).await?;
    Ok((axum::http::StatusCode::CREATED, Json(item)))}

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
pub async fn update_monitor_preset(Path(id): Path<u32>, State(state): State<AppState>, Json(req): Json<UpdateMonitorPresetRequest>) -> AppResult<Json<MonitorPresetResponse>> {
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
pub async fn delete_monitor_preset(Path(id): Path<u32>, State(state): State<AppState>) -> AppResult<axum::http::StatusCode> {
    crate::service::monitor_presets::delete(&state, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
