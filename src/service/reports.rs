use crate::dto::request::reports::{CreateReportRequest, UpdateReportRequest};
use crate::dto::response::ReportResponse;
use crate::error::{AppError, AppResult, Resource, ResourceType};
use crate::repo;
use crate::server::state::AppState;

pub async fn list_all(state: &AppState) -> AppResult<Vec<ReportResponse>> {
    let items = repo::reports::find_all(state.db()).await?;
    Ok(items.iter().map(ReportResponse::from).collect())
}

pub async fn get_by_id(state: &AppState, id: u32) -> AppResult<ReportResponse> {
    let item = repo::reports::find_by_id(state.db(), id).await?;
    let item = item.ok_or_else(|| {
        AppError::NotFoundError(Resource {
            details: vec![("id".into(), id.to_string())],
            resource_type: ResourceType::File,
        })
    })?;
    Ok(ReportResponse::from(&item))
}

pub async fn create(state: &AppState, req: CreateReportRequest) -> AppResult<ReportResponse> {
    let model = repo::reports::create(state.db(), &req).await?;
    Ok(ReportResponse::from(&model))
}

pub async fn update(
    state: &AppState,
    id: u32,
    req: UpdateReportRequest,
) -> AppResult<ReportResponse> {
    let updated = repo::reports::update(state.db(), id, &req).await?;
    let updated = updated.ok_or_else(|| {
        AppError::NotFoundError(Resource {
            details: vec![("id".into(), id.to_string())],
            resource_type: ResourceType::File,
        })
    })?;
    Ok(ReportResponse::from(&updated))
}

pub async fn delete(state: &AppState, id: u32) -> AppResult<()> {
    let ok = repo::reports::delete_by_id(state.db(), id).await?;
    if ok {
        Ok(())
    } else {
        Err(AppError::NotFoundError(Resource {
            details: vec![("id".into(), id.to_string())],
            resource_type: ResourceType::File,
        }))
    }
}
