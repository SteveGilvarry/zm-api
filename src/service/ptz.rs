//! PTZ service layer

use tracing::{info, instrument, warn};

use crate::dto::request::ptz::{
    PtzAbsoluteRequest, PtzFocusRequest, PtzMoveRequest, PtzPresetRequest, PtzRelativeRequest,
    PtzZoomRequest,
};
use crate::dto::response::ptz::{
    PtzCapabilitiesResponse, PtzCommandResponse, PtzProtocolInfo, PtzProtocolListResponse,
    PtzStatusResponse,
};
use crate::error::{AppError, AppResult, Resource, ResourceType};
use crate::ptz::capabilities::PtzCapabilities;
use crate::ptz::error::PtzError;
use crate::ptz::traits::{AbsolutePosition, MoveParams, PtzCommand, RelativePosition, ZoomParams};
use crate::ptz::PtzManager;
use crate::repo;
use crate::server::state::AppState;

fn ptz_to_app_error(e: PtzError) -> AppError {
    match e {
        PtzError::MonitorNotFound(id) => {
            let res = Resource {
                details: vec![("id".into(), id.to_string())],
                resource_type: ResourceType::Monitor,
            };
            AppError::NotFoundError(res)
        }
        PtzError::NoControlConfigured(id) => {
            AppError::BadRequestError(format!("Monitor {} has no PTZ control configured", id))
        }
        PtzError::ControlNotFound(id) => {
            let res = Resource {
                details: vec![("control_id".into(), id.to_string())],
                resource_type: ResourceType::Config,
            };
            AppError::NotFoundError(res)
        }
        PtzError::CommandNotSupported(msg) => {
            AppError::BadRequestError(format!("Command not supported: {}", msg))
        }
        PtzError::AuthenticationFailed(msg) => AppError::UnauthorizedError(msg),
        PtzError::CameraOffline(msg) => {
            AppError::ServiceUnavailableError(format!("Camera offline: {}", msg))
        }
        PtzError::CommandTimeout(msg) => {
            AppError::ServiceUnavailableError(format!("Command timeout: {}", msg))
        }
        PtzError::InvalidParameter(msg) => {
            AppError::BadRequestError(format!("Invalid parameter: {}", msg))
        }
        PtzError::ProtocolError(msg) => {
            AppError::InternalServerError(format!("Protocol error: {}", msg))
        }
        PtzError::PerlBridgeError(msg) => {
            AppError::InternalServerError(format!("Perl bridge error: {}", msg))
        }
        PtzError::InternalError(msg) => AppError::InternalServerError(msg),
    }
}

#[instrument(skip(state, ptz_manager))]
pub async fn get_status(
    state: &AppState,
    ptz_manager: &PtzManager,
    monitor_id: u32,
) -> AppResult<PtzStatusResponse> {
    let result = repo::ptz::get_monitor_with_control(state.db(), monitor_id).await?;
    let (monitor, control) = result.ok_or_else(|| {
        AppError::NotFoundError(Resource {
            details: vec![("id".into(), monitor_id.to_string())],
            resource_type: ResourceType::Monitor,
        })
    })?;

    let Some(control) = control else {
        return Ok(PtzStatusResponse {
            monitor_id,
            available: false,
            protocol: None,
            is_native: false,
            capabilities: PtzCapabilities::default(),
            position: None,
        });
    };

    let capabilities = PtzCapabilities::from(&control);
    let protocol = control.protocol.clone();
    let is_native = protocol
        .as_ref()
        .map(|p| ptz_manager.is_native_protocol(p))
        .unwrap_or(false);

    if let Err(e) = ptz_manager
        .create_and_cache_for_models(&monitor, &control)
        .await
    {
        warn!(monitor_id, error = %e, "Failed to initialize PTZ control");
    }

    Ok(PtzStatusResponse {
        monitor_id,
        available: capabilities.has_any_ptz(),
        protocol,
        is_native,
        capabilities,
        position: None,
    })
}

#[instrument(skip(state))]
pub async fn get_capabilities(
    state: &AppState,
    monitor_id: u32,
) -> AppResult<PtzCapabilitiesResponse> {
    let result = repo::ptz::get_monitor_with_control(state.db(), monitor_id).await?;
    let (_monitor, control) = result.ok_or_else(|| {
        AppError::NotFoundError(Resource {
            details: vec![("id".into(), monitor_id.to_string())],
            resource_type: ResourceType::Monitor,
        })
    })?;

    let control = control.ok_or_else(|| {
        AppError::BadRequestError(format!(
            "Monitor {} has no PTZ control configured",
            monitor_id
        ))
    })?;

    let capabilities = PtzCapabilities::from(&control);

    Ok(PtzCapabilitiesResponse {
        monitor_id,
        control_id: control.id,
        name: control.name.clone(),
        protocol: control.protocol.clone(),
        capabilities,
    })
}

pub fn list_protocols(ptz_manager: &PtzManager) -> PtzProtocolListResponse {
    let protocols = ptz_manager.list_protocols();
    let native_protocols: Vec<String> = protocols
        .iter()
        .filter(|p| p.is_native)
        .map(|p| p.name.clone())
        .collect();

    PtzProtocolListResponse {
        protocols: protocols.into_iter().map(PtzProtocolInfo::from).collect(),
        native_protocols,
        perl_fallback_enabled: true,
    }
}

#[instrument(skip(state, ptz_manager))]
pub async fn move_direction(
    state: &AppState,
    ptz_manager: &PtzManager,
    monitor_id: u32,
    direction: &str,
    request: PtzMoveRequest,
) -> AppResult<PtzCommandResponse> {
    let (monitor, control) = get_monitor_and_control(state, monitor_id).await?;
    let params = MoveParams::from(request);
    let command = match direction {
        "up" => PtzCommand::MoveUp(params),
        "down" => PtzCommand::MoveDown(params),
        "left" => PtzCommand::MoveLeft(params),
        "right" => PtzCommand::MoveRight(params),
        "up-left" | "upleft" => PtzCommand::MoveUpLeft(params),
        "up-right" | "upright" => PtzCommand::MoveUpRight(params),
        "down-left" | "downleft" => PtzCommand::MoveDownLeft(params),
        "down-right" | "downright" => PtzCommand::MoveDownRight(params),
        _ => {
            return Err(AppError::BadRequestError(format!(
                "Invalid direction: {}",
                direction
            )))
        }
    };
    execute_command(ptz_manager, &monitor, &control, command).await
}

#[instrument(skip(state, ptz_manager))]
pub async fn move_stop(
    state: &AppState,
    ptz_manager: &PtzManager,
    monitor_id: u32,
) -> AppResult<PtzCommandResponse> {
    let (monitor, control) = get_monitor_and_control(state, monitor_id).await?;
    execute_command(ptz_manager, &monitor, &control, PtzCommand::MoveStop).await
}

#[instrument(skip(state, ptz_manager))]
pub async fn zoom(
    state: &AppState,
    ptz_manager: &PtzManager,
    monitor_id: u32,
    direction: &str,
    request: PtzZoomRequest,
) -> AppResult<PtzCommandResponse> {
    let (monitor, control) = get_monitor_and_control(state, monitor_id).await?;
    let params = ZoomParams::from(request);
    let command = match direction {
        "in" => PtzCommand::ZoomIn(params),
        "out" => PtzCommand::ZoomOut(params),
        _ => {
            return Err(AppError::BadRequestError(format!(
                "Invalid zoom direction: {}",
                direction
            )))
        }
    };
    execute_command(ptz_manager, &monitor, &control, command).await
}

#[instrument(skip(state, ptz_manager))]
pub async fn zoom_stop(
    state: &AppState,
    ptz_manager: &PtzManager,
    monitor_id: u32,
) -> AppResult<PtzCommandResponse> {
    let (monitor, control) = get_monitor_and_control(state, monitor_id).await?;
    execute_command(ptz_manager, &monitor, &control, PtzCommand::ZoomStop).await
}

#[instrument(skip(state, ptz_manager))]
pub async fn focus(
    state: &AppState,
    ptz_manager: &PtzManager,
    monitor_id: u32,
    direction: &str,
    request: PtzFocusRequest,
) -> AppResult<PtzCommandResponse> {
    let (monitor, control) = get_monitor_and_control(state, monitor_id).await?;
    let command = match direction {
        "near" => PtzCommand::FocusNear(request.into()),
        "far" => PtzCommand::FocusFar(request.into()),
        "auto" => PtzCommand::FocusAuto,
        _ => {
            return Err(AppError::BadRequestError(format!(
                "Invalid focus direction: {}",
                direction
            )))
        }
    };
    execute_command(ptz_manager, &monitor, &control, command).await
}

#[instrument(skip(state, ptz_manager))]
pub async fn focus_stop(
    state: &AppState,
    ptz_manager: &PtzManager,
    monitor_id: u32,
) -> AppResult<PtzCommandResponse> {
    let (monitor, control) = get_monitor_and_control(state, monitor_id).await?;
    execute_command(ptz_manager, &monitor, &control, PtzCommand::FocusStop).await
}

#[instrument(skip(state, ptz_manager))]
pub async fn goto_preset(
    state: &AppState,
    ptz_manager: &PtzManager,
    monitor_id: u32,
    preset_id: u32,
) -> AppResult<PtzCommandResponse> {
    let (monitor, control) = get_monitor_and_control(state, monitor_id).await?;
    execute_command(
        ptz_manager,
        &monitor,
        &control,
        PtzCommand::GotoPreset { preset_id },
    )
    .await
}

#[instrument(skip(state, ptz_manager))]
pub async fn set_preset(
    state: &AppState,
    ptz_manager: &PtzManager,
    monitor_id: u32,
    preset_id: u32,
    request: PtzPresetRequest,
) -> AppResult<PtzCommandResponse> {
    let (monitor, control) = get_monitor_and_control(state, monitor_id).await?;
    let command = PtzCommand::SetPreset {
        preset_id,
        name: request.name,
    };
    execute_command(ptz_manager, &monitor, &control, command).await
}

#[instrument(skip(state, ptz_manager))]
pub async fn clear_preset(
    state: &AppState,
    ptz_manager: &PtzManager,
    monitor_id: u32,
    preset_id: u32,
) -> AppResult<PtzCommandResponse> {
    let (monitor, control) = get_monitor_and_control(state, monitor_id).await?;
    execute_command(
        ptz_manager,
        &monitor,
        &control,
        PtzCommand::ClearPreset { preset_id },
    )
    .await
}

#[instrument(skip(state, ptz_manager))]
pub async fn goto_home(
    state: &AppState,
    ptz_manager: &PtzManager,
    monitor_id: u32,
) -> AppResult<PtzCommandResponse> {
    let (monitor, control) = get_monitor_and_control(state, monitor_id).await?;
    execute_command(ptz_manager, &monitor, &control, PtzCommand::GotoHome).await
}

#[instrument(skip(state, ptz_manager))]
pub async fn move_absolute(
    state: &AppState,
    ptz_manager: &PtzManager,
    monitor_id: u32,
    request: PtzAbsoluteRequest,
) -> AppResult<PtzCommandResponse> {
    let (monitor, control) = get_monitor_and_control(state, monitor_id).await?;
    let pos = AbsolutePosition {
        pan: request.pan,
        tilt: request.tilt,
        zoom: request.zoom,
    };
    execute_command(
        ptz_manager,
        &monitor,
        &control,
        PtzCommand::MoveAbsolute(pos),
    )
    .await
}

#[instrument(skip(state, ptz_manager))]
pub async fn move_relative(
    state: &AppState,
    ptz_manager: &PtzManager,
    monitor_id: u32,
    request: PtzRelativeRequest,
) -> AppResult<PtzCommandResponse> {
    let (monitor, control) = get_monitor_and_control(state, monitor_id).await?;
    let pos = RelativePosition {
        pan_delta: request.pan_delta,
        tilt_delta: request.tilt_delta,
        zoom_delta: request.zoom_delta,
    };
    execute_command(
        ptz_manager,
        &monitor,
        &control,
        PtzCommand::MoveRelative(pos),
    )
    .await
}

async fn get_monitor_and_control(
    state: &AppState,
    monitor_id: u32,
) -> AppResult<(
    crate::entity::monitors::Model,
    crate::entity::controls::Model,
)> {
    let result = repo::ptz::get_monitor_with_control(state.db(), monitor_id).await?;
    let (monitor, control) = result.ok_or_else(|| {
        AppError::NotFoundError(Resource {
            details: vec![("id".into(), monitor_id.to_string())],
            resource_type: ResourceType::Monitor,
        })
    })?;

    let control = control.ok_or_else(|| {
        AppError::BadRequestError(format!(
            "Monitor {} has no PTZ control configured",
            monitor_id
        ))
    })?;

    Ok((monitor, control))
}

async fn execute_command(
    ptz_manager: &PtzManager,
    monitor: &crate::entity::monitors::Model,
    control: &crate::entity::controls::Model,
    command: PtzCommand,
) -> AppResult<PtzCommandResponse> {
    let result = ptz_manager
        .execute_with_models(monitor, control, command)
        .await
        .map_err(ptz_to_app_error)?;

    info!(
        monitor_id = monitor.id,
        success = result.success,
        "PTZ command executed"
    );

    Ok(PtzCommandResponse {
        success: result.success,
        message: result.message,
    })
}
