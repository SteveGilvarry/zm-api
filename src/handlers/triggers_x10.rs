use axum::{extract::{Path, State}, Json};
use crate::dto::response::TriggerX10Response;
use crate::dto::request::triggers_x10::{CreateTriggerX10Request, UpdateTriggerX10Request};
use crate::error::AppResult;
use crate::server::state::AppState;

/// List all X10 triggers.
///
/// - Requires a valid JWT.
#[utoipa::path(
    get,
    path = "/api/v3/triggers_x10",
    responses((status = 200, description = "List X10 triggers", body = [TriggerX10Response])),
    tag = "TriggersX10",
    security(("jwt" = []))
)]
pub async fn list_triggers_x10(State(state): State<AppState>) -> AppResult<Json<Vec<TriggerX10Response>>> {
    let items = crate::service::triggers_x10::list_all(&state).await?;
    Ok(Json(items))
}

/// Get an X10 trigger by monitor_id.
///
/// - Requires a valid JWT; responds 404 if not found.
#[utoipa::path(
    get,
    path = "/api/v3/triggers_x10/{monitor_id}",
    params(("monitor_id" = u32, Path, description = "Monitor ID")),
    responses((status = 200, description = "X10 trigger detail", body = TriggerX10Response)),
    tag = "TriggersX10",
    security(("jwt" = []))
)]
pub async fn get_trigger_x10(Path(monitor_id): Path<u32>, State(state): State<AppState>) -> AppResult<Json<TriggerX10Response>> {
    let item = crate::service::triggers_x10::get_by_id(&state, monitor_id).await?;
    Ok(Json(item))
}

/// Create a new X10 trigger entry.
///
/// - Requires a valid JWT.
/// - monitor_id is the primary key.
#[utoipa::path(
    post,
    path = "/api/v3/triggers_x10",
    request_body = CreateTriggerX10Request,
    responses((status = 201, description = "Created X10 trigger", body = TriggerX10Response)),
    tag = "TriggersX10",
    security(("jwt" = []))
)]
pub async fn create_trigger_x10(State(state): State<AppState>, Json(req): Json<CreateTriggerX10Request>) -> AppResult<(axum::http::StatusCode, Json<TriggerX10Response>)> {
    let item = crate::service::triggers_x10::create(&state, req).await?;
    Ok((axum::http::StatusCode::CREATED, Json(item)))
}

/// Update an X10 trigger entry.
///
/// - Partial update.
/// - Requires a valid JWT.
#[utoipa::path(
    patch,
    path = "/api/v3/triggers_x10/{monitor_id}",
    params(("monitor_id" = u32, Path, description = "Monitor ID")),
    request_body = UpdateTriggerX10Request,
    responses((status = 200, description = "Updated X10 trigger", body = TriggerX10Response)),
    tag = "TriggersX10",
    security(("jwt" = []))
)]
pub async fn update_trigger_x10(Path(monitor_id): Path<u32>, State(state): State<AppState>, Json(req): Json<UpdateTriggerX10Request>) -> AppResult<Json<TriggerX10Response>> {
    let item = crate::service::triggers_x10::update(&state, monitor_id, req).await?;
    Ok(Json(item))
}

/// Delete an X10 trigger by monitor_id.
///
/// - Responds 204 on success, 404 if not found.
/// - Requires a valid JWT.
#[utoipa::path(
    delete,
    path = "/api/v3/triggers_x10/{monitor_id}",
    params(("monitor_id" = u32, Path, description = "Monitor ID")),
    responses((status = 204, description = "Deleted X10 trigger")),
    tag = "TriggersX10",
    security(("jwt" = []))
)]
pub async fn delete_trigger_x10(Path(monitor_id): Path<u32>, State(state): State<AppState>) -> AppResult<axum::http::StatusCode> {
    crate::service::triggers_x10::delete(&state, monitor_id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
