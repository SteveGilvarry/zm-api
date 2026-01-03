use crate::dto::request::montage_layouts::{
    CreateMontageLayoutRequest, UpdateMontageLayoutRequest,
};
use crate::dto::response::MontageLayoutResponse;
use crate::error::{AppError, AppResult, Resource, ResourceType};
use crate::repo;
use crate::server::state::AppState;

pub async fn list_all(
    state: &AppState,
    user_id: Option<u32>,
) -> AppResult<Vec<MontageLayoutResponse>> {
    let items = if let Some(uid) = user_id {
        repo::montage_layouts::find_by_user(state.db(), uid).await?
    } else {
        repo::montage_layouts::find_all(state.db()).await?
    };
    Ok(items.iter().map(MontageLayoutResponse::from).collect())
}

pub async fn get_by_id(state: &AppState, id: u32) -> AppResult<MontageLayoutResponse> {
    let item = repo::montage_layouts::find_by_id(state.db(), id).await?;
    let item = item.ok_or_else(|| {
        AppError::NotFoundError(Resource {
            details: vec![("id".into(), id.to_string())],
            resource_type: ResourceType::Message,
        })
    })?;
    Ok(MontageLayoutResponse::from(&item))
}

pub async fn create(
    state: &AppState,
    req: CreateMontageLayoutRequest,
) -> AppResult<MontageLayoutResponse> {
    let model = repo::montage_layouts::create(state.db(), &req).await?;
    Ok(MontageLayoutResponse::from(&model))
}

pub async fn update(
    state: &AppState,
    id: u32,
    req: UpdateMontageLayoutRequest,
) -> AppResult<MontageLayoutResponse> {
    let updated = repo::montage_layouts::update(state.db(), id, &req).await?;
    let updated = updated.ok_or_else(|| {
        AppError::NotFoundError(Resource {
            details: vec![("id".into(), id.to_string())],
            resource_type: ResourceType::Message,
        })
    })?;
    Ok(MontageLayoutResponse::from(&updated))
}

pub async fn delete(state: &AppState, id: u32) -> AppResult<()> {
    let ok = repo::montage_layouts::delete_by_id(state.db(), id).await?;
    if ok {
        Ok(())
    } else {
        Err(AppError::NotFoundError(Resource {
            details: vec![("id".into(), id.to_string())],
            resource_type: ResourceType::Message,
        }))
    }
}
