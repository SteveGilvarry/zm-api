use crate::dto::request::montage_layouts::{
    CreateMontageLayoutRequest, UpdateMontageLayoutRequest,
};
use crate::dto::response::MontageLayoutResponse;
use crate::error::AppResult;
use crate::server::state::AppState;
use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct MontageLayoutQuery {
    pub user_id: Option<u32>,
}

/// List montage layouts; optionally filter by user id.
///
/// - Requires a valid JWT.
#[utoipa::path(
    get,
    path = "/api/v3/montage_layouts",
    params(("user_id" = Option<u32>, Query, description = "Filter by user")),
    responses((status = 200, description = "List montage layouts", body = [MontageLayoutResponse])),
    tag = "Montage Layouts",
    security(("jwt" = []))
)]
pub async fn list_montage_layouts(
    State(state): State<AppState>,
    Query(q): Query<MontageLayoutQuery>,
) -> AppResult<Json<Vec<MontageLayoutResponse>>> {
    let items = crate::service::montage_layouts::list_all(&state, q.user_id).await?;
    Ok(Json(items))
}

/// Get a montage layout by id.
///
/// - Requires a valid JWT; responds 404 if not found.
#[utoipa::path(
    get,
    path = "/api/v3/montage_layouts/{id}",
    params(("id" = u32, Path, description = "Montage Layout ID")),
    responses((status = 200, description = "Montage layout detail", body = MontageLayoutResponse)),
    tag = "Montage Layouts",
    security(("jwt" = []))
)]
pub async fn get_montage_layout(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<Json<MontageLayoutResponse>> {
    let item = crate::service::montage_layouts::get_by_id(&state, id).await?;
    Ok(Json(item))
}

/// Create a new montage layout entry.
///
/// - Requires a valid JWT.
#[utoipa::path(
    post,
    path = "/api/v3/montage_layouts",
    request_body = CreateMontageLayoutRequest,
    responses((status = 201, description = "Created montage layout", body = MontageLayoutResponse)),
    tag = "Montage Layouts",
    security(("jwt" = []))
)]
pub async fn create_montage_layout(
    State(state): State<AppState>,
    Json(req): Json<CreateMontageLayoutRequest>,
) -> AppResult<(axum::http::StatusCode, Json<MontageLayoutResponse>)> {
    let item = crate::service::montage_layouts::create(&state, req).await?;
    Ok((axum::http::StatusCode::CREATED, Json(item)))
}

/// Update a montage layout entry.
///
/// - Partial update.
/// - Requires a valid JWT.
#[utoipa::path(
    patch,
    path = "/api/v3/montage_layouts/{id}",
    params(("id" = u32, Path, description = "Montage Layout ID")),
    request_body = UpdateMontageLayoutRequest,
    responses((status = 200, description = "Updated montage layout", body = MontageLayoutResponse)),
    tag = "Montage Layouts",
    security(("jwt" = []))
)]
pub async fn update_montage_layout(
    Path(id): Path<u32>,
    State(state): State<AppState>,
    Json(req): Json<UpdateMontageLayoutRequest>,
) -> AppResult<Json<MontageLayoutResponse>> {
    let item = crate::service::montage_layouts::update(&state, id, req).await?;
    Ok(Json(item))
}

/// Delete a montage layout by id.
///
/// - Responds 204 on success, 404 if not found.
/// - Requires a valid JWT.
#[utoipa::path(
    delete,
    path = "/api/v3/montage_layouts/{id}",
    params(("id" = u32, Path, description = "Montage Layout ID")),
    responses((status = 204, description = "Deleted montage layout")),
    tag = "Montage Layouts",
    security(("jwt" = []))
)]
pub async fn delete_montage_layout(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<axum::http::StatusCode> {
    crate::service::montage_layouts::delete(&state, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
