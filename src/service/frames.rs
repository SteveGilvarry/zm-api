use crate::dto::request::frames::{CreateFrameRequest, UpdateFrameRequest};
use crate::dto::response::frames::FrameResponse;
use crate::dto::{PaginatedResponse, PaginationParams};
use crate::error::{AppError, AppResult};
use crate::repo;
use crate::server::state::AppState;
use crate::service::monitor_acl::MonitorScope;
use crate::util::authz::Level;

fn frame_not_found(event_id: u64, frame_id: u32) -> AppError {
    AppError::NotFoundError(crate::error::Resource {
        resource_type: crate::error::ResourceType::File,
        details: vec![
            ("event_id".to_string(), event_id.to_string()),
            ("frame_id".to_string(), frame_id.to_string()),
        ],
    })
}

/// Row-level ACL for a frame: a frame links to a monitor only through its
/// parent event, so resolve the event and check its monitor. Skipped entirely
/// for unrestricted callers.
async fn ensure_event_monitor_visible(
    state: &AppState,
    event_id: u64,
    required: Level,
    scope: &MonitorScope,
    frame_id: u32,
) -> AppResult<()> {
    if !scope.is_restricted() {
        return Ok(());
    }
    let event = repo::events::find_by_id(state, event_id)
        .await?
        .ok_or_else(|| frame_not_found(event_id, frame_id))?;
    if scope.allows(event.monitor_id, required) {
        Ok(())
    } else {
        Err(frame_not_found(event_id, frame_id))
    }
}

/// List all frames for a single event (bounded — `event_id` is required, see
/// [`repo::frames::find_all`]). Broad listings must use [`list_paginated`].
pub async fn list_all(
    state: &AppState,
    event_id: u64,
    scope: &MonitorScope,
) -> AppResult<Vec<FrameResponse>> {
    let filter = scope.visible_ids(Level::View);
    let frames = repo::frames::find_all(state.db(), event_id, filter.as_deref()).await?;
    Ok(frames.iter().map(FrameResponse::from).collect())
}

/// List frames with pagination, optionally filtered by event_id
pub async fn list_paginated(
    state: &AppState,
    event_id: Option<u64>,
    params: &PaginationParams,
    scope: &MonitorScope,
) -> AppResult<PaginatedResponse<FrameResponse>> {
    let filter = scope.visible_ids(Level::View);
    let (items, total) =
        repo::frames::find_paginated(state.db(), event_id, params, filter.as_deref()).await?;
    let responses: Vec<FrameResponse> = items.iter().map(FrameResponse::from).collect();
    Ok(PaginatedResponse::from_params(responses, total, params))
}

/// Get a frame by event and frame number
pub async fn get(
    state: &AppState,
    event_id: u64,
    frame_id: u32,
    scope: &MonitorScope,
) -> AppResult<FrameResponse> {
    let frame = repo::frames::find_by_key(state.db(), event_id, frame_id)
        .await?
        .ok_or_else(|| frame_not_found(event_id, frame_id))?;
    ensure_event_monitor_visible(state, event_id, Level::View, scope, frame_id).await?;
    Ok(FrameResponse::from(&frame))
}

/// Create a new frame
pub async fn create(
    state: &AppState,
    req: CreateFrameRequest,
    scope: &MonitorScope,
) -> AppResult<FrameResponse> {
    ensure_event_monitor_visible(state, req.event_id, Level::Edit, scope, req.frame_id).await?;
    let frame = repo::frames::create(state.db(), &req).await?;
    Ok(FrameResponse::from(&frame))
}

/// Update a frame by event and frame number
pub async fn update(
    state: &AppState,
    event_id: u64,
    frame_id: u32,
    req: UpdateFrameRequest,
    scope: &MonitorScope,
) -> AppResult<FrameResponse> {
    repo::frames::find_by_key(state.db(), event_id, frame_id)
        .await?
        .ok_or_else(|| frame_not_found(event_id, frame_id))?;
    ensure_event_monitor_visible(state, event_id, Level::Edit, scope, frame_id).await?;
    let frame = repo::frames::update(state.db(), event_id, frame_id, &req).await?;
    Ok(FrameResponse::from(&frame))
}

/// Delete a frame by event and frame number
pub async fn delete(
    state: &AppState,
    event_id: u64,
    frame_id: u32,
    scope: &MonitorScope,
) -> AppResult<()> {
    repo::frames::find_by_key(state.db(), event_id, frame_id)
        .await?
        .ok_or_else(|| frame_not_found(event_id, frame_id))?;
    ensure_event_monitor_visible(state, event_id, Level::Edit, scope, frame_id).await?;
    repo::frames::delete_by_key(state.db(), event_id, frame_id).await
}
