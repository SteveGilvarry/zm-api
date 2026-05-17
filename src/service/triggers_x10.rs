use crate::dto::request::triggers_x10::{CreateTriggerX10Request, UpdateTriggerX10Request};
use crate::dto::response::TriggerX10Response;
use crate::dto::{PaginatedResponse, PaginationParams};
use crate::error::{AppError, AppResult, Resource, ResourceType};
use crate::repo;
use crate::server::state::AppState;
use crate::service::monitor_acl::MonitorScope;
use crate::util::authz::Level;

fn trigger_not_found(monitor_id: u32) -> AppError {
    AppError::NotFoundError(Resource {
        details: vec![("monitor_id".into(), monitor_id.to_string())],
        resource_type: ResourceType::Message,
    })
}

pub async fn list_all(
    state: &AppState,
    scope: &MonitorScope,
) -> AppResult<Vec<TriggerX10Response>> {
    let filter = scope.visible_ids(Level::View);
    let items = repo::triggers_x10::find_all(state.db(), filter.as_deref()).await?;
    Ok(items.iter().map(TriggerX10Response::from).collect())
}

pub async fn list_paginated(
    state: &AppState,
    params: &PaginationParams,
    scope: &MonitorScope,
) -> AppResult<PaginatedResponse<TriggerX10Response>> {
    let filter = scope.visible_ids(Level::View);
    let (items, total) =
        repo::triggers_x10::find_paginated(state.db(), params, filter.as_deref()).await?;
    let responses: Vec<TriggerX10Response> = items.iter().map(TriggerX10Response::from).collect();
    Ok(PaginatedResponse::from_params(responses, total, params))
}

pub async fn get_by_id(
    state: &AppState,
    monitor_id: u32,
    scope: &MonitorScope,
) -> AppResult<TriggerX10Response> {
    if !scope.allows(monitor_id, Level::View) {
        return Err(trigger_not_found(monitor_id));
    }
    let item = repo::triggers_x10::find_by_id(state.db(), monitor_id)
        .await?
        .ok_or_else(|| trigger_not_found(monitor_id))?;
    Ok(TriggerX10Response::from(&item))
}

pub async fn create(
    state: &AppState,
    req: CreateTriggerX10Request,
    scope: &MonitorScope,
) -> AppResult<TriggerX10Response> {
    if !scope.allows(req.monitor_id, Level::Edit) {
        return Err(trigger_not_found(req.monitor_id));
    }
    let model = repo::triggers_x10::create(state.db(), &req).await?;
    Ok(TriggerX10Response::from(&model))
}

pub async fn update(
    state: &AppState,
    monitor_id: u32,
    req: UpdateTriggerX10Request,
    scope: &MonitorScope,
) -> AppResult<TriggerX10Response> {
    if !scope.allows(monitor_id, Level::Edit) {
        return Err(trigger_not_found(monitor_id));
    }
    let updated = repo::triggers_x10::update(state.db(), monitor_id, &req)
        .await?
        .ok_or_else(|| trigger_not_found(monitor_id))?;
    Ok(TriggerX10Response::from(&updated))
}

pub async fn delete(state: &AppState, monitor_id: u32, scope: &MonitorScope) -> AppResult<()> {
    if !scope.allows(monitor_id, Level::Edit) {
        return Err(trigger_not_found(monitor_id));
    }
    let ok = repo::triggers_x10::delete_by_id(state.db(), monitor_id).await?;
    if ok {
        Ok(())
    } else {
        Err(trigger_not_found(monitor_id))
    }
}
