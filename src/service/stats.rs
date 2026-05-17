use crate::dto::request::stats::{CreateStatRequest, UpdateStatRequest};
use crate::dto::response::StatResponse;
use crate::dto::{PaginatedResponse, PaginationParams};
use crate::error::{AppError, AppResult, Resource, ResourceType};
use crate::repo;
use crate::server::state::AppState;
use crate::service::monitor_acl::MonitorScope;
use crate::util::authz::Level;

fn stat_not_found(id: u32) -> AppError {
    AppError::NotFoundError(Resource {
        details: vec![("id".into(), id.to_string())],
        resource_type: ResourceType::Message,
    })
}

pub async fn list_all(state: &AppState, scope: &MonitorScope) -> AppResult<Vec<StatResponse>> {
    let filter = scope.visible_ids(Level::View);
    let items = repo::stats::find_all(state.db(), filter.as_deref()).await?;
    Ok(items.iter().map(StatResponse::from).collect())
}

pub async fn list_paginated(
    state: &AppState,
    params: &PaginationParams,
    scope: &MonitorScope,
) -> AppResult<PaginatedResponse<StatResponse>> {
    let filter = scope.visible_ids(Level::View);
    let (items, total) = repo::stats::find_paginated(state.db(), params, filter.as_deref()).await?;
    let responses: Vec<StatResponse> = items.iter().map(StatResponse::from).collect();
    Ok(PaginatedResponse::from_params(responses, total, params))
}

pub async fn get_by_id(state: &AppState, id: u32, scope: &MonitorScope) -> AppResult<StatResponse> {
    let item = repo::stats::find_by_id(state.db(), id)
        .await?
        .ok_or_else(|| stat_not_found(id))?;
    if !scope.allows(item.monitor_id, Level::View) {
        return Err(stat_not_found(id));
    }
    Ok(StatResponse::from(&item))
}

pub async fn create(
    state: &AppState,
    req: CreateStatRequest,
    scope: &MonitorScope,
) -> AppResult<StatResponse> {
    if !scope.allows(req.monitor_id, Level::Edit) {
        return Err(stat_not_found(0));
    }
    let model = repo::stats::create(state.db(), &req).await?;
    Ok(StatResponse::from(&model))
}

pub async fn update(
    state: &AppState,
    id: u32,
    req: UpdateStatRequest,
    scope: &MonitorScope,
) -> AppResult<StatResponse> {
    // Fetch first so the stat's monitor can be ACL-checked before mutation.
    let existing = repo::stats::find_by_id(state.db(), id)
        .await?
        .ok_or_else(|| stat_not_found(id))?;
    if !scope.allows(existing.monitor_id, Level::Edit) {
        return Err(stat_not_found(id));
    }
    let updated = repo::stats::update(state.db(), id, &req)
        .await?
        .ok_or_else(|| stat_not_found(id))?;
    Ok(StatResponse::from(&updated))
}

pub async fn delete(state: &AppState, id: u32, scope: &MonitorScope) -> AppResult<()> {
    let existing = repo::stats::find_by_id(state.db(), id)
        .await?
        .ok_or_else(|| stat_not_found(id))?;
    if !scope.allows(existing.monitor_id, Level::Edit) {
        return Err(stat_not_found(id));
    }
    let ok = repo::stats::delete_by_id(state.db(), id).await?;
    if ok {
        Ok(())
    } else {
        Err(stat_not_found(id))
    }
}
