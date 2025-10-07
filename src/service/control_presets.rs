use crate::dto::response::ControlPresetResponse;
use crate::dto::request::control_presets::{CreateControlPresetRequest, UpdateControlPresetRequest};
use crate::error::{AppError, AppResult, Resource, ResourceType};
use crate::repo;
use crate::server::state::AppState;

pub async fn list_all(state: &AppState) -> AppResult<Vec<ControlPresetResponse>> {
    let items = repo::control_presets::find_all(state.db()).await?;
    Ok(items.iter().map(ControlPresetResponse::from).collect())
}

pub async fn list_by_monitor(state: &AppState, monitor_id: u32) -> AppResult<Vec<ControlPresetResponse>> {
    let items = repo::control_presets::find_by_monitor(state.db(), monitor_id).await?;
    Ok(items.iter().map(ControlPresetResponse::from).collect())
}

pub async fn get_by_id(state: &AppState, monitor_id: u32, preset: u32) -> AppResult<ControlPresetResponse> {
    let item = repo::control_presets::find_by_monitor_and_preset(state.db(), monitor_id, preset).await?;
    let item = item.ok_or_else(|| AppError::NotFoundError(Resource{
        details: vec![("monitor_id".into(), monitor_id.to_string()), ("preset".into(), preset.to_string())],
        resource_type: ResourceType::Message
    }))?;
    Ok(ControlPresetResponse::from(&item))
}

pub async fn create(state: &AppState, req: CreateControlPresetRequest) -> AppResult<ControlPresetResponse> {
    let model = repo::control_presets::create(state.db(), &req).await?;
    Ok(ControlPresetResponse::from(&model))
}

pub async fn update(state: &AppState, monitor_id: u32, preset: u32, req: UpdateControlPresetRequest) -> AppResult<ControlPresetResponse> {
    let updated = repo::control_presets::update(state.db(), monitor_id, preset, &req).await?;
    let updated = updated.ok_or_else(|| AppError::NotFoundError(Resource{
        details: vec![("monitor_id".into(), monitor_id.to_string()), ("preset".into(), preset.to_string())],
        resource_type: ResourceType::Message
    }))?;
    Ok(ControlPresetResponse::from(&updated))
}

pub async fn delete(state: &AppState, monitor_id: u32, preset: u32) -> AppResult<()> {
    let ok = repo::control_presets::delete_by_id(state.db(), monitor_id, preset).await?;
    if ok { Ok(()) } else {
        Err(AppError::NotFoundError(Resource{
            details: vec![("monitor_id".into(), monitor_id.to_string()), ("preset".into(), preset.to_string())],
            resource_type: ResourceType::Message
        }))
    }
}
