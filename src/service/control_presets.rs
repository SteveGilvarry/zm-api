use crate::dto::request::control_presets::{
    CreateControlPresetRequest, UpdateControlPresetRequest,
};
use crate::dto::response::ControlPresetResponse;
use crate::dto::{PaginatedResponse, PaginationParams};
use crate::error::{AppError, AppResult, Resource, ResourceType};
use crate::repo;
use crate::server::state::AppState;
use crate::service::monitor_acl::MonitorScope;
use crate::util::authz::Level;

fn preset_not_found(monitor_id: u32, preset: u32) -> AppError {
    AppError::NotFoundError(Resource {
        details: vec![
            ("monitor_id".into(), monitor_id.to_string()),
            ("preset".into(), preset.to_string()),
        ],
        resource_type: ResourceType::Message,
    })
}

pub async fn list_all(
    state: &AppState,
    scope: &MonitorScope,
) -> AppResult<Vec<ControlPresetResponse>> {
    let filter = scope.visible_ids(Level::View);
    let items = repo::control_presets::find_all(state.db(), filter.as_deref()).await?;
    Ok(items.iter().map(ControlPresetResponse::from).collect())
}

pub async fn list_paginated(
    state: &AppState,
    params: &PaginationParams,
    scope: &MonitorScope,
) -> AppResult<PaginatedResponse<ControlPresetResponse>> {
    let filter = scope.visible_ids(Level::View);
    let (items, total) =
        repo::control_presets::find_paginated(state.db(), params, filter.as_deref()).await?;
    let responses: Vec<ControlPresetResponse> =
        items.iter().map(ControlPresetResponse::from).collect();
    Ok(PaginatedResponse::from_params(responses, total, params))
}

pub async fn list_by_monitor(
    state: &AppState,
    monitor_id: u32,
    scope: &MonitorScope,
) -> AppResult<Vec<ControlPresetResponse>> {
    if !scope.allows(monitor_id, Level::View) {
        return Err(preset_not_found(monitor_id, 0));
    }
    let items = repo::control_presets::find_by_monitor(state.db(), monitor_id).await?;
    Ok(items.iter().map(ControlPresetResponse::from).collect())
}

pub async fn get_by_id(
    state: &AppState,
    monitor_id: u32,
    preset: u32,
    scope: &MonitorScope,
) -> AppResult<ControlPresetResponse> {
    if !scope.allows(monitor_id, Level::View) {
        return Err(preset_not_found(monitor_id, preset));
    }
    let item = repo::control_presets::find_by_monitor_and_preset(state.db(), monitor_id, preset)
        .await?
        .ok_or_else(|| preset_not_found(monitor_id, preset))?;
    Ok(ControlPresetResponse::from(&item))
}

pub async fn create(
    state: &AppState,
    req: CreateControlPresetRequest,
    scope: &MonitorScope,
) -> AppResult<ControlPresetResponse> {
    if !scope.allows(req.monitor_id, Level::Edit) {
        return Err(preset_not_found(req.monitor_id, req.preset));
    }
    let model = repo::control_presets::create(state.db(), &req).await?;
    Ok(ControlPresetResponse::from(&model))
}

pub async fn update(
    state: &AppState,
    monitor_id: u32,
    preset: u32,
    req: UpdateControlPresetRequest,
    scope: &MonitorScope,
) -> AppResult<ControlPresetResponse> {
    if !scope.allows(monitor_id, Level::Edit) {
        return Err(preset_not_found(monitor_id, preset));
    }
    let updated = repo::control_presets::update(state.db(), monitor_id, preset, &req)
        .await?
        .ok_or_else(|| preset_not_found(monitor_id, preset))?;
    Ok(ControlPresetResponse::from(&updated))
}

pub async fn delete(
    state: &AppState,
    monitor_id: u32,
    preset: u32,
    scope: &MonitorScope,
) -> AppResult<()> {
    if !scope.allows(monitor_id, Level::Edit) {
        return Err(preset_not_found(monitor_id, preset));
    }
    let ok = repo::control_presets::delete_by_id(state.db(), monitor_id, preset).await?;
    if ok {
        Ok(())
    } else {
        Err(preset_not_found(monitor_id, preset))
    }
}
