use crate::dto::request::object_types::{CreateObjectTypeRequest, UpdateObjectTypeRequest};
use crate::dto::response::ObjectTypeResponse;
use crate::dto::{PaginatedResponse, PaginationParams};
use crate::error::{AppError, AppResult, Resource, ResourceType};
use crate::repo;
use crate::server::state::AppState;

pub async fn list_all(state: &AppState) -> AppResult<Vec<ObjectTypeResponse>> {
    let items = repo::object_types::find_all(state.db()).await?;
    Ok(items.iter().map(ObjectTypeResponse::from).collect())
}

pub async fn list_paginated(
    state: &AppState,
    params: &PaginationParams,
) -> AppResult<PaginatedResponse<ObjectTypeResponse>> {
    let (items, total) = repo::object_types::find_paginated(state.db(), params).await?;
    let responses: Vec<ObjectTypeResponse> = items.iter().map(ObjectTypeResponse::from).collect();
    Ok(PaginatedResponse::from_params(responses, total, params))
}

pub async fn get_by_id(state: &AppState, id: i32) -> AppResult<ObjectTypeResponse> {
    let item = repo::object_types::find_by_id(state.db(), id).await?;
    let item = item.ok_or_else(|| {
        AppError::NotFoundError(Resource {
            details: vec![("id".into(), id.to_string())],
            resource_type: ResourceType::File,
        })
    })?;
    Ok(ObjectTypeResponse::from(&item))
}

pub async fn create(
    state: &AppState,
    req: CreateObjectTypeRequest,
) -> AppResult<ObjectTypeResponse> {
    let model = repo::object_types::create(state.db(), &req).await?;
    Ok(ObjectTypeResponse::from(&model))
}

pub async fn update(
    state: &AppState,
    id: i32,
    req: UpdateObjectTypeRequest,
) -> AppResult<ObjectTypeResponse> {
    let updated = repo::object_types::update(state.db(), id, &req).await?;
    let updated = updated.ok_or_else(|| {
        AppError::NotFoundError(Resource {
            details: vec![("id".into(), id.to_string())],
            resource_type: ResourceType::File,
        })
    })?;
    Ok(ObjectTypeResponse::from(&updated))
}

pub async fn delete(state: &AppState, id: i32) -> AppResult<()> {
    let ok = repo::object_types::delete_by_id(state.db(), id).await?;
    if ok {
        Ok(())
    } else {
        Err(AppError::NotFoundError(Resource {
            details: vec![("id".into(), id.to_string())],
            resource_type: ResourceType::File,
        }))
    }
}
