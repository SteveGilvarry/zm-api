use crate::dto::request::stats::{CreateStatRequest, UpdateStatRequest};
use crate::dto::response::StatResponse;
use crate::error::{AppError, AppResult, Resource, ResourceType};
use crate::repo;
use crate::server::state::AppState;

pub async fn list_all(state: &AppState) -> AppResult<Vec<StatResponse>> {
    let items = repo::stats::find_all(state.db()).await?;
    Ok(items.iter().map(StatResponse::from).collect())
}

pub async fn get_by_id(state: &AppState, id: u32) -> AppResult<StatResponse> {
    let item = repo::stats::find_by_id(state.db(), id).await?;
    let item = item.ok_or_else(|| {
        AppError::NotFoundError(Resource {
            details: vec![("id".into(), id.to_string())],
            resource_type: ResourceType::Message,
        })
    })?;
    Ok(StatResponse::from(&item))
}

pub async fn create(state: &AppState, req: CreateStatRequest) -> AppResult<StatResponse> {
    let model = repo::stats::create(state.db(), &req).await?;
    Ok(StatResponse::from(&model))
}

pub async fn update(state: &AppState, id: u32, req: UpdateStatRequest) -> AppResult<StatResponse> {
    let updated = repo::stats::update(state.db(), id, &req).await?;
    let updated = updated.ok_or_else(|| {
        AppError::NotFoundError(Resource {
            details: vec![("id".into(), id.to_string())],
            resource_type: ResourceType::Message,
        })
    })?;
    Ok(StatResponse::from(&updated))
}

pub async fn delete(state: &AppState, id: u32) -> AppResult<()> {
    let ok = repo::stats::delete_by_id(state.db(), id).await?;
    if ok {
        Ok(())
    } else {
        Err(AppError::NotFoundError(Resource {
            details: vec![("id".into(), id.to_string())],
            resource_type: ResourceType::Message,
        }))
    }
}
