use crate::dto::request::sessions::{CreateSessionRequest, UpdateSessionRequest};
use crate::dto::response::SessionResponse;
use crate::dto::{PaginatedResponse, PaginationParams};
use crate::error::{AppError, AppResult, Resource, ResourceType};
use crate::repo;
use crate::server::state::AppState;

pub async fn list_all(state: &AppState) -> AppResult<Vec<SessionResponse>> {
    let items = repo::sessions::find_all(state.db()).await?;
    Ok(items.iter().map(SessionResponse::from).collect())
}

pub async fn list_paginated(
    state: &AppState,
    params: &PaginationParams,
) -> AppResult<PaginatedResponse<SessionResponse>> {
    let (items, total) = repo::sessions::find_paginated(state.db(), params).await?;
    let responses: Vec<SessionResponse> = items.iter().map(SessionResponse::from).collect();
    Ok(PaginatedResponse::from_params(responses, total, params))
}

pub async fn get_by_id(state: &AppState, id: &str) -> AppResult<SessionResponse> {
    let item = repo::sessions::find_by_id(state.db(), id).await?;
    let item = item.ok_or_else(|| {
        AppError::NotFoundError(Resource {
            details: vec![("id".into(), id.to_string())],
            resource_type: ResourceType::Message,
        })
    })?;
    Ok(SessionResponse::from(&item))
}

pub async fn create(state: &AppState, req: CreateSessionRequest) -> AppResult<SessionResponse> {
    let model = repo::sessions::create(state.db(), &req).await?;
    Ok(SessionResponse::from(&model))
}

pub async fn update(
    state: &AppState,
    id: &str,
    req: UpdateSessionRequest,
) -> AppResult<SessionResponse> {
    let updated = repo::sessions::update(state.db(), id, &req).await?;
    let updated = updated.ok_or_else(|| {
        AppError::NotFoundError(Resource {
            details: vec![("id".into(), id.to_string())],
            resource_type: ResourceType::Message,
        })
    })?;
    Ok(SessionResponse::from(&updated))
}

pub async fn delete(state: &AppState, id: &str) -> AppResult<()> {
    let ok = repo::sessions::delete_by_id(state.db(), id).await?;
    if ok {
        Ok(())
    } else {
        Err(AppError::NotFoundError(Resource {
            details: vec![("id".into(), id.to_string())],
            resource_type: ResourceType::Message,
        }))
    }
}
