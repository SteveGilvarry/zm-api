use crate::dto::request::CreateModelRequest;
use crate::dto::response::ModelResponse;
use crate::error::AppResult;
use crate::server::state::AppState;
use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ModelListQuery {
    pub manufacturer_id: Option<u32>,
}

/// List camera models; optionally filter by manufacturer id.
///
/// - Requires a valid JWT.
#[utoipa::path(
    get,
    path = "/api/v3/models",
    params(("manufacturer_id" = Option<u32>, Query, description = "Filter by manufacturer")),
    responses((status = 200, description = "List models", body = [ModelResponse])),
    tag = "Models",
    security(("jwt" = []))
)]
pub async fn list_models(
    State(state): State<AppState>,
    Query(q): Query<ModelListQuery>,
) -> AppResult<Json<Vec<ModelResponse>>> {
    let items = crate::service::models::list_all(&state, q.manufacturer_id).await?;
    Ok(Json(items))
}

/// Get a camera model by id.
///
/// - Requires a valid JWT; responds 404 if not found.
#[utoipa::path(
    get,
    path = "/api/v3/models/{id}",
    params(("id" = u32, Path, description = "Model ID")),
    responses((status = 200, description = "Model detail", body = ModelResponse)),
    tag = "Models",
    security(("jwt" = []))
)]
pub async fn get_model(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<Json<ModelResponse>> {
    let item = crate::service::models::get_by_id(&state, id).await?;
    Ok(Json(item))
}

/// Create a new camera model entry.
///
/// - Optionally associates with a manufacturer id.
/// - Requires a valid JWT.
#[utoipa::path(
    post,
    path = "/api/v3/models",
    request_body = CreateModelRequest,
    responses((status = 201, description = "Created model", body = ModelResponse)),
    tag = "Models",
    security(("jwt" = []))
)]
pub async fn create_model(
    State(state): State<AppState>,
    Json(req): Json<CreateModelRequest>,
) -> AppResult<(axum::http::StatusCode, Json<ModelResponse>)> {
    let item = crate::service::models::create(&state, req).await?;
    Ok((axum::http::StatusCode::CREATED, Json(item)))
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateModelRequest {
    pub name: Option<String>,
    pub manufacturer_id: Option<i32>,
}

/// Update a camera model entry.
///
/// - Partial update; allows changing the model name and/or manufacturer id.
/// - Requires a valid JWT.
#[utoipa::path(
    patch,
    path = "/api/v3/models/{id}",
    params(("id" = u32, Path, description = "Model ID")),
    request_body = UpdateModelRequest,
    responses((status = 200, description = "Updated model", body = ModelResponse)),
    tag = "Models",
    security(("jwt" = []))
)]
pub async fn update_model(
    Path(id): Path<u32>,
    State(state): State<AppState>,
    Json(req): Json<UpdateModelRequest>,
) -> AppResult<Json<ModelResponse>> {
    let item = crate::service::models::update(&state, id, req.name, req.manufacturer_id).await?;
    Ok(Json(item))
}

/// Delete a camera model by id.
///
/// - Responds 204 on success, 404 if not found.
/// - Requires a valid JWT.
#[utoipa::path(
    delete,
    path = "/api/v3/models/{id}",
    params(("id" = u32, Path, description = "Model ID")),
    responses((status = 204, description = "Deleted model")),
    tag = "Models",
    security(("jwt" = []))
)]
pub async fn delete_model(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<axum::http::StatusCode> {
    crate::service::models::delete(&state, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
