//! HTTP handlers for the AI object-detection registry.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use garde::Validate;

use crate::dto::request::ai::{
    AiObjectClassQuery, CreateAiDatasetRequest, CreateAiModelRequest, CreateAiObjectClassRequest,
    UpdateAiDatasetRequest, UpdateAiModelRequest, UpdateAiObjectClassRequest,
};
use crate::dto::response::ai::{
    AiDatasetResponse, AiModelResponse, AiObjectClassResponse, PaginatedAiDatasetsResponse,
    PaginatedAiModelsResponse, PaginatedAiObjectClassesResponse,
};
use crate::dto::PaginationParams;
use crate::error::{AppError, AppResponseError, AppResult};
use crate::server::state::AppState;
use crate::service;

// --- Datasets --------------------------------------------------------------

/// List AI datasets (COCO and any user-added datasets).
#[utoipa::path(
    get, path = "/api/v3/ai/datasets", tag = "AI",
    responses((status = 200, description = "Paginated datasets", body = PaginatedAiDatasetsResponse)),
    security(("jwt" = []))
)]
pub async fn list_datasets(
    Query(params): Query<PaginationParams>,
    State(state): State<AppState>,
) -> AppResult<Json<PaginatedAiDatasetsResponse>> {
    let result = service::ai::list_datasets(&state, &params).await?;
    Ok(Json(PaginatedAiDatasetsResponse::from(result)))
}

/// Get one AI dataset.
#[utoipa::path(
    get, path = "/api/v3/ai/datasets/{id}", tag = "AI",
    params(("id" = u32, Path, description = "Dataset ID")),
    responses(
        (status = 200, description = "Dataset", body = AiDatasetResponse),
        (status = 404, description = "Not found", body = AppResponseError)
    ),
    security(("jwt" = []))
)]
pub async fn get_dataset(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<Json<AiDatasetResponse>> {
    Ok(Json(service::ai::get_dataset(&state, id).await?))
}

/// Create an AI dataset.
#[utoipa::path(
    post, path = "/api/v3/ai/datasets", tag = "AI",
    request_body = CreateAiDatasetRequest,
    responses((status = 201, description = "Created", body = AiDatasetResponse)),
    security(("jwt" = []))
)]
pub async fn create_dataset(
    State(state): State<AppState>,
    Json(req): Json<CreateAiDatasetRequest>,
) -> AppResult<(StatusCode, Json<AiDatasetResponse>)> {
    req.validate().map_err(AppError::InvalidInputError)?;
    let item = service::ai::create_dataset(&state, req).await?;
    Ok((StatusCode::CREATED, Json(item)))
}

/// Update an AI dataset (partial).
#[utoipa::path(
    patch, path = "/api/v3/ai/datasets/{id}", tag = "AI",
    params(("id" = u32, Path, description = "Dataset ID")),
    request_body = UpdateAiDatasetRequest,
    responses((status = 200, description = "Updated", body = AiDatasetResponse)),
    security(("jwt" = []))
)]
pub async fn update_dataset(
    Path(id): Path<u32>,
    State(state): State<AppState>,
    Json(req): Json<UpdateAiDatasetRequest>,
) -> AppResult<Json<AiDatasetResponse>> {
    req.validate().map_err(AppError::InvalidInputError)?;
    Ok(Json(service::ai::update_dataset(&state, id, req).await?))
}

/// Delete an AI dataset. Its object classes cascade.
#[utoipa::path(
    delete, path = "/api/v3/ai/datasets/{id}", tag = "AI",
    params(("id" = u32, Path, description = "Dataset ID")),
    responses((status = 204, description = "Deleted")),
    security(("jwt" = []))
)]
pub async fn delete_dataset(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<StatusCode> {
    service::ai::delete_dataset(&state, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// --- Models ----------------------------------------------------------------

/// List AI models, each with its dataset name resolved.
#[utoipa::path(
    get, path = "/api/v3/ai/models", tag = "AI",
    responses((status = 200, description = "Paginated models", body = PaginatedAiModelsResponse)),
    security(("jwt" = []))
)]
pub async fn list_models(
    Query(params): Query<PaginationParams>,
    State(state): State<AppState>,
) -> AppResult<Json<PaginatedAiModelsResponse>> {
    let result = service::ai::list_models(&state, &params).await?;
    Ok(Json(PaginatedAiModelsResponse::from(result)))
}

/// Get one AI model.
#[utoipa::path(
    get, path = "/api/v3/ai/models/{id}", tag = "AI",
    params(("id" = u32, Path, description = "Model ID")),
    responses(
        (status = 200, description = "Model", body = AiModelResponse),
        (status = 404, description = "Not found", body = AppResponseError)
    ),
    security(("jwt" = []))
)]
pub async fn get_model(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<Json<AiModelResponse>> {
    Ok(Json(service::ai::get_model(&state, id).await?))
}

/// Register an AI model.
#[utoipa::path(
    post, path = "/api/v3/ai/models", tag = "AI",
    request_body = CreateAiModelRequest,
    responses(
        (status = 201, description = "Created", body = AiModelResponse),
        (status = 400, description = "Unknown dataset", body = AppResponseError)
    ),
    security(("jwt" = []))
)]
pub async fn create_model(
    State(state): State<AppState>,
    Json(req): Json<CreateAiModelRequest>,
) -> AppResult<(StatusCode, Json<AiModelResponse>)> {
    req.validate().map_err(AppError::InvalidInputError)?;
    let item = service::ai::create_model(&state, req).await?;
    Ok((StatusCode::CREATED, Json(item)))
}

/// Update an AI model (partial).
#[utoipa::path(
    patch, path = "/api/v3/ai/models/{id}", tag = "AI",
    params(("id" = u32, Path, description = "Model ID")),
    request_body = UpdateAiModelRequest,
    responses((status = 200, description = "Updated", body = AiModelResponse)),
    security(("jwt" = []))
)]
pub async fn update_model(
    Path(id): Path<u32>,
    State(state): State<AppState>,
    Json(req): Json<UpdateAiModelRequest>,
) -> AppResult<Json<AiModelResponse>> {
    req.validate().map_err(AppError::InvalidInputError)?;
    Ok(Json(service::ai::update_model(&state, id, req).await?))
}

/// Delete an AI model.
#[utoipa::path(
    delete, path = "/api/v3/ai/models/{id}", tag = "AI",
    params(("id" = u32, Path, description = "Model ID")),
    responses((status = 204, description = "Deleted")),
    security(("jwt" = []))
)]
pub async fn delete_model(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<StatusCode> {
    service::ai::delete_model(&state, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// --- Object classes --------------------------------------------------------

/// List object classes, optionally filtered to one dataset.
#[utoipa::path(
    get, path = "/api/v3/ai/object-classes", tag = "AI",
    params(("dataset_id" = Option<u32>, Query, description = "Only classes in this dataset")),
    responses((status = 200, description = "Paginated classes", body = PaginatedAiObjectClassesResponse)),
    security(("jwt" = []))
)]
pub async fn list_classes(
    Query(filter): Query<AiObjectClassQuery>,
    Query(params): Query<PaginationParams>,
    State(state): State<AppState>,
) -> AppResult<Json<PaginatedAiObjectClassesResponse>> {
    filter.validate().map_err(AppError::InvalidInputError)?;
    let result = service::ai::list_classes(&state, &params, filter.dataset_id).await?;
    Ok(Json(PaginatedAiObjectClassesResponse::from(result)))
}

/// Get one object class.
#[utoipa::path(
    get, path = "/api/v3/ai/object-classes/{id}", tag = "AI",
    params(("id" = u32, Path, description = "Object class ID")),
    responses(
        (status = 200, description = "Object class", body = AiObjectClassResponse),
        (status = 404, description = "Not found", body = AppResponseError)
    ),
    security(("jwt" = []))
)]
pub async fn get_class(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<Json<AiObjectClassResponse>> {
    Ok(Json(service::ai::get_class(&state, id).await?))
}

/// Create an object class.
#[utoipa::path(
    post, path = "/api/v3/ai/object-classes", tag = "AI",
    request_body = CreateAiObjectClassRequest,
    responses(
        (status = 201, description = "Created", body = AiObjectClassResponse),
        (status = 400, description = "Unknown dataset", body = AppResponseError)
    ),
    security(("jwt" = []))
)]
pub async fn create_class(
    State(state): State<AppState>,
    Json(req): Json<CreateAiObjectClassRequest>,
) -> AppResult<(StatusCode, Json<AiObjectClassResponse>)> {
    req.validate().map_err(AppError::InvalidInputError)?;
    let item = service::ai::create_class(&state, req).await?;
    Ok((StatusCode::CREATED, Json(item)))
}

/// Update an object class (partial).
#[utoipa::path(
    patch, path = "/api/v3/ai/object-classes/{id}", tag = "AI",
    params(("id" = u32, Path, description = "Object class ID")),
    request_body = UpdateAiObjectClassRequest,
    responses((status = 200, description = "Updated", body = AiObjectClassResponse)),
    security(("jwt" = []))
)]
pub async fn update_class(
    Path(id): Path<u32>,
    State(state): State<AppState>,
    Json(req): Json<UpdateAiObjectClassRequest>,
) -> AppResult<Json<AiObjectClassResponse>> {
    req.validate().map_err(AppError::InvalidInputError)?;
    Ok(Json(service::ai::update_class(&state, id, req).await?))
}

/// Delete an object class.
#[utoipa::path(
    delete, path = "/api/v3/ai/object-classes/{id}", tag = "AI",
    params(("id" = u32, Path, description = "Object class ID")),
    responses((status = 204, description = "Deleted")),
    security(("jwt" = []))
)]
pub async fn delete_class(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<StatusCode> {
    service::ai::delete_class(&state, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
