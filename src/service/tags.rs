use crate::dto::response::TagResponse;
use crate::dto::request::tags::{CreateTagRequest, UpdateTagRequest};
use crate::error::{AppError, AppResult, Resource, ResourceType};
use crate::repo;
use crate::server::state::AppState;

pub async fn list_all(state: &AppState) -> AppResult<Vec<TagResponse>> {
    let items = repo::tags::find_all(state.db()).await?;
    Ok(items.iter().map(TagResponse::from).collect())
}

pub async fn get_by_id(state: &AppState, id: u64) -> AppResult<TagResponse> {
    let item = repo::tags::find_by_id(state.db(), id).await?;
    let item = item.ok_or_else(|| AppError::NotFoundError(Resource{
        details: vec![("id".into(), id.to_string())],
        resource_type: ResourceType::Message
    }))?;
    Ok(TagResponse::from(&item))
}

pub async fn create(state: &AppState, req: CreateTagRequest) -> AppResult<TagResponse> {
    let model = repo::tags::create(state.db(), &req).await?;
    Ok(TagResponse::from(&model))
}

pub async fn update(state: &AppState, id: u64, req: UpdateTagRequest) -> AppResult<TagResponse> {
    let updated = repo::tags::update(state.db(), id, &req).await?;
    let updated = updated.ok_or_else(|| AppError::NotFoundError(Resource{
        details: vec![("id".into(), id.to_string())],
        resource_type: ResourceType::Message
    }))?;
    Ok(TagResponse::from(&updated))
}

pub async fn delete(state: &AppState, id: u64) -> AppResult<()> {
    let ok = repo::tags::delete_by_id(state.db(), id).await?;
    if ok { Ok(()) } else {
        Err(AppError::NotFoundError(Resource{
            details: vec![("id".into(), id.to_string())],
            resource_type: ResourceType::Message
        }))
    }
}
