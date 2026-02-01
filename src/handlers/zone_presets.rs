use crate::dto::request::CreateZonePresetRequest;
use crate::dto::response::zone_presets::PaginatedZonePresetsResponse;
use crate::dto::response::ZonePresetResponse;
use crate::dto::PaginationParams;
use crate::error::AppResult;
use crate::server::state::AppState;
use axum::{
    extract::{Path, Query, State},
    Json,
};

/// List available zone presets which define reusable zone parameters.
///
/// - Useful for applying consistent detection thresholds across monitors.
/// - Requires a valid JWT.
#[utoipa::path(
    get,
    path = "/api/v3/zone-presets",
    params(
        ("page" = Option<u64>, Query, description = "Page number (1-indexed)", example = 1),
        ("page_size" = Option<u64>, Query, description = "Items per page (max 1000)", example = 25)
    ),
    responses((status = 200, description = "Paginated list of zone presets", body = PaginatedZonePresetsResponse)),
    tag = "Zones",
    security(("jwt" = []))
)]
pub async fn list_zone_presets(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> AppResult<Json<PaginatedZonePresetsResponse>> {
    let result = crate::service::zone_presets::list_paginated(&state, &params).await?;
    Ok(Json(PaginatedZonePresetsResponse::from(result)))
}

/// Get a single zone preset by id.
///
/// - Requires a valid JWT; responds 404 if not found.
#[utoipa::path(
    get,
    path = "/api/v3/zone-presets/{id}",
    params(("id" = u32, Path, description = "Zone preset ID")),
    responses((status = 200, description = "Zone preset detail", body = ZonePresetResponse)),
    tag = "Zones",
    security(("jwt" = []))
)]
pub async fn get_zone_preset(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<Json<ZonePresetResponse>> {
    let item = crate::service::zone_presets::get_by_id(&state, id).await?;
    Ok(Json(item))
}

/// Create a new zone preset.
///
/// - Accepts string variants for `type`, `units`, and `check_method` which are converted to enums.
/// - Requires a valid JWT.
#[utoipa::path(
    post,
    path = "/api/v3/zone-presets",
    request_body = CreateZonePresetRequest,
    responses((status = 201, description = "Created zone preset", body = ZonePresetResponse)),
    tag = "Zones",
    security(("jwt" = []))
)]
pub async fn create_zone_preset(
    State(state): State<AppState>,
    Json(req): Json<CreateZonePresetRequest>,
) -> AppResult<(axum::http::StatusCode, Json<ZonePresetResponse>)> {
    let item = crate::service::zone_presets::create(&state, req).await?;
    Ok((axum::http::StatusCode::CREATED, Json(item)))
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct UpdateZonePresetRequest {
    pub name: Option<String>,
    pub r#type: Option<String>,
    pub units: Option<String>,
    pub check_method: Option<String>,
}

/// Update fields of an existing zone preset (partial update).
///
/// - Fields are optional and applied when present.
/// - Requires a valid JWT; responds 404 if not found.
#[utoipa::path(
    patch,
    path = "/api/v3/zone-presets/{id}",
    params(("id" = u32, Path, description = "Zone preset ID")),
    request_body = UpdateZonePresetRequest,
    responses((status = 200, description = "Updated zone preset", body = ZonePresetResponse)),
    tag = "Zones",
    security(("jwt" = []))
)]
pub async fn update_zone_preset(
    Path(id): Path<u32>,
    State(state): State<AppState>,
    Json(req): Json<UpdateZonePresetRequest>,
) -> AppResult<Json<ZonePresetResponse>> {
    let item = crate::service::zone_presets::update(
        &state,
        id,
        req.name,
        req.r#type,
        req.units,
        req.check_method,
    )
    .await?;
    Ok(Json(item))
}

/// Delete a zone preset by id.
///
/// - Responds 204 on success, 404 if not found.
/// - Requires a valid JWT.
#[utoipa::path(
    delete,
    path = "/api/v3/zone-presets/{id}",
    params(("id" = u32, Path, description = "Zone preset ID")),
    responses((status = 204, description = "Deleted zone preset")),
    tag = "Zones",
    security(("jwt" = []))
)]
pub async fn delete_zone_preset(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<axum::http::StatusCode> {
    crate::service::zone_presets::delete(&state, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
