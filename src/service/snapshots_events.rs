use crate::dto::request::snapshots_events::CreateSnapshotEventRequest;
use crate::dto::response::SnapshotEventResponse;
use crate::error::{AppError, AppResult, Resource, ResourceType};
use crate::repo;
use crate::server::state::AppState;

pub async fn list_all(
    state: &AppState,
    snapshot_id: Option<u32>,
    event_id: Option<u64>,
) -> AppResult<Vec<SnapshotEventResponse>> {
    let items = if let Some(sid) = snapshot_id {
        repo::snapshots_events::find_by_snapshot_id(state.db(), sid).await?
    } else if let Some(eid) = event_id {
        repo::snapshots_events::find_by_event_id(state.db(), eid).await?
    } else {
        repo::snapshots_events::find_all(state.db()).await?
    };
    Ok(items.iter().map(SnapshotEventResponse::from).collect())
}

pub async fn get_by_id(state: &AppState, id: u32) -> AppResult<SnapshotEventResponse> {
    let item = repo::snapshots_events::find_by_id(state.db(), id).await?;
    let item = item.ok_or_else(|| {
        AppError::NotFoundError(Resource {
            details: vec![("id".into(), id.to_string())],
            resource_type: ResourceType::File,
        })
    })?;
    Ok(SnapshotEventResponse::from(&item))
}

pub async fn create(
    state: &AppState,
    req: CreateSnapshotEventRequest,
) -> AppResult<SnapshotEventResponse> {
    let model = repo::snapshots_events::create(state.db(), &req).await?;
    Ok(SnapshotEventResponse::from(&model))
}

pub async fn delete(state: &AppState, id: u32) -> AppResult<()> {
    let ok = repo::snapshots_events::delete_by_id(state.db(), id).await?;
    if ok {
        Ok(())
    } else {
        Err(AppError::NotFoundError(Resource {
            details: vec![("id".into(), id.to_string())],
            resource_type: ResourceType::File,
        }))
    }
}
