use crate::dto::request::monitor_presets::{
    CreateMonitorPresetRequest, UpdateMonitorPresetRequest,
};
use crate::dto::response::MonitorPresetResponse;
use crate::error::{AppError, AppResult, Resource, ResourceType};
use crate::repo;
use crate::server::state::AppState;

pub async fn list_all(
    state: &AppState,
    model_id: Option<u32>,
) -> AppResult<Vec<MonitorPresetResponse>> {
    let items = if let Some(mid) = model_id {
        repo::monitor_presets::find_by_model(state.db(), mid).await?
    } else {
        repo::monitor_presets::find_all(state.db()).await?
    };
    Ok(items.iter().map(MonitorPresetResponse::from).collect())
}

pub async fn get_by_id(state: &AppState, id: u32) -> AppResult<MonitorPresetResponse> {
    let item = repo::monitor_presets::find_by_id(state.db(), id).await?;
    let item = item.ok_or_else(|| {
        AppError::NotFoundError(Resource {
            details: vec![("id".into(), id.to_string())],
            resource_type: ResourceType::Message,
        })
    })?;
    Ok(MonitorPresetResponse::from(&item))
}

pub async fn create(
    state: &AppState,
    req: CreateMonitorPresetRequest,
) -> AppResult<MonitorPresetResponse> {
    let model = repo::monitor_presets::create(state.db(), &req).await?;
    Ok(MonitorPresetResponse::from(&model))
}

pub async fn update(
    state: &AppState,
    id: u32,
    req: UpdateMonitorPresetRequest,
) -> AppResult<MonitorPresetResponse> {
    let updated = repo::monitor_presets::update(state.db(), id, &req).await?;
    let updated = updated.ok_or_else(|| {
        AppError::NotFoundError(Resource {
            details: vec![("id".into(), id.to_string())],
            resource_type: ResourceType::Message,
        })
    })?;
    Ok(MonitorPresetResponse::from(&updated))
}

pub async fn delete(state: &AppState, id: u32) -> AppResult<()> {
    let ok = repo::monitor_presets::delete_by_id(state.db(), id).await?;
    if ok {
        Ok(())
    } else {
        Err(AppError::NotFoundError(Resource {
            details: vec![("id".into(), id.to_string())],
            resource_type: ResourceType::Message,
        }))
    }
}
