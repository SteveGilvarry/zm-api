use crate::dto::request::event_data::{CreateEventDataRequest, UpdateEventDataRequest};
use crate::dto::response::EventDataResponse;
use crate::dto::{PaginatedResponse, PaginationParams};
use crate::error::{AppError, AppResult, Resource, ResourceType};
use crate::repo;
use crate::server::state::AppState;
use crate::service::monitor_acl::MonitorScope;
use crate::util::authz::Level;

fn event_data_not_found(id: u64) -> AppError {
    AppError::NotFoundError(Resource {
        details: vec![("id".into(), id.to_string())],
        resource_type: ResourceType::File,
    })
}

pub async fn list_all(
    state: &AppState,
    event_id: Option<u64>,
    scope: &MonitorScope,
) -> AppResult<Vec<EventDataResponse>> {
    let filter = scope.visible_ids(Level::View);
    let items = repo::event_data::find_all(state.db(), event_id, filter.as_deref()).await?;
    Ok(items.iter().map(EventDataResponse::from).collect())
}

pub async fn list_paginated(
    state: &AppState,
    params: &PaginationParams,
    event_id: Option<u64>,
    scope: &MonitorScope,
) -> AppResult<PaginatedResponse<EventDataResponse>> {
    let filter = scope.visible_ids(Level::View);
    let (items, total) =
        repo::event_data::find_paginated(state.db(), params, event_id, filter.as_deref()).await?;
    let responses: Vec<EventDataResponse> = items.iter().map(EventDataResponse::from).collect();
    Ok(PaginatedResponse::from_params(responses, total, params))
}

pub async fn get_by_id(
    state: &AppState,
    id: u64,
    scope: &MonitorScope,
) -> AppResult<EventDataResponse> {
    let item = repo::event_data::find_by_id(state.db(), id)
        .await?
        .ok_or_else(|| event_data_not_found(id))?;
    if !scope.allows_opt(item.monitor_id, Level::View) {
        return Err(event_data_not_found(id));
    }
    Ok(EventDataResponse::from(&item))
}

pub async fn create(
    state: &AppState,
    req: CreateEventDataRequest,
    scope: &MonitorScope,
) -> AppResult<EventDataResponse> {
    if !scope.allows_opt(req.monitor_id, Level::Edit) {
        return Err(event_data_not_found(0));
    }
    let model = repo::event_data::create(state.db(), &req).await?;
    Ok(EventDataResponse::from(&model))
}

pub async fn update(
    state: &AppState,
    id: u64,
    req: UpdateEventDataRequest,
    scope: &MonitorScope,
) -> AppResult<EventDataResponse> {
    let existing = repo::event_data::find_by_id(state.db(), id)
        .await?
        .ok_or_else(|| event_data_not_found(id))?;
    if !scope.allows_opt(existing.monitor_id, Level::Edit) {
        return Err(event_data_not_found(id));
    }
    let updated = repo::event_data::update(state.db(), id, &req)
        .await?
        .ok_or_else(|| event_data_not_found(id))?;
    Ok(EventDataResponse::from(&updated))
}

pub async fn delete(state: &AppState, id: u64, scope: &MonitorScope) -> AppResult<()> {
    let existing = repo::event_data::find_by_id(state.db(), id)
        .await?
        .ok_or_else(|| event_data_not_found(id))?;
    if !scope.allows_opt(existing.monitor_id, Level::Edit) {
        return Err(event_data_not_found(id));
    }
    let ok = repo::event_data::delete_by_id(state.db(), id).await?;
    if ok {
        Ok(())
    } else {
        Err(event_data_not_found(id))
    }
}
