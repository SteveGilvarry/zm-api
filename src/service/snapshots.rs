use crate::dto::response::SnapshotResponse;
use crate::dto::request::snapshots::{CreateSnapshotRequest, UpdateSnapshotRequest};
use crate::error::{AppError, AppResult, Resource, ResourceType};
use crate::repo;
use crate::server::state::AppState;

pub async fn list_all(state: &AppState) -> AppResult<Vec<SnapshotResponse>> {
    let items = repo::snapshots::find_all(state.db()).await?;
    Ok(items.iter().map(SnapshotResponse::from).collect())
}

pub async fn get_by_id(state: &AppState, id: u32) -> AppResult<SnapshotResponse> {
    let item = repo::snapshots::find_by_id(state.db(), id).await?;
    let item = item.ok_or_else(|| AppError::NotFoundError(Resource{
        details: vec![("id".into(), id.to_string())],
        resource_type: ResourceType::Message
    }))?;
    Ok(SnapshotResponse::from(&item))
}

pub async fn create(state: &AppState, req: CreateSnapshotRequest) -> AppResult<SnapshotResponse> {
    let model = repo::snapshots::create(state.db(), &req).await?;
    Ok(SnapshotResponse::from(&model))
}

pub async fn update(state: &AppState, id: u32, req: UpdateSnapshotRequest) -> AppResult<SnapshotResponse> {
    let updated = repo::snapshots::update(state.db(), id, &req).await?;
    let updated = updated.ok_or_else(|| AppError::NotFoundError(Resource{
        details: vec![("id".into(), id.to_string())],
        resource_type: ResourceType::Message
    }))?;
    Ok(SnapshotResponse::from(&updated))
}

pub async fn delete(state: &AppState, id: u32) -> AppResult<()> {
    let ok = repo::snapshots::delete_by_id(state.db(), id).await?;
    if ok { Ok(()) } else {
        Err(AppError::NotFoundError(Resource{
            details: vec![("id".into(), id.to_string())],
            resource_type: ResourceType::Message
        }))
    }
}
