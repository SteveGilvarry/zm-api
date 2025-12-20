use crate::dto::request::events_tags::CreateEventTagRequest;
use crate::dto::response::EventTagResponse;
use crate::error::{AppError, AppResult, Resource, ResourceType};
use crate::repo;
use crate::server::state::AppState;

pub async fn list_all(
    state: &AppState,
    event_id: Option<u64>,
    tag_id: Option<u64>,
) -> AppResult<Vec<EventTagResponse>> {
    let items = repo::events_tags::find_all(state.db(), event_id, tag_id).await?;
    Ok(items.iter().map(EventTagResponse::from).collect())
}

pub async fn get_by_id(
    state: &AppState,
    tag_id: u64,
    event_id: u64,
) -> AppResult<EventTagResponse> {
    let item = repo::events_tags::find_by_composite_id(state.db(), tag_id, event_id).await?;
    let item = item.ok_or_else(|| {
        AppError::NotFoundError(Resource {
            details: vec![
                ("tag_id".into(), tag_id.to_string()),
                ("event_id".into(), event_id.to_string()),
            ],
            resource_type: ResourceType::EventTag,
        })
    })?;
    Ok(EventTagResponse::from(&item))
}

pub async fn create(
    state: &AppState,
    req: CreateEventTagRequest,
) -> AppResult<EventTagResponse> {
    // Check if association already exists
    let existing =
        repo::events_tags::find_by_composite_id(state.db(), req.tag_id, req.event_id).await?;
    if existing.is_some() {
        return Err(AppError::ResourceExistsError(Resource {
            details: vec![
                ("tag_id".into(), req.tag_id.to_string()),
                ("event_id".into(), req.event_id.to_string()),
            ],
            resource_type: ResourceType::EventTag,
        }));
    }

    let model = repo::events_tags::create(state.db(), &req).await?;
    Ok(EventTagResponse::from(&model))
}

pub async fn delete(state: &AppState, tag_id: u64, event_id: u64) -> AppResult<()> {
    let ok = repo::events_tags::delete_by_composite_id(state.db(), tag_id, event_id).await?;
    if ok {
        Ok(())
    } else {
        Err(AppError::NotFoundError(Resource {
            details: vec![
                ("tag_id".into(), tag_id.to_string()),
                ("event_id".into(), event_id.to_string()),
            ],
            resource_type: ResourceType::EventTag,
        }))
    }
}
