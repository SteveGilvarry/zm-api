use crate::dto::request::snapshots_events::CreateSnapshotEventRequest;
use crate::dto::response::SnapshotEventResponse;
use crate::dto::{PaginatedResponse, PaginationParams};
use crate::error::{AppError, AppResult, Resource, ResourceType};
use crate::repo;
use crate::server::state::AppState;
use crate::service::monitor_acl::MonitorScope;
use crate::util::authz::Level;

fn not_found(id: u32) -> AppError {
    AppError::NotFoundError(Resource {
        details: vec![("id".into(), id.to_string())],
        resource_type: ResourceType::File,
    })
}

/// Row-level ACL: a `Snapshots_Events` row links to a monitor only through
/// its parent event. Resolve the event and check whether the caller can see
/// that monitor at the required level. Unrestricted callers skip the lookup.
async fn ensure_event_monitor_visible(
    state: &AppState,
    event_id: u64,
    required: Level,
    scope: &MonitorScope,
    snapshot_event_id: u32,
) -> AppResult<()> {
    if !scope.is_restricted() {
        return Ok(());
    }
    let event = repo::events::find_by_id(state, event_id)
        .await?
        .ok_or_else(|| not_found(snapshot_event_id))?;
    if scope.allows(event.monitor_id, required) {
        Ok(())
    } else {
        Err(not_found(snapshot_event_id))
    }
}

pub async fn list_all(
    state: &AppState,
    snapshot_id: Option<u32>,
    event_id: Option<u64>,
    scope: &MonitorScope,
) -> AppResult<Vec<SnapshotEventResponse>> {
    let filter = scope.visible_ids(Level::View);
    let items = if let Some(sid) = snapshot_id {
        repo::snapshots_events::find_by_snapshot_id(state.db(), sid, filter.as_deref()).await?
    } else if let Some(eid) = event_id {
        // For a single event, the cheapest check is to resolve the event and
        // assert scope; if it passes, return all rows for that event.
        if scope.is_restricted() {
            let event = repo::events::find_by_id(state, eid).await?;
            match event {
                Some(e) if scope.allows(e.monitor_id, Level::View) => {
                    repo::snapshots_events::find_by_event_id(state.db(), eid).await?
                }
                _ => Vec::new(),
            }
        } else {
            repo::snapshots_events::find_by_event_id(state.db(), eid).await?
        }
    } else {
        repo::snapshots_events::find_all(state.db(), filter.as_deref()).await?
    };
    Ok(items.iter().map(SnapshotEventResponse::from).collect())
}

pub async fn list_paginated(
    state: &AppState,
    params: &PaginationParams,
    scope: &MonitorScope,
) -> AppResult<PaginatedResponse<SnapshotEventResponse>> {
    let filter = scope.visible_ids(Level::View);
    let (items, total) =
        repo::snapshots_events::find_paginated(state.db(), params, filter.as_deref()).await?;
    let responses: Vec<SnapshotEventResponse> =
        items.iter().map(SnapshotEventResponse::from).collect();
    Ok(PaginatedResponse::from_params(responses, total, params))
}

pub async fn get_by_id(
    state: &AppState,
    id: u32,
    scope: &MonitorScope,
) -> AppResult<SnapshotEventResponse> {
    let item = repo::snapshots_events::find_by_id(state.db(), id)
        .await?
        .ok_or_else(|| not_found(id))?;
    ensure_event_monitor_visible(state, item.event_id, Level::View, scope, id).await?;
    Ok(SnapshotEventResponse::from(&item))
}

pub async fn create(
    state: &AppState,
    req: CreateSnapshotEventRequest,
    scope: &MonitorScope,
) -> AppResult<SnapshotEventResponse> {
    ensure_event_monitor_visible(state, req.event_id, Level::Edit, scope, 0).await?;
    let model = repo::snapshots_events::create(state.db(), &req).await?;
    Ok(SnapshotEventResponse::from(&model))
}

pub async fn delete(state: &AppState, id: u32, scope: &MonitorScope) -> AppResult<()> {
    let item = repo::snapshots_events::find_by_id(state.db(), id)
        .await?
        .ok_or_else(|| not_found(id))?;
    ensure_event_monitor_visible(state, item.event_id, Level::Edit, scope, id).await?;
    let ok = repo::snapshots_events::delete_by_id(state.db(), id).await?;
    if ok {
        Ok(())
    } else {
        Err(not_found(id))
    }
}
