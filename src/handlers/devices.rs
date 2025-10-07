use axum::{extract::{Path, State}, Json};
use crate::dto::response::DeviceResponse;
use crate::dto::request::devices::{CreateDeviceRequest, UpdateDeviceRequest};
use crate::error::AppResult;
use crate::server::state::AppState;

/// List all devices.
///
/// - Requires a valid JWT.
#[utoipa::path(
    get,
    path = "/api/v3/devices",
    responses((status = 200, description = "List devices", body = [DeviceResponse])),
    tag = "Devices",
    security(("jwt" = []))
)]
pub async fn list_devices(State(state): State<AppState>) -> AppResult<Json<Vec<DeviceResponse>>> {
    let items = crate::service::devices::list_all(&state).await?;
    Ok(Json(items))
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
pub async fn get_device(Path(id): Path<u32>, State(state): State<AppState>) -> AppResult<Json<DeviceResponse>> {
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
pub async fn create_device(State(state): State<AppState>, Json(req): Json<CreateDeviceRequest>) -> AppResult<(axum::http::StatusCode, Json<DeviceResponse>)> {
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
pub async fn update_device(Path(id): Path<u32>, State(state): State<AppState>, Json(req): Json<UpdateDeviceRequest>) -> AppResult<Json<DeviceResponse>> {
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
pub async fn delete_device(Path(id): Path<u32>, State(state): State<AppState>) -> AppResult<axum::http::StatusCode> {
    crate::service::devices::delete(&state, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
