use crate::dto::response::GroupMonitorResponse;
use crate::dto::request::groups_monitors::CreateGroupMonitorRequest;
use crate::error::{AppError, AppResult, Resource, ResourceType};
use crate::repo;
use crate::server::state::AppState;

pub async fn list_all(state: &AppState, group_id: Option<u32>, monitor_id: Option<u32>) -> AppResult<Vec<GroupMonitorResponse>> {
    let items = if let Some(gid) = group_id {
        repo::groups_monitors::find_by_group_id(state.db(), gid).await?
    } else if let Some(mid) = monitor_id {
        repo::groups_monitors::find_by_monitor_id(state.db(), mid).await?
    } else {
        repo::groups_monitors::find_all(state.db()).await?
    };
    Ok(items.iter().map(GroupMonitorResponse::from).collect())
}

pub async fn get_by_id(state: &AppState, id: u32) -> AppResult<GroupMonitorResponse> {
    let item = repo::groups_monitors::find_by_id(state.db(), id).await?;
    let item = item.ok_or_else(|| AppError::NotFoundError(Resource{
        details: vec![("id".into(), id.to_string())],
        resource_type: ResourceType::File
    }))?;
    Ok(GroupMonitorResponse::from(&item))
}

pub async fn create(state: &AppState, req: CreateGroupMonitorRequest) -> AppResult<GroupMonitorResponse> {
    let model = repo::groups_monitors::create(state.db(), &req).await?;
    Ok(GroupMonitorResponse::from(&model))
}

pub async fn delete(state: &AppState, id: u32) -> AppResult<()> {
    let ok = repo::groups_monitors::delete_by_id(state.db(), id).await?;
    if ok { Ok(()) } else {
        Err(AppError::NotFoundError(Resource{
            details: vec![("id".into(), id.to_string())],
            resource_type: ResourceType::File
        }))
    }
}
