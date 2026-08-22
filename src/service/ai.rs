//! Business logic for the AI object-detection registry.
//!
//! Mirrors the CRUD ZoneMinder's own Options UI offers over `AI_Datasets`,
//! `AI_Models`, and `AI_Object_Classes`. The remaining two AI tables
//! (`AI_Detection_Settings`, `AI_Detections`) are deliberately not exposed
//! yet — see GH #47.

use std::collections::HashMap;

use crate::dto::request::ai::{
    CreateAiDatasetRequest, CreateAiModelRequest, CreateAiObjectClassRequest,
    UpdateAiDatasetRequest, UpdateAiModelRequest, UpdateAiObjectClassRequest,
};
use crate::dto::response::ai::{AiDatasetResponse, AiModelResponse, AiObjectClassResponse};
use crate::dto::{PaginatedResponse, PaginationParams};
use crate::error::{AppError, AppResult, Resource, ResourceType};
use crate::repo;
use crate::server::state::AppState;

fn not_found(kind: &str, id: u32) -> AppError {
    AppError::NotFoundError(Resource {
        details: vec![(format!("{kind}_id"), id.to_string())],
        resource_type: ResourceType::Config,
    })
}

// --- Datasets --------------------------------------------------------------

pub async fn list_datasets(
    state: &AppState,
    params: &PaginationParams,
) -> AppResult<PaginatedResponse<AiDatasetResponse>> {
    let (items, total) = repo::ai::datasets_paginated(state.db(), params).await?;
    let responses = items.iter().map(AiDatasetResponse::from).collect();
    Ok(PaginatedResponse::from_params(responses, total, params))
}

pub async fn get_dataset(state: &AppState, id: u32) -> AppResult<AiDatasetResponse> {
    let item = repo::ai::dataset_by_id(state.db(), id)
        .await?
        .ok_or_else(|| not_found("dataset", id))?;
    Ok(AiDatasetResponse::from(&item))
}

pub async fn create_dataset(
    state: &AppState,
    req: CreateAiDatasetRequest,
) -> AppResult<AiDatasetResponse> {
    let model = repo::ai::create_dataset(state.db(), &req).await?;
    Ok(AiDatasetResponse::from(&model))
}

pub async fn update_dataset(
    state: &AppState,
    id: u32,
    req: UpdateAiDatasetRequest,
) -> AppResult<AiDatasetResponse> {
    let item = repo::ai::update_dataset(state.db(), id, &req)
        .await?
        .ok_or_else(|| not_found("dataset", id))?;
    Ok(AiDatasetResponse::from(&item))
}

pub async fn delete_dataset(state: &AppState, id: u32) -> AppResult<()> {
    if repo::ai::delete_dataset(state.db(), id).await? {
        Ok(())
    } else {
        Err(not_found("dataset", id))
    }
}

// --- Models ----------------------------------------------------------------

/// List models, joining each one's dataset name so a client can render the
/// registry table without a second round trip (the legacy UI shows this too).
pub async fn list_models(
    state: &AppState,
    params: &PaginationParams,
) -> AppResult<PaginatedResponse<AiModelResponse>> {
    let (items, total) = repo::ai::models_paginated(state.db(), params).await?;

    // Resolve only the dataset names actually referenced, in one query rather
    // than one per row.
    let referenced: Vec<u32> = {
        let mut v: Vec<u32> = items.iter().filter_map(|m| m.dataset_id).collect();
        v.sort_unstable();
        v.dedup();
        v
    };
    let names: HashMap<u32, String> = repo::ai::datasets_by_ids(state.db(), &referenced)
        .await?
        .into_iter()
        .map(|d| (d.id, d.name))
        .collect();

    let responses = items
        .iter()
        .map(|m| {
            let mut r = AiModelResponse::from(m);
            r.dataset_name = m.dataset_id.and_then(|d| names.get(&d).cloned());
            r
        })
        .collect();
    Ok(PaginatedResponse::from_params(responses, total, params))
}

pub async fn get_model(state: &AppState, id: u32) -> AppResult<AiModelResponse> {
    let item = repo::ai::model_by_id(state.db(), id)
        .await?
        .ok_or_else(|| not_found("model", id))?;
    let mut resp = AiModelResponse::from(&item);
    if let Some(d) = item.dataset_id {
        resp.dataset_name = repo::ai::dataset_by_id(state.db(), d)
            .await?
            .map(|x| x.name);
    }
    Ok(resp)
}

pub async fn create_model(
    state: &AppState,
    req: CreateAiModelRequest,
) -> AppResult<AiModelResponse> {
    // Reject a dangling dataset link up front rather than surfacing a raw FK error.
    if let Some(d) = req.dataset_id {
        if repo::ai::dataset_by_id(state.db(), d).await?.is_none() {
            return Err(AppError::BadRequestError(format!(
                "dataset {d} does not exist"
            )));
        }
    }
    let model = repo::ai::create_model(state.db(), &req).await?;
    Ok(AiModelResponse::from(&model))
}

pub async fn update_model(
    state: &AppState,
    id: u32,
    req: UpdateAiModelRequest,
) -> AppResult<AiModelResponse> {
    if let Some(Some(d)) = req.dataset_id {
        if repo::ai::dataset_by_id(state.db(), d).await?.is_none() {
            return Err(AppError::BadRequestError(format!(
                "dataset {d} does not exist"
            )));
        }
    }
    let item = repo::ai::update_model(state.db(), id, &req)
        .await?
        .ok_or_else(|| not_found("model", id))?;
    Ok(AiModelResponse::from(&item))
}

pub async fn delete_model(state: &AppState, id: u32) -> AppResult<()> {
    if repo::ai::delete_model(state.db(), id).await? {
        Ok(())
    } else {
        Err(not_found("model", id))
    }
}

// --- Object classes --------------------------------------------------------

pub async fn list_classes(
    state: &AppState,
    params: &PaginationParams,
    dataset_id: Option<u32>,
) -> AppResult<PaginatedResponse<AiObjectClassResponse>> {
    let (items, total) = repo::ai::classes_paginated(state.db(), params, dataset_id).await?;
    let responses = items.iter().map(AiObjectClassResponse::from).collect();
    Ok(PaginatedResponse::from_params(responses, total, params))
}

pub async fn get_class(state: &AppState, id: u32) -> AppResult<AiObjectClassResponse> {
    let item = repo::ai::class_by_id(state.db(), id)
        .await?
        .ok_or_else(|| not_found("object_class", id))?;
    Ok(AiObjectClassResponse::from(&item))
}

pub async fn create_class(
    state: &AppState,
    req: CreateAiObjectClassRequest,
) -> AppResult<AiObjectClassResponse> {
    if repo::ai::dataset_by_id(state.db(), req.dataset_id)
        .await?
        .is_none()
    {
        return Err(AppError::BadRequestError(format!(
            "dataset {} does not exist",
            req.dataset_id
        )));
    }
    let model = repo::ai::create_class(state.db(), &req).await?;
    Ok(AiObjectClassResponse::from(&model))
}

pub async fn update_class(
    state: &AppState,
    id: u32,
    req: UpdateAiObjectClassRequest,
) -> AppResult<AiObjectClassResponse> {
    if let Some(d) = req.dataset_id {
        if repo::ai::dataset_by_id(state.db(), d).await?.is_none() {
            return Err(AppError::BadRequestError(format!(
                "dataset {d} does not exist"
            )));
        }
    }
    let item = repo::ai::update_class(state.db(), id, &req)
        .await?
        .ok_or_else(|| not_found("object_class", id))?;
    Ok(AiObjectClassResponse::from(&item))
}

pub async fn delete_class(state: &AppState, id: u32) -> AppResult<()> {
    if repo::ai::delete_class(state.db(), id).await? {
        Ok(())
    } else {
        Err(not_found("object_class", id))
    }
}
