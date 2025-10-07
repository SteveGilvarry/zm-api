use crate::dto::response::StateResponse;
use crate::dto::request::states::{CreateStateRequest, UpdateStateRequest};
use crate::error::{AppError, AppResult, Resource, ResourceType};
use crate::repo;
use crate::server::state::AppState;

pub async fn list_all(state: &AppState) -> AppResult<Vec<StateResponse>> {
    let items = repo::states::find_all(state.db()).await?;
    Ok(items.iter().map(StateResponse::from).collect())
}

pub async fn get_by_id(state: &AppState, id: u32) -> AppResult<StateResponse> {
    let item = repo::states::find_by_id(state.db(), id).await?;
    let item = item.ok_or_else(|| AppError::NotFoundError(Resource{
        details: vec![("id".into(), id.to_string())],
        resource_type: ResourceType::Message
    }))?;
    Ok(StateResponse::from(&item))
}

pub async fn create(state: &AppState, req: CreateStateRequest) -> AppResult<StateResponse> {
    let model = repo::states::create(state.db(), &req).await?;
    Ok(StateResponse::from(&model))
}

pub async fn update(state: &AppState, id: u32, req: UpdateStateRequest) -> AppResult<StateResponse> {
    let updated = repo::states::update(state.db(), id, &req).await?;
    let updated = updated.ok_or_else(|| AppError::NotFoundError(Resource{
        details: vec![("id".into(), id.to_string())],
        resource_type: ResourceType::Message
    }))?;
    Ok(StateResponse::from(&updated))
}

pub async fn delete(state: &AppState, id: u32) -> AppResult<()> {
    let ok = repo::states::delete_by_id(state.db(), id).await?;
    if ok { Ok(()) } else {
        Err(AppError::NotFoundError(Resource{
            details: vec![("id".into(), id.to_string())],
            resource_type: ResourceType::Message
        }))
    }
}
