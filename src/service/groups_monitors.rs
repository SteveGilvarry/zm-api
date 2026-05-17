use crate::dto::request::groups_monitors::CreateGroupMonitorRequest;
use crate::dto::response::GroupMonitorResponse;
use crate::dto::{PaginatedResponse, PaginationParams};
use crate::error::{AppError, AppResult, Resource, ResourceType};
use crate::repo;
use crate::server::state::AppState;
use crate::service::monitor_acl::MonitorScope;
use crate::util::authz::Level;

fn group_monitor_not_found(id: u32) -> AppError {
    AppError::NotFoundError(Resource {
        details: vec![("id".into(), id.to_string())],
        resource_type: ResourceType::File,
    })
}

pub async fn list_paginated(
    state: &AppState,
    params: &PaginationParams,
    scope: &MonitorScope,
) -> AppResult<PaginatedResponse<GroupMonitorResponse>> {
    let filter = scope.visible_ids(Level::View);
    let (items, total) =
        repo::groups_monitors::find_paginated(state.db(), params, filter.as_deref()).await?;
    let responses: Vec<GroupMonitorResponse> =
        items.iter().map(GroupMonitorResponse::from).collect();
    Ok(PaginatedResponse::from_params(responses, total, params))
}

pub async fn list_all(
    state: &AppState,
    group_id: Option<u32>,
    monitor_id: Option<u32>,
    scope: &MonitorScope,
) -> AppResult<Vec<GroupMonitorResponse>> {
    let items = if let Some(gid) = group_id {
        repo::groups_monitors::find_by_group_id(state.db(), gid).await?
    } else if let Some(mid) = monitor_id {
        repo::groups_monitors::find_by_monitor_id(state.db(), mid).await?
    } else {
        repo::groups_monitors::find_all(state.db()).await?
    };
    // Row-level ACL: drop links to monitors outside the caller's scope.
    Ok(items
        .iter()
        .filter(|i| scope.allows(i.monitor_id, Level::View))
        .map(GroupMonitorResponse::from)
        .collect())
}

pub async fn get_by_id(
    state: &AppState,
    id: u32,
    scope: &MonitorScope,
) -> AppResult<GroupMonitorResponse> {
    let item = repo::groups_monitors::find_by_id(state.db(), id)
        .await?
        .ok_or_else(|| group_monitor_not_found(id))?;
    if !scope.allows(item.monitor_id, Level::View) {
        return Err(group_monitor_not_found(id));
    }
    Ok(GroupMonitorResponse::from(&item))
}

pub async fn create(
    state: &AppState,
    req: CreateGroupMonitorRequest,
    scope: &MonitorScope,
) -> AppResult<GroupMonitorResponse> {
    if !scope.allows(req.monitor_id, Level::Edit) {
        return Err(group_monitor_not_found(0));
    }
    let model = repo::groups_monitors::create(state.db(), &req).await?;
    Ok(GroupMonitorResponse::from(&model))
}

pub async fn delete(state: &AppState, id: u32, scope: &MonitorScope) -> AppResult<()> {
    let item = repo::groups_monitors::find_by_id(state.db(), id)
        .await?
        .ok_or_else(|| group_monitor_not_found(id))?;
    if !scope.allows(item.monitor_id, Level::Edit) {
        return Err(group_monitor_not_found(id));
    }
    let ok = repo::groups_monitors::delete_by_id(state.db(), id).await?;
    if ok {
        Ok(())
    } else {
        Err(group_monitor_not_found(id))
    }
}
