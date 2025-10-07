use crate::dto::response::ServerStatResponse;
use crate::dto::request::server_stats::CreateServerStatRequest;
use crate::error::{AppError, AppResult, Resource, ResourceType};
use crate::repo;
use crate::server::state::AppState;

pub async fn list_all(state: &AppState) -> AppResult<Vec<ServerStatResponse>> {
    let items = repo::server_stats::find_all(state.db()).await?;
    Ok(items.iter().map(ServerStatResponse::from).collect())
}

pub async fn get_by_id(state: &AppState, id: u32) -> AppResult<ServerStatResponse> {
    let item = repo::server_stats::find_by_id(state.db(), id).await?;
    let item = item.ok_or_else(|| AppError::NotFoundError(Resource{
        details: vec![("id".into(), id.to_string())],
        resource_type: ResourceType::File
    }))?;
    Ok(ServerStatResponse::from(&item))
}

pub async fn create(state: &AppState, req: CreateServerStatRequest) -> AppResult<ServerStatResponse> {
    let model = repo::server_stats::create(state.db(), &req).await?;
    Ok(ServerStatResponse::from(&model))
}

pub async fn delete(state: &AppState, id: u32) -> AppResult<()> {
    let ok = repo::server_stats::delete_by_id(state.db(), id).await?;
    if ok { Ok(()) } else {
        Err(AppError::NotFoundError(Resource{
            details: vec![("id".into(), id.to_string())],
            resource_type: ResourceType::File
        }))
    }
}
