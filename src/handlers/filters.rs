use crate::dto::request::CreateFilterRequest;
use crate::dto::response::FilterResponse;
use crate::error::AppResult;
use crate::server::state::AppState;
use axum::{
    extract::{Path, State},
    Json,
};
use serde::Deserialize;

/// List saved event filters.
///
/// - Supports server-side scheduled filters; this lists definitions (not executions).
/// - Requires a valid JWT.
#[utoipa::path(
    get,
    path = "/api/v3/filters",
    responses((status = 200, description = "List filters", body = serde_json::Value)),
    tag = "Filters",
    security(("jwt" = []))
)]
pub async fn list_filters(State(state): State<AppState>) -> AppResult<Json<Vec<FilterResponse>>> {
    let items = crate::service::filters::list_all(&state).await?;
    Ok(Json(items))
}

/// Get a single filter definition by id.
///
/// - Requires a valid JWT.
#[utoipa::path(
    get,
    path = "/api/v3/filters/{id}",
    params(("id" = u32, Path, description = "Filter ID")),
    responses((status = 200, description = "Filter details", body = serde_json::Value)),
    tag = "Filters",
    security(("jwt" = []))
)]
pub async fn get_filter(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<Json<FilterResponse>> {
    let item = crate::service::filters::get_by_id(&state, id).await?;
    Ok(Json(item))
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateFilterRequest {
    pub name: Option<String>,
    pub query: Option<String>,
}

/// Update filter name or JSON query definition.
///
/// - Requires a valid JWT; validates JSON shape at the application level.
#[utoipa::path(
    put,
    path = "/api/v3/filters/{id}",
    params(("id" = u32, Path, description = "Filter ID")),
    request_body = UpdateFilterRequest,
    responses((status = 200, description = "Updated filter", body = serde_json::Value)),
    tag = "Filters",
    security(("jwt" = []))
)]
pub async fn update_filter(
    Path(id): Path<u32>,
    State(state): State<AppState>,
    Json(req): Json<UpdateFilterRequest>,
) -> AppResult<Json<FilterResponse>> {
    let item = crate::service::filters::update(&state, id, req.name, req.query).await?;
    Ok(Json(item))
}

/// Create a new filter definition.
///
/// - `query_json` should be a valid filter expression understood by ZoneMinder.
/// - Requires a valid JWT.
#[utoipa::path(
    post,
    path = "/api/v3/filters",
    request_body = CreateFilterRequest,
    responses((status = 201, description = "Created filter", body = FilterResponse)),
    tag = "Filters",
    security(("jwt" = []))
)]
pub async fn create_filter(
    State(state): State<AppState>,
    Json(req): Json<CreateFilterRequest>,
) -> AppResult<(axum::http::StatusCode, Json<FilterResponse>)> {
    let item = crate::service::filters::create(&state, req).await?;
    Ok((axum::http::StatusCode::CREATED, Json(item)))
}

/// Delete a filter by id.
///
/// - Responds 204 on success, 404 if not found.
/// - Requires a valid JWT.
#[utoipa::path(
    delete,
    path = "/api/v3/filters/{id}",
    params(("id" = u32, Path, description = "Filter ID")),
    responses((status = 204, description = "Deleted filter")),
    tag = "Filters",
    security(("jwt" = []))
)]
pub async fn delete_filter(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<axum::http::StatusCode> {
    crate::service::filters::delete(&state, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
