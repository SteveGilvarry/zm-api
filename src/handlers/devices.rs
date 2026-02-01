use crate::dto::request::devices::{CreateDeviceRequest, UpdateDeviceRequest};
use crate::dto::response::devices::PaginatedDevicesResponse;
use crate::dto::response::DeviceResponse;
use crate::dto::PaginationParams;
use crate::error::AppResult;
use crate::server::state::AppState;
use axum::{
    extract::{Path, Query, State},
    Json,
};

/// List all devices with pagination.
///
/// - Requires a valid JWT.
#[utoipa::path(
    get,
    path = "/api/v3/devices",
    params(
        ("page" = Option<u64>, Query, description = "Page number (1-indexed)", example = 1),
        ("page_size" = Option<u64>, Query, description = "Items per page (max 1000)", example = 25)
    ),
    responses((status = 200, description = "Paginated list of devices", body = PaginatedDevicesResponse)),
    tag = "Devices",
    security(("jwt" = []))
)]
pub async fn list_devices(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> AppResult<Json<PaginatedDevicesResponse>> {
    let result = crate::service::devices::list_paginated(&state, &params).await?;
    Ok(Json(PaginatedDevicesResponse::from(result)))
}

/// Get device by id.
///
/// - Requires a valid JWT; responds 404 if not found.
#[utoipa::path(
    get,
    path = "/api/v3/devices/{id}",
    params(("id" = u32, Path, description = "Device ID")),
    responses((status = 200, description = "Device detail", body = DeviceResponse)),
    tag = "Devices",
    security(("jwt" = []))
)]
pub async fn get_device(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<Json<DeviceResponse>> {
    let item = crate::service::devices::get_by_id(&state, id).await?;
    Ok(Json(item))
}

/// Create a new device record.
///
/// - Requires a valid JWT.
#[utoipa::path(
    post,
    path = "/api/v3/devices",
    request_body = CreateDeviceRequest,
    responses((status = 201, description = "Created device", body = DeviceResponse)),
    tag = "Devices",
    security(("jwt" = []))
)]
pub async fn create_device(
    State(state): State<AppState>,
    Json(req): Json<CreateDeviceRequest>,
) -> AppResult<(axum::http::StatusCode, Json<DeviceResponse>)> {
    let item = crate::service::devices::create(&state, req).await?;
    Ok((axum::http::StatusCode::CREATED, Json(item)))
}

/// Update a device.
///
/// - Partial update.
/// - Requires a valid JWT.
#[utoipa::path(
    patch,
    path = "/api/v3/devices/{id}",
    params(("id" = u32, Path, description = "Device ID")),
    request_body = UpdateDeviceRequest,
    responses((status = 200, description = "Updated device", body = DeviceResponse)),
    tag = "Devices",
    security(("jwt" = []))
)]
pub async fn update_device(
    Path(id): Path<u32>,
    State(state): State<AppState>,
    Json(req): Json<UpdateDeviceRequest>,
) -> AppResult<Json<DeviceResponse>> {
    let item = crate::service::devices::update(&state, id, req).await?;
    Ok(Json(item))
}

/// Delete a device by id.
///
/// - Responds 204 on success, 404 if not found.
/// - Requires a valid JWT.
#[utoipa::path(
    delete,
    path = "/api/v3/devices/{id}",
    params(("id" = u32, Path, description = "Device ID")),
    responses((status = 204, description = "Deleted device")),
    tag = "Devices",
    security(("jwt" = []))
)]
pub async fn delete_device(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<axum::http::StatusCode> {
    crate::service::devices::delete(&state, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
