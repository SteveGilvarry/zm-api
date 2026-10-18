//! `/api/v3/monitors/{id}/actions`: ZoneMinder 1.39.30's MonitorActions
//! (#179). Editing is Monitors-gated; running a Manual action is
//! Control-gated, on the monitor and on the target.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use garde::Validate;

use crate::dto::request::monitor_actions::{
    CreateMonitorActionRequest, UpdateMonitorActionRequest,
};
use crate::dto::response::monitor_actions::{ActionTargetResponse, MonitorActionResponse};
use crate::dto::response::ptz::PtzCommandResponse;
use crate::error::{AppError, AppResponseError, AppResult};
use crate::server::state::AppState;
use crate::service::monitor_acl::MonitorScope;
use crate::service::monitor_actions as svc;

/// List a monitor's actions
#[utoipa::path(
    get,
    path = "/api/v3/monitors/{id}/actions",
    operation_id = "listMonitorActions",
    params(("id" = u32, Path, description = "Monitor ID")),
    responses(
        (status = 200, description = "The monitor's actions, in sequence order", body = Vec<MonitorActionResponse>),
        (status = 404, description = "Monitor not found", body = AppResponseError)
    ),
    tag = "Monitors",
    security(("jwt" = []))
)]
pub async fn list_actions(
    State(state): State<AppState>,
    Path(id): Path<u32>,
) -> AppResult<Json<Vec<MonitorActionResponse>>> {
    Ok(Json(svc::list(&state, id).await?))
}

/// Monitors an action can drive
///
/// Each controllable monitor whose control can do at least one action type,
/// with those types and its audio file range (ZoneMinder's
/// `MonitorAction::targetCandidates`).
#[utoipa::path(
    get,
    path = "/api/v3/monitors/{id}/actions/targets",
    operation_id = "listMonitorActionTargets",
    params(("id" = u32, Path, description = "Monitor ID")),
    responses((status = 200, description = "Action targets", body = Vec<ActionTargetResponse>)),
    tag = "Monitors",
    security(("jwt" = []))
)]
pub async fn list_targets(
    State(state): State<AppState>,
    scope: MonitorScope,
    Path(_id): Path<u32>,
) -> AppResult<Json<Vec<ActionTargetResponse>>> {
    Ok(Json(svc::targets(&state, &scope).await?))
}

/// Get one monitor action
#[utoipa::path(
    get,
    path = "/api/v3/monitors/{id}/actions/{action_id}",
    operation_id = "getMonitorAction",
    params(
        ("id" = u32, Path, description = "Monitor ID"),
        ("action_id" = u32, Path, description = "Action ID")
    ),
    responses(
        (status = 200, description = "Monitor action", body = MonitorActionResponse),
        (status = 404, description = "Not found", body = AppResponseError)
    ),
    tag = "Monitors",
    security(("jwt" = []))
)]
pub async fn get_action(
    State(state): State<AppState>,
    Path((id, action_id)): Path<(u32, u32)>,
) -> AppResult<Json<MonitorActionResponse>> {
    Ok(Json(svc::get(&state, id, action_id).await?))
}

/// Add a monitor action
///
/// A type the target's control can't perform is a 400. zmc reads the list at
/// start, so the monitor's daemons restart.
#[utoipa::path(
    post,
    path = "/api/v3/monitors/{id}/actions",
    operation_id = "createMonitorAction",
    params(("id" = u32, Path, description = "Monitor ID")),
    request_body = CreateMonitorActionRequest,
    responses(
        (status = 201, description = "Created", body = MonitorActionResponse),
        (status = 400, description = "The target can't perform it", body = AppResponseError),
        (status = 404, description = "Monitor not found", body = AppResponseError)
    ),
    tag = "Monitors",
    security(("jwt" = []))
)]
pub async fn create_action(
    State(state): State<AppState>,
    scope: MonitorScope,
    Path(id): Path<u32>,
    Json(req): Json<CreateMonitorActionRequest>,
) -> AppResult<(StatusCode, Json<MonitorActionResponse>)> {
    req.validate().map_err(AppError::InvalidInputError)?;
    Ok((
        StatusCode::CREATED,
        Json(svc::create(&state, &scope, id, req).await?),
    ))
}

/// Change a monitor action
#[utoipa::path(
    patch,
    path = "/api/v3/monitors/{id}/actions/{action_id}",
    operation_id = "updateMonitorAction",
    params(
        ("id" = u32, Path, description = "Monitor ID"),
        ("action_id" = u32, Path, description = "Action ID")
    ),
    request_body = UpdateMonitorActionRequest,
    responses(
        (status = 200, description = "Updated", body = MonitorActionResponse),
        (status = 400, description = "The target can't perform it", body = AppResponseError),
        (status = 404, description = "Not found", body = AppResponseError)
    ),
    tag = "Monitors",
    security(("jwt" = []))
)]
pub async fn update_action(
    State(state): State<AppState>,
    scope: MonitorScope,
    Path((id, action_id)): Path<(u32, u32)>,
    Json(req): Json<UpdateMonitorActionRequest>,
) -> AppResult<Json<MonitorActionResponse>> {
    req.validate().map_err(AppError::InvalidInputError)?;
    Ok(Json(svc::update(&state, &scope, id, action_id, req).await?))
}

/// Delete a monitor action
#[utoipa::path(
    delete,
    path = "/api/v3/monitors/{id}/actions/{action_id}",
    operation_id = "deleteMonitorAction",
    params(
        ("id" = u32, Path, description = "Monitor ID"),
        ("action_id" = u32, Path, description = "Action ID")
    ),
    responses(
        (status = 204, description = "Deleted"),
        (status = 404, description = "Not found", body = AppResponseError)
    ),
    tag = "Monitors",
    security(("jwt" = []))
)]
pub async fn delete_action(
    State(state): State<AppState>,
    Path((id, action_id)): Path<(u32, u32)>,
) -> AppResult<StatusCode> {
    svc::delete(&state, id, action_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Run a Manual action
///
/// Sends the stored action's command to the target's control. Needs Control
/// view on the monitor and on the target; the command comes from the row,
/// not the request.
#[utoipa::path(
    post,
    path = "/api/v3/monitors/{id}/actions/{action_id}/run",
    operation_id = "runMonitorAction",
    params(
        ("id" = u32, Path, description = "Monitor ID"),
        ("action_id" = u32, Path, description = "Action ID")
    ),
    responses(
        (status = 200, description = "Command sent", body = PtzCommandResponse),
        (status = 400, description = "Not an enabled Manual action, or the control can't do it", body = AppResponseError),
        (status = 404, description = "Not found", body = AppResponseError)
    ),
    tag = "Monitors",
    security(("jwt" = []))
)]
pub async fn run_action(
    State(state): State<AppState>,
    scope: MonitorScope,
    Path((id, action_id)): Path<(u32, u32)>,
) -> AppResult<Json<PtzCommandResponse>> {
    Ok(Json(svc::run(&state, &scope, id, action_id).await?))
}
