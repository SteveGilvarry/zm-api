use crate::dto::request::CreateManufacturerRequest;
use crate::dto::response::ManufacturerResponse;
use crate::error::AppResult;
use crate::server::state::AppState;
use axum::{
    extract::{Path, State},
    Json,
};
use serde::Deserialize;

/// List manufacturers for camera models supported in ZoneMinder presets.
///
/// - Requires a valid JWT.
#[utoipa::path(
    get,
    path = "/api/v3/manufacturers",
    responses((status = 200, description = "List manufacturers", body = serde_json::Value)),
    tag = "Manufacturers",
    security(("jwt" = []))
)]
pub async fn list_manufacturers(
    State(state): State<AppState>,
) -> AppResult<Json<Vec<ManufacturerResponse>>> {
    let items = crate::service::manufacturers::list_all(&state).await?;
    Ok(Json(items))
}

/// Get manufacturer by id.
///
/// - Requires a valid JWT; responds 404 if not found.
#[utoipa::path(
    get,
    path = "/api/v3/manufacturers/{id}",
    params(("id" = u32, Path, description = "Manufacturer ID")),
    responses((status = 200, description = "Manufacturer detail", body = serde_json::Value)),
    tag = "Manufacturers",
    security(("jwt" = []))
)]
pub async fn get_manufacturer(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<Json<ManufacturerResponse>> {
    let item = crate::service::manufacturers::get_by_id(&state, id).await?;
    Ok(Json(item))
}

/// Create a new manufacturer record.
///
/// - Name must be unique.
/// - Requires a valid JWT.
#[utoipa::path(
    post,
    path = "/api/v3/manufacturers",
    request_body = CreateManufacturerRequest,
    responses((status = 201, description = "Created manufacturer", body = ManufacturerResponse)),
    tag = "Manufacturers",
    security(("jwt" = []))
)]
pub async fn create_manufacturer(
    State(state): State<AppState>,
    Json(req): Json<CreateManufacturerRequest>,
) -> AppResult<(axum::http::StatusCode, Json<ManufacturerResponse>)> {
    let item = crate::service::manufacturers::create(&state, req).await?;
    Ok((axum::http::StatusCode::CREATED, Json(item)))
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateManufacturerRequest {
    pub name: Option<String>,
}

/// Update a manufacturer name.
///
/// - Partial update.
/// - Requires a valid JWT.
#[utoipa::path(
    patch,
    path = "/api/v3/manufacturers/{id}",
    params(("id" = u32, Path, description = "Manufacturer ID")),
    request_body = UpdateManufacturerRequest,
    responses((status = 200, description = "Updated manufacturer", body = ManufacturerResponse)),
    tag = "Manufacturers",
    security(("jwt" = []))
)]
pub async fn update_manufacturer(
    Path(id): Path<u32>,
    State(state): State<AppState>,
    Json(req): Json<UpdateManufacturerRequest>,
) -> AppResult<Json<ManufacturerResponse>> {
    let item = crate::service::manufacturers::update(&state, id, req.name).await?;
    Ok(Json(item))
}

/// Delete a manufacturer by id.
///
/// - Responds 204 on success, 404 if not found.
/// - Requires a valid JWT.
#[utoipa::path(
    delete,
    path = "/api/v3/manufacturers/{id}",
    params(("id" = u32, Path, description = "Manufacturer ID")),
    responses((status = 204, description = "Deleted manufacturer")),
    tag = "Manufacturers",
    security(("jwt" = []))
)]
pub async fn delete_manufacturer(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<axum::http::StatusCode> {
    crate::service::manufacturers::delete(&state, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
