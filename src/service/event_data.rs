use crate::dto::request::event_data::{CreateEventDataRequest, UpdateEventDataRequest};
use crate::dto::response::EventDataResponse;
use crate::dto::{PaginatedResponse, PaginationParams};
use crate::error::{AppError, AppResult, Resource, ResourceType};
use crate::repo;
use crate::server::state::AppState;

pub async fn list_all(
    state: &AppState,
    event_id: Option<u64>,
) -> AppResult<Vec<EventDataResponse>> {
    let items = repo::event_data::find_all(state.db(), event_id).await?;
    Ok(items.iter().map(EventDataResponse::from).collect())
}

pub async fn list_paginated(
    state: &AppState,
    params: &PaginationParams,
    event_id: Option<u64>,
) -> AppResult<PaginatedResponse<EventDataResponse>> {
    let (items, total) = repo::event_data::find_paginated(state.db(), params, event_id).await?;
    let responses: Vec<EventDataResponse> = items.iter().map(EventDataResponse::from).collect();
    Ok(PaginatedResponse::from_params(responses, total, params))
}

pub async fn get_by_id(state: &AppState, id: u64) -> AppResult<EventDataResponse> {
    let item = repo::event_data::find_by_id(state.db(), id).await?;
    let item = item.ok_or_else(|| {
        AppError::NotFoundError(Resource {
            details: vec![("id".into(), id.to_string())],
            resource_type: ResourceType::File,
        })
    })?;
    Ok(EventDataResponse::from(&item))
}

pub async fn create(state: &AppState, req: CreateEventDataRequest) -> AppResult<EventDataResponse> {
    let model = repo::event_data::create(state.db(), &req).await?;
    Ok(EventDataResponse::from(&model))
}

pub async fn update(
    state: &AppState,
    id: u64,
    req: UpdateEventDataRequest,
) -> AppResult<EventDataResponse> {
    let updated = repo::event_data::update(state.db(), id, &req).await?;
    let updated = updated.ok_or_else(|| {
        AppError::NotFoundError(Resource {
            details: vec![("id".into(), id.to_string())],
            resource_type: ResourceType::File,
        })
    })?;
    Ok(EventDataResponse::from(&updated))
}

pub async fn delete(state: &AppState, id: u64) -> AppResult<()> {
    let ok = repo::event_data::delete_by_id(state.db(), id).await?;
    if ok {
        Ok(())
    } else {
        Err(AppError::NotFoundError(Resource {
            details: vec![("id".into(), id.to_string())],
            resource_type: ResourceType::File,
        }))
    }
}
