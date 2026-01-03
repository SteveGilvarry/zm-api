use crate::dto::request::user_preferences::{
    CreateUserPreferenceRequest, UpdateUserPreferenceRequest,
};
use crate::dto::response::UserPreferenceResponse;
use crate::error::{AppError, AppResult, Resource, ResourceType};
use crate::repo;
use crate::server::state::AppState;

pub async fn list_all(
    state: &AppState,
    user_id: Option<u32>,
) -> AppResult<Vec<UserPreferenceResponse>> {
    let items = if let Some(uid) = user_id {
        repo::user_preferences::find_by_user(state.db(), uid).await?
    } else {
        repo::user_preferences::find_all(state.db()).await?
    };
    Ok(items.iter().map(UserPreferenceResponse::from).collect())
}

pub async fn get_by_id(state: &AppState, id: u32) -> AppResult<UserPreferenceResponse> {
    let item = repo::user_preferences::find_by_id(state.db(), id).await?;
    let item = item.ok_or_else(|| {
        AppError::NotFoundError(Resource {
            details: vec![("id".into(), id.to_string())],
            resource_type: ResourceType::Message,
        })
    })?;
    Ok(UserPreferenceResponse::from(&item))
}

pub async fn create(
    state: &AppState,
    req: CreateUserPreferenceRequest,
) -> AppResult<UserPreferenceResponse> {
    let model = repo::user_preferences::create(state.db(), &req).await?;
    Ok(UserPreferenceResponse::from(&model))
}

pub async fn update(
    state: &AppState,
    id: u32,
    req: UpdateUserPreferenceRequest,
) -> AppResult<UserPreferenceResponse> {
    let updated = repo::user_preferences::update(state.db(), id, &req).await?;
    let updated = updated.ok_or_else(|| {
        AppError::NotFoundError(Resource {
            details: vec![("id".into(), id.to_string())],
            resource_type: ResourceType::Message,
        })
    })?;
    Ok(UserPreferenceResponse::from(&updated))
}

pub async fn delete(state: &AppState, id: u32) -> AppResult<()> {
    let ok = repo::user_preferences::delete_by_id(state.db(), id).await?;
    if ok {
        Ok(())
    } else {
        Err(AppError::NotFoundError(Resource {
            details: vec![("id".into(), id.to_string())],
            resource_type: ResourceType::Message,
        }))
    }
}
