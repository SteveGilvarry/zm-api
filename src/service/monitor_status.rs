use crate::dto::response::MonitorStatusResponse;
use crate::dto::request::monitor_status::UpdateMonitorStatusRequest;
use crate::error::{AppError, AppResult, Resource, ResourceType};
use crate::repo;
use crate::server::state::AppState;

pub async fn list_all(state: &AppState) -> AppResult<Vec<MonitorStatusResponse>> {
    let items = repo::monitor_status::find_all(state.db()).await?;
    Ok(items.iter().map(MonitorStatusResponse::from).collect())
}

pub async fn get_by_monitor_id(state: &AppState, monitor_id: u32) -> AppResult<MonitorStatusResponse> {
    let item = repo::monitor_status::find_by_monitor_id(state.db(), monitor_id).await?;
    let item = item.ok_or_else(|| AppError::NotFoundError(Resource{
        details: vec![("monitor_id".into(), monitor_id.to_string())],
        resource_type: ResourceType::File
    }))?;
    Ok(MonitorStatusResponse::from(&item))
}

pub async fn update(state: &AppState, monitor_id: u32, req: UpdateMonitorStatusRequest) -> AppResult<MonitorStatusResponse> {
    let updated = repo::monitor_status::update(state.db(), monitor_id, &req).await?;
    let updated = updated.ok_or_else(|| AppError::NotFoundError(Resource{
        details: vec![("monitor_id".into(), monitor_id.to_string())],
        resource_type: ResourceType::File
    }))?;
    Ok(MonitorStatusResponse::from(&updated))
}
