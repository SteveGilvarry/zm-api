use crate::dto::request::devices::{CreateDeviceRequest, UpdateDeviceRequest};
use crate::dto::response::DeviceResponse;
use crate::error::{AppError, AppResult, Resource, ResourceType};
use crate::repo;
use crate::server::state::AppState;

pub async fn list_all(state: &AppState) -> AppResult<Vec<DeviceResponse>> {
    let items = repo::devices::find_all(state.db()).await?;
    Ok(items.iter().map(DeviceResponse::from).collect())
}

pub async fn get_by_id(state: &AppState, id: u32) -> AppResult<DeviceResponse> {
    let item = repo::devices::find_by_id(state.db(), id).await?;
    let item = item.ok_or_else(|| {
        AppError::NotFoundError(Resource {
            details: vec![("id".into(), id.to_string())],
            resource_type: ResourceType::Message,
        })
    })?;
    Ok(DeviceResponse::from(&item))
}

pub async fn create(state: &AppState, req: CreateDeviceRequest) -> AppResult<DeviceResponse> {
    let model = repo::devices::create(state.db(), &req).await?;
    Ok(DeviceResponse::from(&model))
}

pub async fn update(
    state: &AppState,
    id: u32,
    req: UpdateDeviceRequest,
) -> AppResult<DeviceResponse> {
    let updated = repo::devices::update(state.db(), id, &req).await?;
    let updated = updated.ok_or_else(|| {
        AppError::NotFoundError(Resource {
            details: vec![("id".into(), id.to_string())],
            resource_type: ResourceType::Message,
        })
    })?;
    Ok(DeviceResponse::from(&updated))
}

pub async fn delete(state: &AppState, id: u32) -> AppResult<()> {
    let ok = repo::devices::delete_by_id(state.db(), id).await?;
    if ok {
        Ok(())
    } else {
        Err(AppError::NotFoundError(Resource {
            details: vec![("id".into(), id.to_string())],
            resource_type: ResourceType::Message,
        }))
    }
}
