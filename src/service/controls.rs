use crate::dto::request::controls::{CreateControlRequest, UpdateControlRequest};
use crate::dto::response::ControlResponse;
use crate::dto::{PaginatedResponse, PaginationParams};
use crate::error::{AppError, AppResult, Resource, ResourceType};
use crate::repo;
use crate::server::state::AppState;

pub async fn list_all(state: &AppState) -> AppResult<Vec<ControlResponse>> {
    let items = repo::controls::find_all(state.db()).await?;
    Ok(items.iter().map(ControlResponse::from).collect())
}

pub async fn list_paginated(
    state: &AppState,
    params: &PaginationParams,
) -> AppResult<PaginatedResponse<ControlResponse>> {
    let (items, total) = repo::controls::find_paginated(state.db(), params).await?;
    let responses: Vec<ControlResponse> = items.iter().map(ControlResponse::from).collect();
    Ok(PaginatedResponse::from_params(responses, total, params))
}

pub async fn get_by_id(state: &AppState, id: u32) -> AppResult<ControlResponse> {
    let item = repo::controls::find_by_id(state.db(), id).await?;
    let item = item.ok_or_else(|| {
        AppError::NotFoundError(Resource {
            details: vec![("id".into(), id.to_string())],
            resource_type: ResourceType::Message,
        })
    })?;
    Ok(ControlResponse::from(&item))
}

pub async fn create(state: &AppState, req: CreateControlRequest) -> AppResult<ControlResponse> {
    let model = repo::controls::create(state.db(), &req).await?;
    Ok(ControlResponse::from(&model))
}

pub async fn update(
    state: &AppState,
    id: u32,
    req: UpdateControlRequest,
) -> AppResult<ControlResponse> {
    let updated = repo::controls::update(state.db(), id, &req).await?;
    let updated = updated.ok_or_else(|| {
        AppError::NotFoundError(Resource {
            details: vec![("id".into(), id.to_string())],
            resource_type: ResourceType::Message,
        })
    })?;
    Ok(ControlResponse::from(&updated))
}

pub async fn delete(state: &AppState, id: u32) -> AppResult<()> {
    let ok = repo::controls::delete_by_id(state.db(), id).await?;
    if ok {
        Ok(())
    } else {
        Err(AppError::NotFoundError(Resource {
            details: vec![("id".into(), id.to_string())],
            resource_type: ResourceType::Message,
        }))
    }
}
