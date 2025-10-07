use crate::server::state::AppState;
use crate::dto::request::frames::{CreateFrameRequest, UpdateFrameRequest};
use crate::dto::response::frames::FrameResponse;
use crate::error::{AppError, AppResult};
use crate::repo;

/// List all frames, optionally filtered by event_id
pub async fn list_all(state: &AppState, event_id: Option<u64>) -> AppResult<Vec<FrameResponse>> {
    let frames = repo::frames::find_all(state.db(), event_id).await?;
    Ok(frames.iter().map(FrameResponse::from).collect())
}

/// Get frame by id
pub async fn get_by_id(state: &AppState, id: u64) -> AppResult<FrameResponse> {
    let frame = repo::frames::find_by_id(state.db(), id)
        .await?
        .ok_or_else(|| {
            AppError::NotFoundError(crate::error::Resource {
                resource_type: crate::error::ResourceType::File,
                details: vec![("id".to_string(), id.to_string())],
            })
        })?;
    Ok(FrameResponse::from(&frame))
}

/// Create a new frame
pub async fn create(state: &AppState, req: CreateFrameRequest) -> AppResult<FrameResponse> {
    let frame = repo::frames::create(state.db(), &req).await?;
    Ok(FrameResponse::from(&frame))
}

/// Update frame by id
pub async fn update(
    state: &AppState,
    id: u64,
    req: UpdateFrameRequest,
) -> AppResult<FrameResponse> {
    let frame = repo::frames::update(state.db(), id, &req).await?;
    Ok(FrameResponse::from(&frame))
}

/// Delete frame by id
pub async fn delete(state: &AppState, id: u64) -> AppResult<()> {
    repo::frames::delete_by_id(state.db(), id).await
}
