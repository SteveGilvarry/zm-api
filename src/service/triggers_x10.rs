use crate::dto::request::triggers_x10::{CreateTriggerX10Request, UpdateTriggerX10Request};
use crate::dto::response::TriggerX10Response;
use crate::dto::{PaginatedResponse, PaginationParams};
use crate::error::{AppError, AppResult, Resource, ResourceType};
use crate::repo;
use crate::server::state::AppState;

pub async fn list_all(state: &AppState) -> AppResult<Vec<TriggerX10Response>> {
    let items = repo::triggers_x10::find_all(state.db()).await?;
    Ok(items.iter().map(TriggerX10Response::from).collect())
}

pub async fn list_paginated(
    state: &AppState,
    params: &PaginationParams,
) -> AppResult<PaginatedResponse<TriggerX10Response>> {
    let (items, total) = repo::triggers_x10::find_paginated(state.db(), params).await?;
    let responses: Vec<TriggerX10Response> = items.iter().map(TriggerX10Response::from).collect();
    Ok(PaginatedResponse::from_params(responses, total, params))
}

pub async fn get_by_id(state: &AppState, monitor_id: u32) -> AppResult<TriggerX10Response> {
    let item = repo::triggers_x10::find_by_id(state.db(), monitor_id).await?;
    let item = item.ok_or_else(|| {
        AppError::NotFoundError(Resource {
            details: vec![("monitor_id".into(), monitor_id.to_string())],
            resource_type: ResourceType::Message,
        })
    })?;
    Ok(TriggerX10Response::from(&item))
}

pub async fn create(
    state: &AppState,
    req: CreateTriggerX10Request,
) -> AppResult<TriggerX10Response> {
    let model = repo::triggers_x10::create(state.db(), &req).await?;
    Ok(TriggerX10Response::from(&model))
}

pub async fn update(
    state: &AppState,
    monitor_id: u32,
    req: UpdateTriggerX10Request,
) -> AppResult<TriggerX10Response> {
    let updated = repo::triggers_x10::update(state.db(), monitor_id, &req).await?;
    let updated = updated.ok_or_else(|| {
        AppError::NotFoundError(Resource {
            details: vec![("monitor_id".into(), monitor_id.to_string())],
            resource_type: ResourceType::Message,
        })
    })?;
    Ok(TriggerX10Response::from(&updated))
}

pub async fn delete(state: &AppState, monitor_id: u32) -> AppResult<()> {
    let ok = repo::triggers_x10::delete_by_id(state.db(), monitor_id).await?;
    if ok {
        Ok(())
    } else {
        Err(AppError::NotFoundError(Resource {
            details: vec![("monitor_id".into(), monitor_id.to_string())],
            resource_type: ResourceType::Message,
        }))
    }
}
