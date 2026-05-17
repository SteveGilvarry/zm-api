use crate::dto::request::monitor_status::UpdateMonitorStatusRequest;
use crate::dto::response::MonitorStatusResponse;
use crate::dto::{PaginatedResponse, PaginationParams};
use crate::error::{AppError, AppResult, Resource, ResourceType};
use crate::repo;
use crate::server::state::AppState;
use crate::service::monitor_acl::MonitorScope;
use crate::util::authz::Level;

fn status_not_found(monitor_id: u32) -> AppError {
    AppError::NotFoundError(Resource {
        details: vec![("monitor_id".into(), monitor_id.to_string())],
        resource_type: ResourceType::Monitor,
    })
}

pub async fn list_all(
    state: &AppState,
    scope: &MonitorScope,
) -> AppResult<Vec<MonitorStatusResponse>> {
    let filter = scope.visible_ids(Level::View);
    let items = repo::monitor_status::find_all(state.db(), filter.as_deref()).await?;
    Ok(items.iter().map(MonitorStatusResponse::from).collect())
}

pub async fn list_paginated(
    state: &AppState,
    params: &PaginationParams,
    scope: &MonitorScope,
) -> AppResult<PaginatedResponse<MonitorStatusResponse>> {
    let filter = scope.visible_ids(Level::View);
    let (items, total) =
        repo::monitor_status::find_paginated(state.db(), params, filter.as_deref()).await?;
    let responses: Vec<MonitorStatusResponse> =
        items.iter().map(MonitorStatusResponse::from).collect();
    Ok(PaginatedResponse::from_params(responses, total, params))
}

pub async fn get_by_monitor_id(
    state: &AppState,
    monitor_id: u32,
    scope: &MonitorScope,
) -> AppResult<MonitorStatusResponse> {
    if !scope.allows(monitor_id, Level::View) {
        return Err(status_not_found(monitor_id));
    }
    let item = repo::monitor_status::find_by_monitor_id(state.db(), monitor_id).await?;
    let item = item.ok_or_else(|| status_not_found(monitor_id))?;
    Ok(MonitorStatusResponse::from(&item))
}

pub async fn update(
    state: &AppState,
    monitor_id: u32,
    req: UpdateMonitorStatusRequest,
    scope: &MonitorScope,
) -> AppResult<MonitorStatusResponse> {
    if !scope.allows(monitor_id, Level::Edit) {
        return Err(status_not_found(monitor_id));
    }
    let updated = repo::monitor_status::update(state.db(), monitor_id, &req).await?;
    let updated = updated.ok_or_else(|| status_not_found(monitor_id))?;
    Ok(MonitorStatusResponse::from(&updated))
}
