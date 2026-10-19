//! PTZ control handlers

use axum::{
    extract::{Path, State},
    Json,
};
use garde::Validate;
use tracing::instrument;

use crate::dto::request::ptz::{
    PtzAbsoluteRequest, PtzFocusRequest, PtzMoveRequest, PtzPresetRequest, PtzRelativeRequest,
    PtzZoomRequest,
};
use crate::dto::response::ptz::{
    PtzCapabilitiesResponse, PtzCommandResponse, PtzProtocolListResponse, PtzStatusResponse,
};
use crate::error::{AppError, AppResponseError, AppResult};
use crate::server::state::AppState;
use crate::service;

/// Get PTZ status for a monitor
#[utoipa::path(
    get,
    path = "/api/v3/ptz/monitors/{id}/status",
    operation_id = "getPtzStatus",
    tag = "PTZ",
    params(
        ("id" = u32, Path, description = "Monitor ID")
    ),
    responses(
        (status = 200, description = "PTZ status", body = PtzStatusResponse),
        (status = 404, description = "Monitor not found", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(("jwt" = []))
)]
#[instrument(skip(state))]
pub async fn get_status(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<Json<PtzStatusResponse>> {
    let ptz_manager = state.ptz_manager();
    let result = service::ptz::get_status(&state, ptz_manager, id).await?;
    Ok(Json(result))
}

/// Get PTZ capabilities for a monitor
#[utoipa::path(
    get,
    path = "/api/v3/ptz/monitors/{id}/capabilities",
    operation_id = "getPtzCapabilities",
    tag = "PTZ",
    params(
        ("id" = u32, Path, description = "Monitor ID")
    ),
    responses(
        (status = 200, description = "PTZ capabilities", body = PtzCapabilitiesResponse),
        (status = 404, description = "Monitor not found", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(("jwt" = []))
)]
#[instrument(skip(state))]
pub async fn get_capabilities(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<Json<PtzCapabilitiesResponse>> {
    let result = service::ptz::get_capabilities(&state, id).await?;
    Ok(Json(result))
}

/// List available PTZ protocols
#[utoipa::path(
    get,
    path = "/api/v3/ptz/protocols",
    operation_id = "listPtzProtocols",
    tag = "PTZ",
    responses(
        (status = 200, description = "List of PTZ protocols", body = PtzProtocolListResponse),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(("jwt" = []))
)]
#[instrument(skip(state))]
pub async fn list_protocols(
    State(state): State<AppState>,
) -> AppResult<Json<PtzProtocolListResponse>> {
    let ptz_manager = state.ptz_manager();
    let result = service::ptz::list_protocols(ptz_manager);
    Ok(Json(result))
}

/// Move camera up
#[utoipa::path(
    post,
    path = "/api/v3/ptz/monitors/{id}/move/up",
    operation_id = "ptzMoveUp",
    tag = "PTZ",
    params(("id" = u32, Path, description = "Monitor ID")),
    request_body = PtzMoveRequest,
    responses(
        (status = 200, description = "Command executed", body = PtzCommandResponse),
        (status = 400, description = "Bad request", body = AppResponseError),
        (status = 404, description = "Monitor not found", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(("jwt" = []))
)]
#[instrument(skip(state))]
pub async fn move_up(
    Path(id): Path<u32>,
    State(state): State<AppState>,
    Json(request): Json<PtzMoveRequest>,
) -> AppResult<Json<PtzCommandResponse>> {
    request.validate().map_err(AppError::InvalidInputError)?;
    let ptz_manager = state.ptz_manager();
    let result = service::ptz::move_direction(&state, ptz_manager, id, "up", request).await?;
    Ok(Json(result))
}

/// Move camera down
#[utoipa::path(
    post,
    path = "/api/v3/ptz/monitors/{id}/move/down",
    operation_id = "ptzMoveDown",
    tag = "PTZ",
    params(("id" = u32, Path, description = "Monitor ID")),
    request_body = PtzMoveRequest,
    responses(
        (status = 200, description = "Command executed", body = PtzCommandResponse),
        (status = 400, description = "Bad request", body = AppResponseError),
        (status = 404, description = "Monitor not found", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(("jwt" = []))
)]
#[instrument(skip(state))]
pub async fn move_down(
    Path(id): Path<u32>,
    State(state): State<AppState>,
    Json(request): Json<PtzMoveRequest>,
) -> AppResult<Json<PtzCommandResponse>> {
    request.validate().map_err(AppError::InvalidInputError)?;
    let ptz_manager = state.ptz_manager();
    let result = service::ptz::move_direction(&state, ptz_manager, id, "down", request).await?;
    Ok(Json(result))
}

/// Move camera left
#[utoipa::path(
    post,
    path = "/api/v3/ptz/monitors/{id}/move/left",
    operation_id = "ptzMoveLeft",
    tag = "PTZ",
    params(("id" = u32, Path, description = "Monitor ID")),
    request_body = PtzMoveRequest,
    responses(
        (status = 200, description = "Command executed", body = PtzCommandResponse),
        (status = 400, description = "Bad request", body = AppResponseError),
        (status = 404, description = "Monitor not found", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(("jwt" = []))
)]
#[instrument(skip(state))]
pub async fn move_left(
    Path(id): Path<u32>,
    State(state): State<AppState>,
    Json(request): Json<PtzMoveRequest>,
) -> AppResult<Json<PtzCommandResponse>> {
    request.validate().map_err(AppError::InvalidInputError)?;
    let ptz_manager = state.ptz_manager();
    let result = service::ptz::move_direction(&state, ptz_manager, id, "left", request).await?;
    Ok(Json(result))
}

/// Move camera right
#[utoipa::path(
    post,
    path = "/api/v3/ptz/monitors/{id}/move/right",
    operation_id = "ptzMoveRight",
    tag = "PTZ",
    params(("id" = u32, Path, description = "Monitor ID")),
    request_body = PtzMoveRequest,
    responses(
        (status = 200, description = "Command executed", body = PtzCommandResponse),
        (status = 400, description = "Bad request", body = AppResponseError),
        (status = 404, description = "Monitor not found", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(("jwt" = []))
)]
#[instrument(skip(state))]
pub async fn move_right(
    Path(id): Path<u32>,
    State(state): State<AppState>,
    Json(request): Json<PtzMoveRequest>,
) -> AppResult<Json<PtzCommandResponse>> {
    request.validate().map_err(AppError::InvalidInputError)?;
    let ptz_manager = state.ptz_manager();
    let result = service::ptz::move_direction(&state, ptz_manager, id, "right", request).await?;
    Ok(Json(result))
}

/// Move camera diagonally up-left
#[utoipa::path(
    post,
    path = "/api/v3/ptz/monitors/{id}/move/up-left",
    operation_id = "ptzMoveUpLeft",
    tag = "PTZ",
    params(("id" = u32, Path, description = "Monitor ID")),
    request_body = PtzMoveRequest,
    responses(
        (status = 200, description = "Command executed", body = PtzCommandResponse),
        (status = 400, description = "Bad request", body = AppResponseError),
        (status = 404, description = "Monitor not found", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(("jwt" = []))
)]
#[instrument(skip(state))]
pub async fn move_up_left(
    Path(id): Path<u32>,
    State(state): State<AppState>,
    Json(request): Json<PtzMoveRequest>,
) -> AppResult<Json<PtzCommandResponse>> {
    request.validate().map_err(AppError::InvalidInputError)?;
    let ptz_manager = state.ptz_manager();
    let result = service::ptz::move_direction(&state, ptz_manager, id, "up-left", request).await?;
    Ok(Json(result))
}

/// Move camera diagonally up-right
#[utoipa::path(
    post,
    path = "/api/v3/ptz/monitors/{id}/move/up-right",
    operation_id = "ptzMoveUpRight",
    tag = "PTZ",
    params(("id" = u32, Path, description = "Monitor ID")),
    request_body = PtzMoveRequest,
    responses(
        (status = 200, description = "Command executed", body = PtzCommandResponse),
        (status = 400, description = "Bad request", body = AppResponseError),
        (status = 404, description = "Monitor not found", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(("jwt" = []))
)]
#[instrument(skip(state))]
pub async fn move_up_right(
    Path(id): Path<u32>,
    State(state): State<AppState>,
    Json(request): Json<PtzMoveRequest>,
) -> AppResult<Json<PtzCommandResponse>> {
    request.validate().map_err(AppError::InvalidInputError)?;
    let ptz_manager = state.ptz_manager();
    let result = service::ptz::move_direction(&state, ptz_manager, id, "up-right", request).await?;
    Ok(Json(result))
}

/// Move camera diagonally down-left
#[utoipa::path(
    post,
    path = "/api/v3/ptz/monitors/{id}/move/down-left",
    operation_id = "ptzMoveDownLeft",
    tag = "PTZ",
    params(("id" = u32, Path, description = "Monitor ID")),
    request_body = PtzMoveRequest,
    responses(
        (status = 200, description = "Command executed", body = PtzCommandResponse),
        (status = 400, description = "Bad request", body = AppResponseError),
        (status = 404, description = "Monitor not found", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(("jwt" = []))
)]
#[instrument(skip(state))]
pub async fn move_down_left(
    Path(id): Path<u32>,
    State(state): State<AppState>,
    Json(request): Json<PtzMoveRequest>,
) -> AppResult<Json<PtzCommandResponse>> {
    request.validate().map_err(AppError::InvalidInputError)?;
    let ptz_manager = state.ptz_manager();
    let result =
        service::ptz::move_direction(&state, ptz_manager, id, "down-left", request).await?;
    Ok(Json(result))
}

/// Move camera diagonally down-right
#[utoipa::path(
    post,
    path = "/api/v3/ptz/monitors/{id}/move/down-right",
    operation_id = "ptzMoveDownRight",
    tag = "PTZ",
    params(("id" = u32, Path, description = "Monitor ID")),
    request_body = PtzMoveRequest,
    responses(
        (status = 200, description = "Command executed", body = PtzCommandResponse),
        (status = 400, description = "Bad request", body = AppResponseError),
        (status = 404, description = "Monitor not found", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(("jwt" = []))
)]
#[instrument(skip(state))]
pub async fn move_down_right(
    Path(id): Path<u32>,
    State(state): State<AppState>,
    Json(request): Json<PtzMoveRequest>,
) -> AppResult<Json<PtzCommandResponse>> {
    request.validate().map_err(AppError::InvalidInputError)?;
    let ptz_manager = state.ptz_manager();
    let result =
        service::ptz::move_direction(&state, ptz_manager, id, "down-right", request).await?;
    Ok(Json(result))
}

/// Stop camera movement
#[utoipa::path(
    post,
    path = "/api/v3/ptz/monitors/{id}/move/stop",
    operation_id = "ptzMoveStop",
    tag = "PTZ",
    params(("id" = u32, Path, description = "Monitor ID")),
    responses(
        (status = 200, description = "Command executed", body = PtzCommandResponse),
        (status = 404, description = "Monitor not found", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(("jwt" = []))
)]
#[instrument(skip(state))]
pub async fn move_stop(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<Json<PtzCommandResponse>> {
    let ptz_manager = state.ptz_manager();
    let result = service::ptz::move_stop(&state, ptz_manager, id).await?;
    Ok(Json(result))
}

/// Zoom in
#[utoipa::path(
    post,
    path = "/api/v3/ptz/monitors/{id}/zoom/in",
    operation_id = "ptzZoomIn",
    tag = "PTZ",
    params(("id" = u32, Path, description = "Monitor ID")),
    request_body = PtzZoomRequest,
    responses(
        (status = 200, description = "Command executed", body = PtzCommandResponse),
        (status = 400, description = "Bad request", body = AppResponseError),
        (status = 404, description = "Monitor not found", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(("jwt" = []))
)]
#[instrument(skip(state))]
pub async fn zoom_in(
    Path(id): Path<u32>,
    State(state): State<AppState>,
    Json(request): Json<PtzZoomRequest>,
) -> AppResult<Json<PtzCommandResponse>> {
    request.validate().map_err(AppError::InvalidInputError)?;
    let ptz_manager = state.ptz_manager();
    let result = service::ptz::zoom(&state, ptz_manager, id, "in", request).await?;
    Ok(Json(result))
}

/// Zoom out
#[utoipa::path(
    post,
    path = "/api/v3/ptz/monitors/{id}/zoom/out",
    operation_id = "ptzZoomOut",
    tag = "PTZ",
    params(("id" = u32, Path, description = "Monitor ID")),
    request_body = PtzZoomRequest,
    responses(
        (status = 200, description = "Command executed", body = PtzCommandResponse),
        (status = 400, description = "Bad request", body = AppResponseError),
        (status = 404, description = "Monitor not found", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(("jwt" = []))
)]
#[instrument(skip(state))]
pub async fn zoom_out(
    Path(id): Path<u32>,
    State(state): State<AppState>,
    Json(request): Json<PtzZoomRequest>,
) -> AppResult<Json<PtzCommandResponse>> {
    request.validate().map_err(AppError::InvalidInputError)?;
    let ptz_manager = state.ptz_manager();
    let result = service::ptz::zoom(&state, ptz_manager, id, "out", request).await?;
    Ok(Json(result))
}

/// Stop zoom
#[utoipa::path(
    post,
    path = "/api/v3/ptz/monitors/{id}/zoom/stop",
    operation_id = "ptzZoomStop",
    tag = "PTZ",
    params(("id" = u32, Path, description = "Monitor ID")),
    responses(
        (status = 200, description = "Command executed", body = PtzCommandResponse),
        (status = 404, description = "Monitor not found", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(("jwt" = []))
)]
#[instrument(skip(state))]
pub async fn zoom_stop(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<Json<PtzCommandResponse>> {
    let ptz_manager = state.ptz_manager();
    let result = service::ptz::zoom_stop(&state, ptz_manager, id).await?;
    Ok(Json(result))
}

/// Focus near
#[utoipa::path(
    post,
    path = "/api/v3/ptz/monitors/{id}/focus/near",
    operation_id = "ptzFocusNear",
    tag = "PTZ",
    params(("id" = u32, Path, description = "Monitor ID")),
    request_body = PtzFocusRequest,
    responses(
        (status = 200, description = "Command executed", body = PtzCommandResponse),
        (status = 400, description = "Bad request", body = AppResponseError),
        (status = 404, description = "Monitor not found", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(("jwt" = []))
)]
#[instrument(skip(state))]
pub async fn focus_near(
    Path(id): Path<u32>,
    State(state): State<AppState>,
    Json(request): Json<PtzFocusRequest>,
) -> AppResult<Json<PtzCommandResponse>> {
    request.validate().map_err(AppError::InvalidInputError)?;
    let ptz_manager = state.ptz_manager();
    let result = service::ptz::focus(&state, ptz_manager, id, "near", request).await?;
    Ok(Json(result))
}

/// Focus far
#[utoipa::path(
    post,
    path = "/api/v3/ptz/monitors/{id}/focus/far",
    operation_id = "ptzFocusFar",
    tag = "PTZ",
    params(("id" = u32, Path, description = "Monitor ID")),
    request_body = PtzFocusRequest,
    responses(
        (status = 200, description = "Command executed", body = PtzCommandResponse),
        (status = 400, description = "Bad request", body = AppResponseError),
        (status = 404, description = "Monitor not found", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(("jwt" = []))
)]
#[instrument(skip(state))]
pub async fn focus_far(
    Path(id): Path<u32>,
    State(state): State<AppState>,
    Json(request): Json<PtzFocusRequest>,
) -> AppResult<Json<PtzCommandResponse>> {
    request.validate().map_err(AppError::InvalidInputError)?;
    let ptz_manager = state.ptz_manager();
    let result = service::ptz::focus(&state, ptz_manager, id, "far", request).await?;
    Ok(Json(result))
}

/// Auto focus
#[utoipa::path(
    post,
    path = "/api/v3/ptz/monitors/{id}/focus/auto",
    operation_id = "ptzFocusAuto",
    tag = "PTZ",
    params(("id" = u32, Path, description = "Monitor ID")),
    responses(
        (status = 200, description = "Command executed", body = PtzCommandResponse),
        (status = 404, description = "Monitor not found", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(("jwt" = []))
)]
#[instrument(skip(state))]
pub async fn focus_auto(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<Json<PtzCommandResponse>> {
    let ptz_manager = state.ptz_manager();
    let result =
        service::ptz::focus(&state, ptz_manager, id, "auto", PtzFocusRequest::default()).await?;
    Ok(Json(result))
}

/// Stop focus
#[utoipa::path(
    post,
    path = "/api/v3/ptz/monitors/{id}/focus/stop",
    operation_id = "ptzFocusStop",
    tag = "PTZ",
    params(("id" = u32, Path, description = "Monitor ID")),
    responses(
        (status = 200, description = "Command executed", body = PtzCommandResponse),
        (status = 404, description = "Monitor not found", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(("jwt" = []))
)]
#[instrument(skip(state))]
pub async fn focus_stop(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<Json<PtzCommandResponse>> {
    let ptz_manager = state.ptz_manager();
    let result = service::ptz::focus_stop(&state, ptz_manager, id).await?;
    Ok(Json(result))
}

/// Open the iris
#[utoipa::path(
    post,
    path = "/api/v3/ptz/monitors/{id}/iris/open",
    operation_id = "ptzIrisOpen",
    tag = "PTZ",
    params(("id" = u32, Path, description = "Monitor ID")),
    responses(
        (status = 200, description = "Command executed", body = PtzCommandResponse),
        (status = 400, description = "Bad request", body = AppResponseError),
        (status = 404, description = "Monitor not found", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(("jwt" = []))
)]
#[instrument(skip(state))]
pub async fn iris_open(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<Json<PtzCommandResponse>> {
    let ptz_manager = state.ptz_manager();
    let result = service::ptz::iris(&state, ptz_manager, id, "open").await?;
    Ok(Json(result))
}

/// Close the iris
#[utoipa::path(
    post,
    path = "/api/v3/ptz/monitors/{id}/iris/close",
    operation_id = "ptzIrisClose",
    tag = "PTZ",
    params(("id" = u32, Path, description = "Monitor ID")),
    responses(
        (status = 200, description = "Command executed", body = PtzCommandResponse),
        (status = 400, description = "Bad request", body = AppResponseError),
        (status = 404, description = "Monitor not found", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(("jwt" = []))
)]
#[instrument(skip(state))]
pub async fn iris_close(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<Json<PtzCommandResponse>> {
    let ptz_manager = state.ptz_manager();
    let result = service::ptz::iris(&state, ptz_manager, id, "close").await?;
    Ok(Json(result))
}

/// Enable auto-iris
#[utoipa::path(
    post,
    path = "/api/v3/ptz/monitors/{id}/iris/auto",
    operation_id = "ptzIrisAuto",
    tag = "PTZ",
    params(("id" = u32, Path, description = "Monitor ID")),
    responses(
        (status = 200, description = "Command executed", body = PtzCommandResponse),
        (status = 400, description = "Bad request", body = AppResponseError),
        (status = 404, description = "Monitor not found", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(("jwt" = []))
)]
#[instrument(skip(state))]
pub async fn iris_auto(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<Json<PtzCommandResponse>> {
    let ptz_manager = state.ptz_manager();
    let result = service::ptz::iris(&state, ptz_manager, id, "auto").await?;
    Ok(Json(result))
}

/// Stop iris movement
#[utoipa::path(
    post,
    path = "/api/v3/ptz/monitors/{id}/iris/stop",
    operation_id = "ptzIrisStop",
    tag = "PTZ",
    params(("id" = u32, Path, description = "Monitor ID")),
    responses(
        (status = 200, description = "Command executed", body = PtzCommandResponse),
        (status = 400, description = "Bad request", body = AppResponseError),
        (status = 404, description = "Monitor not found", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(("jwt" = []))
)]
#[instrument(skip(state))]
pub async fn iris_stop(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<Json<PtzCommandResponse>> {
    let ptz_manager = state.ptz_manager();
    let result = service::ptz::iris(&state, ptz_manager, id, "stop").await?;
    Ok(Json(result))
}

/// Wake the camera
#[utoipa::path(
    post,
    path = "/api/v3/ptz/monitors/{id}/power/wake",
    operation_id = "ptzPowerWake",
    tag = "PTZ",
    params(("id" = u32, Path, description = "Monitor ID")),
    responses(
        (status = 200, description = "Command executed", body = PtzCommandResponse),
        (status = 400, description = "Bad request", body = AppResponseError),
        (status = 404, description = "Monitor not found", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(("jwt" = []))
)]
#[instrument(skip(state))]
pub async fn power_wake(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<Json<PtzCommandResponse>> {
    let ptz_manager = state.ptz_manager();
    let result = service::ptz::power(&state, ptz_manager, id, "wake").await?;
    Ok(Json(result))
}

/// Put the camera to sleep
#[utoipa::path(
    post,
    path = "/api/v3/ptz/monitors/{id}/power/sleep",
    operation_id = "ptzPowerSleep",
    tag = "PTZ",
    params(("id" = u32, Path, description = "Monitor ID")),
    responses(
        (status = 200, description = "Command executed", body = PtzCommandResponse),
        (status = 400, description = "Bad request", body = AppResponseError),
        (status = 404, description = "Monitor not found", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(("jwt" = []))
)]
#[instrument(skip(state))]
pub async fn power_sleep(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<Json<PtzCommandResponse>> {
    let ptz_manager = state.ptz_manager();
    let result = service::ptz::power(&state, ptz_manager, id, "sleep").await?;
    Ok(Json(result))
}

/// Reset the camera
#[utoipa::path(
    post,
    path = "/api/v3/ptz/monitors/{id}/power/reset",
    operation_id = "ptzPowerReset",
    tag = "PTZ",
    params(("id" = u32, Path, description = "Monitor ID")),
    responses(
        (status = 200, description = "Command executed", body = PtzCommandResponse),
        (status = 400, description = "Bad request", body = AppResponseError),
        (status = 404, description = "Monitor not found", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(("jwt" = []))
)]
#[instrument(skip(state))]
pub async fn power_reset(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<Json<PtzCommandResponse>> {
    let ptz_manager = state.ptz_manager();
    let result = service::ptz::power(&state, ptz_manager, id, "reset").await?;
    Ok(Json(result))
}

/// Reboot the camera
#[utoipa::path(
    post,
    path = "/api/v3/ptz/monitors/{id}/power/reboot",
    operation_id = "ptzPowerReboot",
    tag = "PTZ",
    params(("id" = u32, Path, description = "Monitor ID")),
    responses(
        (status = 200, description = "Command executed", body = PtzCommandResponse),
        (status = 400, description = "Bad request", body = AppResponseError),
        (status = 404, description = "Monitor not found", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(("jwt" = []))
)]
#[instrument(skip(state))]
pub async fn power_reboot(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<Json<PtzCommandResponse>> {
    let ptz_manager = state.ptz_manager();
    let result = service::ptz::power(&state, ptz_manager, id, "reboot").await?;
    Ok(Json(result))
}

/// Go to preset
#[utoipa::path(
    post,
    path = "/api/v3/ptz/monitors/{id}/presets/{preset_id}/goto",
    operation_id = "ptzGotoPreset",
    tag = "PTZ",
    params(
        ("id" = u32, Path, description = "Monitor ID"),
        ("preset_id" = u32, Path, description = "Preset ID")
    ),
    responses(
        (status = 200, description = "Command executed", body = PtzCommandResponse),
        (status = 404, description = "Monitor not found", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(("jwt" = []))
)]
#[instrument(skip(state))]
pub async fn goto_preset(
    Path((id, preset_id)): Path<(u32, u32)>,
    State(state): State<AppState>,
) -> AppResult<Json<PtzCommandResponse>> {
    let ptz_manager = state.ptz_manager();
    let result = service::ptz::goto_preset(&state, ptz_manager, id, preset_id).await?;
    Ok(Json(result))
}

/// Set preset
#[utoipa::path(
    post,
    path = "/api/v3/ptz/monitors/{id}/presets/{preset_id}/set",
    operation_id = "ptzSetPreset",
    tag = "PTZ",
    params(
        ("id" = u32, Path, description = "Monitor ID"),
        ("preset_id" = u32, Path, description = "Preset ID")
    ),
    request_body = PtzPresetRequest,
    responses(
        (status = 200, description = "Command executed", body = PtzCommandResponse),
        (status = 400, description = "Bad request", body = AppResponseError),
        (status = 404, description = "Monitor not found", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(("jwt" = []))
)]
#[instrument(skip(state))]
pub async fn set_preset(
    Path((id, preset_id)): Path<(u32, u32)>,
    State(state): State<AppState>,
    Json(request): Json<PtzPresetRequest>,
) -> AppResult<Json<PtzCommandResponse>> {
    request.validate().map_err(AppError::InvalidInputError)?;

    let ptz_manager = state.ptz_manager();
    let result = service::ptz::set_preset(&state, ptz_manager, id, preset_id, request).await?;
    Ok(Json(result))
}

/// Clear preset
#[utoipa::path(
    delete,
    path = "/api/v3/ptz/monitors/{id}/presets/{preset_id}",
    operation_id = "ptzClearPreset",
    tag = "PTZ",
    params(
        ("id" = u32, Path, description = "Monitor ID"),
        ("preset_id" = u32, Path, description = "Preset ID")
    ),
    responses(
        (status = 200, description = "Command executed", body = PtzCommandResponse),
        (status = 404, description = "Monitor not found", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(("jwt" = []))
)]
#[instrument(skip(state))]
pub async fn clear_preset(
    Path((id, preset_id)): Path<(u32, u32)>,
    State(state): State<AppState>,
) -> AppResult<Json<PtzCommandResponse>> {
    let ptz_manager = state.ptz_manager();
    let result = service::ptz::clear_preset(&state, ptz_manager, id, preset_id).await?;
    Ok(Json(result))
}

/// Go to home position
#[utoipa::path(
    post,
    path = "/api/v3/ptz/monitors/{id}/home",
    operation_id = "ptzGotoHome",
    tag = "PTZ",
    params(("id" = u32, Path, description = "Monitor ID")),
    responses(
        (status = 200, description = "Command executed", body = PtzCommandResponse),
        (status = 404, description = "Monitor not found", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(("jwt" = []))
)]
#[instrument(skip(state))]
pub async fn goto_home(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<Json<PtzCommandResponse>> {
    let ptz_manager = state.ptz_manager();
    let result = service::ptz::goto_home(&state, ptz_manager, id).await?;
    Ok(Json(result))
}

/// Move to absolute position
#[utoipa::path(
    post,
    path = "/api/v3/ptz/monitors/{id}/absolute",
    operation_id = "ptzMoveAbsolute",
    tag = "PTZ",
    params(("id" = u32, Path, description = "Monitor ID")),
    request_body = PtzAbsoluteRequest,
    responses(
        (status = 200, description = "Command executed", body = PtzCommandResponse),
        (status = 400, description = "Bad request", body = AppResponseError),
        (status = 404, description = "Monitor not found", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(("jwt" = []))
)]
#[instrument(skip(state))]
pub async fn move_absolute(
    Path(id): Path<u32>,
    State(state): State<AppState>,
    Json(request): Json<PtzAbsoluteRequest>,
) -> AppResult<Json<PtzCommandResponse>> {
    request.validate().map_err(AppError::InvalidInputError)?;
    let ptz_manager = state.ptz_manager();
    let result = service::ptz::move_absolute(&state, ptz_manager, id, request).await?;
    Ok(Json(result))
}

/// Move relative
#[utoipa::path(
    post,
    path = "/api/v3/ptz/monitors/{id}/relative",
    operation_id = "ptzMoveRelative",
    tag = "PTZ",
    params(("id" = u32, Path, description = "Monitor ID")),
    request_body = PtzRelativeRequest,
    responses(
        (status = 200, description = "Command executed", body = PtzCommandResponse),
        (status = 400, description = "Bad request", body = AppResponseError),
        (status = 404, description = "Monitor not found", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(("jwt" = []))
)]
#[instrument(skip(state))]
pub async fn move_relative(
    Path(id): Path<u32>,
    State(state): State<AppState>,
    Json(request): Json<PtzRelativeRequest>,
) -> AppResult<Json<PtzCommandResponse>> {
    request.validate().map_err(AppError::InvalidInputError)?;
    let ptz_manager = state.ptz_manager();
    let result = service::ptz::move_relative(&state, ptz_manager, id, request).await?;
    Ok(Json(result))
}

/// Turn the light on
#[utoipa::path(
    post,
    path = "/api/v3/ptz/monitors/{id}/light/on",
    operation_id = "ptzLightOn",
    tag = "PTZ",
    params(("id" = u32, Path, description = "Monitor ID")),
    responses(
        (status = 200, description = "Command executed", body = PtzCommandResponse),
        (status = 400, description = "The control can't do this", body = AppResponseError),
        (status = 404, description = "Monitor not found", body = AppResponseError)
    ),
    security(("jwt" = []))
)]
#[instrument(skip(state))]
pub async fn light_on(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<Json<PtzCommandResponse>> {
    let ptz_manager = state.ptz_manager();
    Ok(Json(
        service::ptz::light(&state, ptz_manager, id, false, true).await?,
    ))
}

/// Turn the light off
#[utoipa::path(
    post,
    path = "/api/v3/ptz/monitors/{id}/light/off",
    operation_id = "ptzLightOff",
    tag = "PTZ",
    params(("id" = u32, Path, description = "Monitor ID")),
    responses(
        (status = 200, description = "Command executed", body = PtzCommandResponse),
        (status = 400, description = "The control can't do this", body = AppResponseError),
        (status = 404, description = "Monitor not found", body = AppResponseError)
    ),
    security(("jwt" = []))
)]
#[instrument(skip(state))]
pub async fn light_off(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<Json<PtzCommandResponse>> {
    let ptz_manager = state.ptz_manager();
    Ok(Json(
        service::ptz::light(&state, ptz_manager, id, false, false).await?,
    ))
}

/// Turn the indicator light on
#[utoipa::path(
    post,
    path = "/api/v3/ptz/monitors/{id}/indicator-light/on",
    operation_id = "ptzIndicatorLightOn",
    tag = "PTZ",
    params(("id" = u32, Path, description = "Monitor ID")),
    responses(
        (status = 200, description = "Command executed", body = PtzCommandResponse),
        (status = 400, description = "The control can't do this", body = AppResponseError),
        (status = 404, description = "Monitor not found", body = AppResponseError)
    ),
    security(("jwt" = []))
)]
#[instrument(skip(state))]
pub async fn indicator_light_on(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<Json<PtzCommandResponse>> {
    let ptz_manager = state.ptz_manager();
    Ok(Json(
        service::ptz::light(&state, ptz_manager, id, true, true).await?,
    ))
}

/// Turn the indicator light off
#[utoipa::path(
    post,
    path = "/api/v3/ptz/monitors/{id}/indicator-light/off",
    operation_id = "ptzIndicatorLightOff",
    tag = "PTZ",
    params(("id" = u32, Path, description = "Monitor ID")),
    responses(
        (status = 200, description = "Command executed", body = PtzCommandResponse),
        (status = 400, description = "The control can't do this", body = AppResponseError),
        (status = 404, description = "Monitor not found", body = AppResponseError)
    ),
    security(("jwt" = []))
)]
#[instrument(skip(state))]
pub async fn indicator_light_off(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<Json<PtzCommandResponse>> {
    let ptz_manager = state.ptz_manager();
    Ok(Json(
        service::ptz::light(&state, ptz_manager, id, true, false).await?,
    ))
}

/// Stop audio playback
#[utoipa::path(
    post,
    path = "/api/v3/ptz/monitors/{id}/audio/stop",
    operation_id = "ptzAudioStop",
    tag = "PTZ",
    params(("id" = u32, Path, description = "Monitor ID")),
    responses(
        (status = 200, description = "Command executed", body = PtzCommandResponse),
        (status = 400, description = "The control can't do this", body = AppResponseError),
        (status = 404, description = "Monitor not found", body = AppResponseError)
    ),
    security(("jwt" = []))
)]
#[instrument(skip(state))]
pub async fn audio_stop(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<Json<PtzCommandResponse>> {
    let ptz_manager = state.ptz_manager();
    Ok(Json(
        service::ptz::audio(&state, ptz_manager, id, None).await?,
    ))
}

/// Turn the audio volume up
#[utoipa::path(
    post,
    path = "/api/v3/ptz/monitors/{id}/audio/volume/up",
    operation_id = "ptzAudioVolumeUp",
    tag = "PTZ",
    params(("id" = u32, Path, description = "Monitor ID")),
    responses(
        (status = 200, description = "Command executed", body = PtzCommandResponse),
        (status = 400, description = "The control can't do this", body = AppResponseError),
        (status = 404, description = "Monitor not found", body = AppResponseError)
    ),
    security(("jwt" = []))
)]
#[instrument(skip(state))]
pub async fn audio_volume_up(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<Json<PtzCommandResponse>> {
    let ptz_manager = state.ptz_manager();
    Ok(Json(
        service::ptz::audio_volume(&state, ptz_manager, id, true).await?,
    ))
}

/// Turn the audio volume down
#[utoipa::path(
    post,
    path = "/api/v3/ptz/monitors/{id}/audio/volume/down",
    operation_id = "ptzAudioVolumeDown",
    tag = "PTZ",
    params(("id" = u32, Path, description = "Monitor ID")),
    responses(
        (status = 200, description = "Command executed", body = PtzCommandResponse),
        (status = 400, description = "The control can't do this", body = AppResponseError),
        (status = 404, description = "Monitor not found", body = AppResponseError)
    ),
    security(("jwt" = []))
)]
#[instrument(skip(state))]
pub async fn audio_volume_down(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<Json<PtzCommandResponse>> {
    let ptz_manager = state.ptz_manager();
    Ok(Json(
        service::ptz::audio_volume(&state, ptz_manager, id, false).await?,
    ))
}

/// Play one of the device's sounds
///
/// - `file` must be within the control's `min_audio_file..=max_audio_file`.
#[utoipa::path(
    post,
    path = "/api/v3/ptz/monitors/{id}/audio/play/{file}",
    operation_id = "ptzAudioPlay",
    tag = "PTZ",
    params(
        ("id" = u32, Path, description = "Monitor ID"),
        ("file" = u32, Path, description = "Sound number, as the device numbers them")
    ),
    responses(
        (status = 200, description = "Command executed", body = PtzCommandResponse),
        (status = 400, description = "Out of range, or the control can't play audio", body = AppResponseError),
        (status = 404, description = "Monitor not found", body = AppResponseError)
    ),
    security(("jwt" = []))
)]
#[instrument(skip(state))]
pub async fn audio_play(
    Path((id, file)): Path<(u32, u32)>,
    State(state): State<AppState>,
) -> AppResult<Json<PtzCommandResponse>> {
    let ptz_manager = state.ptz_manager();
    Ok(Json(
        service::ptz::audio(&state, ptz_manager, id, Some(file)).await?,
    ))
}

/// The lights' current state, as the camera reports it
///
/// - `light` / `indicator_light` are `on`, `off`, or null when the control has
///   no such light or the control daemon gave no answer.
#[utoipa::path(
    get,
    path = "/api/v3/ptz/monitors/{id}/light",
    operation_id = "ptzLightStatus",
    tag = "PTZ",
    params(("id" = u32, Path, description = "Monitor ID")),
    responses(
        (status = 200, description = "Light state", body = crate::dto::response::ptz::LightStatusResponse),
        (status = 404, description = "Monitor not found", body = AppResponseError)
    ),
    security(("jwt" = []))
)]
#[instrument(skip(state))]
pub async fn light_status(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<Json<crate::dto::response::ptz::LightStatusResponse>> {
    let ptz_manager = state.ptz_manager();
    Ok(Json(
        service::ptz::light_status(&state, ptz_manager, id).await?,
    ))
}
