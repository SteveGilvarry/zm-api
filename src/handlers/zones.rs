use crate::dto::request::CreateZoneRequest;
use crate::dto::response::zones::PaginatedZonesResponse;
use crate::dto::response::ZoneResponse;
use crate::dto::PaginationParams;
use crate::error::AppResult;
use crate::server::state::AppState;
use axum::{
    extract::{Path, Query, State},
    Json,
};

/// List detection zones configured for a given monitor with pagination.
///
/// - Path parameter `id` identifies the monitor.
/// - Requires a valid JWT.
/// - Returns the minimal shape of each zone (id, name, type, units, coords, etc.).
#[utoipa::path(
    get,
    path = "/api/v3/monitors/{id}/zones",
    params(
        ("id" = u32, Path, description = "Monitor ID"),
        ("page" = Option<u64>, Query, description = "Page number (1-indexed)", example = 1),
        ("page_size" = Option<u64>, Query, description = "Items per page (max 1000)", example = 25)
    ),
    responses(
        (status = 200, description = "Paginated zones for a monitor", body = PaginatedZonesResponse),
        (status = 401, description = "Unauthorized", body = crate::error::AppResponseError)
    ),
    tag = "Zones",
    security(("jwt" = []))
)]
pub async fn list_by_monitor(
    Path(id): Path<u32>,
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> AppResult<Json<PaginatedZonesResponse>> {
    let result = crate::service::zones::list_by_monitor_paginated(&state, id, &params).await?;
    Ok(Json(result))
}

/// Fetch a single detection zone by its identifier.
///
/// - Responds with 404 when the zone does not exist.
/// - Requires a valid JWT.
#[utoipa::path(
    get,
    path = "/api/v3/zones/{id}",
    params(("id" = u32, Path, description = "Zone ID")),
    responses(
        (status = 200, description = "Zone details", body = ZoneResponse),
        (status = 404, description = "Zone not found", body = crate::error::AppResponseError),
        (status = 401, description = "Unauthorized", body = crate::error::AppResponseError)
    ),
    tag = "Zones",
    security(("jwt" = []))
)]
pub async fn get(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<Json<ZoneResponse>> {
    let zone = crate::service::zones::get_by_id(&state, id).await?;
    Ok(Json(zone))
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct UpdateZoneRequest {
    pub name: Option<String>,
    pub polygon: Option<String>,
}

/// Update a zone's name and/or polygon geometry for an existing zone.
///
/// - Only `name` and `polygon` (coords) are supported for updates here.
/// - Requires a valid JWT.
#[utoipa::path(
    put,
    path = "/api/v3/zones/{id}",
    params(("id" = u32, Path, description = "Zone ID")),
    request_body = UpdateZoneRequest,
    responses(
        (status = 200, description = "Updated zone", body = ZoneResponse),
        (status = 401, description = "Unauthorized", body = crate::error::AppResponseError)
    ),
    tag = "Zones",
    security(("jwt" = []))
)]
pub async fn update(
    Path(id): Path<u32>,
    State(state): State<AppState>,
    Json(req): Json<UpdateZoneRequest>,
) -> AppResult<Json<ZoneResponse>> {
    let updated = crate::service::zones::update(&state, id, req.name, req.polygon).await?;
    Ok(Json(updated))
}

/// Create a new detection zone for the given monitor.
///
/// - Path parameter `id` identifies the monitor.
/// - `type`, `units`, and `coords` are expected as strings and converted to DB enums.
/// - Requires a valid JWT.
#[utoipa::path(
    post,
    path = "/api/v3/monitors/{id}/zones",
    params(("id" = u32, Path, description = "Monitor ID")),
    request_body = CreateZoneRequest,
    responses((status = 201, description = "Created zone", body = ZoneResponse)),
    tag = "Zones",
    security(("jwt" = []))
)]
pub async fn create(
    Path(id): Path<u32>,
    State(state): State<AppState>,
    Json(req): Json<CreateZoneRequest>,
) -> AppResult<(axum::http::StatusCode, Json<ZoneResponse>)> {
    let zone = crate::service::zones::create(&state, id, req).await?;
    Ok((axum::http::StatusCode::CREATED, Json(zone)))
}

/// Delete a detection zone by id.
///
/// - Responds with 204 on success, 404 if not found.
/// - Requires a valid JWT.
#[utoipa::path(
    delete,
    path = "/api/v3/zones/{id}",
    params(("id" = u32, Path, description = "Zone ID")),
    responses((status = 204, description = "Deleted zone")),
    tag = "Zones",
    security(("jwt" = []))
)]
pub async fn delete(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<axum::http::StatusCode> {
    crate::service::zones::delete(&state, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
