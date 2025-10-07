use crate::dto::response::MonitorPermissionResponse;
use crate::dto::request::monitors_permissions::{CreateMonitorPermissionRequest, UpdateMonitorPermissionRequest};
use crate::error::{AppError, AppResult, Resource, ResourceType};
use crate::repo;
use crate::server::state::AppState;

pub async fn list_all(state: &AppState, monitor_id: Option<u32>, user_id: Option<u32>) -> AppResult<Vec<MonitorPermissionResponse>> {
    let items = if let Some(mid) = monitor_id {
        repo::monitors_permissions::find_by_monitor_id(state.db(), mid).await?
    } else if let Some(uid) = user_id {
        repo::monitors_permissions::find_by_user_id(state.db(), uid).await?
    } else {
        repo::monitors_permissions::find_all(state.db()).await?
    };
    Ok(items.iter().map(MonitorPermissionResponse::from).collect())
}

pub async fn get_by_id(state: &AppState, id: u32) -> AppResult<MonitorPermissionResponse> {
    let item = repo::monitors_permissions::find_by_id(state.db(), id).await?;
    let item = item.ok_or_else(|| AppError::NotFoundError(Resource{
        details: vec![("id".into(), id.to_string())],
        resource_type: ResourceType::File
    }))?;
    Ok(MonitorPermissionResponse::from(&item))
}

pub async fn create(state: &AppState, req: CreateMonitorPermissionRequest) -> AppResult<MonitorPermissionResponse> {
    let model = repo::monitors_permissions::create(state.db(), &req).await?;
    Ok(MonitorPermissionResponse::from(&model))
}

pub async fn update(state: &AppState, id: u32, req: UpdateMonitorPermissionRequest) -> AppResult<MonitorPermissionResponse> {
    let updated = repo::monitors_permissions::update(state.db(), id, &req).await?;
    let updated = updated.ok_or_else(|| AppError::NotFoundError(Resource{
        details: vec![("id".into(), id.to_string())],
        resource_type: ResourceType::File
    }))?;
    Ok(MonitorPermissionResponse::from(&updated))
}

pub async fn delete(state: &AppState, id: u32) -> AppResult<()> {
    let ok = repo::monitors_permissions::delete_by_id(state.db(), id).await?;
    if ok { Ok(()) } else {
        Err(AppError::NotFoundError(Resource{
            details: vec![("id".into(), id.to_string())],
            resource_type: ResourceType::File
        }))
    }
}
