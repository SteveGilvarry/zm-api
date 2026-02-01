use crate::dto::request::CreateStorageRequest;
use crate::dto::response::storage::PaginatedStorageResponse;
use crate::dto::response::StorageResponse;
use crate::dto::PaginationParams;
use crate::error::AppResult;
use crate::server::state::AppState;
use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;

/// List storage definitions used by ZoneMinder for event/video storage.
///
/// - Requires a valid JWT.
#[utoipa::path(
    get,
    path = "/api/v3/storage",
    params(
        ("page" = Option<u64>, Query, description = "Page number (1-indexed)", example = 1),
        ("page_size" = Option<u64>, Query, description = "Items per page (max 1000)", example = 25)
    ),
    responses((status = 200, description = "Paginated list of storage", body = PaginatedStorageResponse)),
    tag = "Storage",
    security(("jwt" = []))
)]
pub async fn list_storage(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> AppResult<Json<PaginatedStorageResponse>> {
    let result = crate::service::storage::list_paginated(&state, &params).await?;
    Ok(Json(PaginatedStorageResponse::from(result)))
}

/// Get a storage entry by id.
///
/// - Requires a valid JWT; responds 404 if not found.
#[utoipa::path(
    get,
    path = "/api/v3/storage/{id}",
    params(("id" = u16, Path, description = "Storage ID")),
    responses((status = 200, description = "Storage detail", body = serde_json::Value)),
    tag = "Storage",
    security(("jwt" = []))
)]
pub async fn get_storage(
    Path(id): Path<u16>,
    State(state): State<AppState>,
) -> AppResult<Json<StorageResponse>> {
    let item = crate::service::storage::get_by_id(&state, id).await?;
    Ok(Json(item))
}

/// Create a new storage entry.
///
/// - Accepts `type` and `scheme` as strings; converted to DB enums.
/// - Requires a valid JWT.
#[utoipa::path(
    post,
    path = "/api/v3/storage",
    request_body = CreateStorageRequest,
    responses((status = 201, description = "Created storage", body = StorageResponse)),
    tag = "Storage",
    security(("jwt" = []))
)]
pub async fn create_storage(
    State(state): State<AppState>,
    Json(req): Json<CreateStorageRequest>,
) -> AppResult<(axum::http::StatusCode, Json<StorageResponse>)> {
    let item = crate::service::storage::create(&state, req).await?;
    Ok((axum::http::StatusCode::CREATED, Json(item)))
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateStorageRequest {
    pub name: Option<String>,
    pub path: Option<String>,
    pub r#type: Option<String>,
    pub enabled: Option<i8>,
    pub scheme: Option<String>,
    pub server_id: Option<u32>,
    pub url: Option<String>,
}

/// Update storage fields (partial update).
///
/// - Applies provided fields; type/scheme strings are mapped to enums.
/// - Requires a valid JWT.
#[utoipa::path(
    patch,
    path = "/api/v3/storage/{id}",
    params(("id" = u16, Path, description = "Storage ID")),
    request_body = UpdateStorageRequest,
    responses((status = 200, description = "Updated storage", body = StorageResponse)),
    tag = "Storage",
    security(("jwt" = []))
)]
pub async fn update_storage(
    Path(id): Path<u16>,
    State(state): State<AppState>,
    Json(req): Json<UpdateStorageRequest>,
) -> AppResult<Json<StorageResponse>> {
    let item = crate::service::storage::update(
        &state,
        id,
        req.name,
        req.path,
        req.r#type,
        req.enabled,
        req.scheme,
        req.server_id,
        req.url,
    )
    .await?;
    Ok(Json(item))
}

/// Delete a storage entry by id.
///
/// - Responds 204 on success, 404 if not found.
/// - Requires a valid JWT.
#[utoipa::path(
    delete,
    path = "/api/v3/storage/{id}",
    params(("id" = u16, Path, description = "Storage ID")),
    responses((status = 204, description = "Deleted storage")),
    tag = "Storage",
    security(("jwt" = []))
)]
pub async fn delete_storage(
    Path(id): Path<u16>,
    State(state): State<AppState>,
) -> AppResult<axum::http::StatusCode> {
    crate::service::storage::delete(&state, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
