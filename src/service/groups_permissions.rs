use crate::dto::response::GroupPermissionResponse;
use crate::dto::request::groups_permissions::{CreateGroupPermissionRequest, UpdateGroupPermissionRequest};
use crate::error::{AppError, AppResult, Resource, ResourceType};
use crate::repo;
use crate::server::state::AppState;

pub async fn list_all(state: &AppState, group_id: Option<u32>, user_id: Option<u32>) -> AppResult<Vec<GroupPermissionResponse>> {
    let items = if let Some(gid) = group_id {
        repo::groups_permissions::find_by_group_id(state.db(), gid).await?
    } else if let Some(uid) = user_id {
        repo::groups_permissions::find_by_user_id(state.db(), uid).await?
    } else {
        repo::groups_permissions::find_all(state.db()).await?
    };
    Ok(items.iter().map(GroupPermissionResponse::from).collect())
}

pub async fn get_by_id(state: &AppState, id: u32) -> AppResult<GroupPermissionResponse> {
    let item = repo::groups_permissions::find_by_id(state.db(), id).await?;
    let item = item.ok_or_else(|| AppError::NotFoundError(Resource{
        details: vec![("id".into(), id.to_string())],
        resource_type: ResourceType::File
    }))?;
    Ok(GroupPermissionResponse::from(&item))
}

pub async fn create(state: &AppState, req: CreateGroupPermissionRequest) -> AppResult<GroupPermissionResponse> {
    let model = repo::groups_permissions::create(state.db(), &req).await?;
    Ok(GroupPermissionResponse::from(&model))
}

pub async fn update(state: &AppState, id: u32, req: UpdateGroupPermissionRequest) -> AppResult<GroupPermissionResponse> {
    let updated = repo::groups_permissions::update(state.db(), id, &req).await?;
    let updated = updated.ok_or_else(|| AppError::NotFoundError(Resource{
        details: vec![("id".into(), id.to_string())],
        resource_type: ResourceType::File
    }))?;
    Ok(GroupPermissionResponse::from(&updated))
}

pub async fn delete(state: &AppState, id: u32) -> AppResult<()> {
    let ok = repo::groups_permissions::delete_by_id(state.db(), id).await?;
    if ok { Ok(()) } else {
        Err(AppError::NotFoundError(Resource{
            details: vec![("id".into(), id.to_string())],
            resource_type: ResourceType::File
        }))
    }
}
