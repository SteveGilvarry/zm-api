use crate::dto::request::filter_ast::FilterQuery;
use crate::dto::request::{CreateFilterRequest, UpdateFilterRequest};
use crate::dto::response::events::PaginatedEventsResponse;
use crate::dto::response::filters::PaginatedFiltersResponse;
use crate::dto::response::FilterResponse;
use crate::dto::PaginationParams;
use crate::error::AppResult;
use crate::server::state::AppState;
use crate::service::monitor_acl::MonitorScope;
use crate::util::claim::UserClaims;
use axum::{
    extract::{Path, Query, State},
    Json,
};

/// List saved event filters with pagination.
///
/// - Supports server-side scheduled filters; this lists definitions (not executions).
/// - Requires a valid JWT.
#[utoipa::path(
    get,
    path = "/api/v3/filters",
    params(
        ("page" = Option<u64>, Query, description = "Page number (1-indexed)", example = 1),
        ("page_size" = Option<u64>, Query, description = "Items per page (max 1000)", example = 25)
    ),
    responses((status = 200, description = "Paginated list of filters", body = PaginatedFiltersResponse)),
    tag = "Filters",
    security(("jwt" = []))
)]
pub async fn list_filters(
    State(state): State<AppState>,
    claims: UserClaims,
    Query(params): Query<PaginationParams>,
) -> AppResult<Json<PaginatedFiltersResponse>> {
    let result = crate::service::filters::list_paginated(&state, &params, &claims).await?;
    Ok(Json(PaginatedFiltersResponse::from(result)))
}

/// Get a single filter definition by id.
///
/// - Requires a valid JWT.
#[utoipa::path(
    get,
    path = "/api/v3/filters/{id}",
    params(("id" = u32, Path, description = "Filter ID")),
    responses((status = 200, description = "Filter details", body = FilterResponse)),
    tag = "Filters",
    security(("jwt" = []))
)]
pub async fn get_filter(
    Path(id): Path<u32>,
    State(state): State<AppState>,
    claims: UserClaims,
) -> AppResult<Json<FilterResponse>> {
    let item = crate::service::filters::get_by_id(&state, id, &claims).await?;
    Ok(Json(item))
}

/// Update any subset of a filter's fields.
///
/// - Requires a valid JWT; only fields present in the body are changed.
#[utoipa::path(
    put,
    path = "/api/v3/filters/{id}",
    params(("id" = u32, Path, description = "Filter ID")),
    request_body = UpdateFilterRequest,
    responses((status = 200, description = "Updated filter", body = FilterResponse)),
    tag = "Filters",
    security(("jwt" = []))
)]
pub async fn update_filter(
    Path(id): Path<u32>,
    State(state): State<AppState>,
    claims: UserClaims,
    Json(req): Json<UpdateFilterRequest>,
) -> AppResult<Json<FilterResponse>> {
    let item = crate::service::filters::update(&state, id, &req, &claims).await?;
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
    claims: UserClaims,
    Json(req): Json<CreateFilterRequest>,
) -> AppResult<(axum::http::StatusCode, Json<FilterResponse>)> {
    let item = crate::service::filters::create(&state, req, &claims).await?;
    Ok((axum::http::StatusCode::CREATED, Json(item)))
}

/// Preview a structured filter: run it now and return the matching events.
///
/// - Body is the structured filter AST (`FilterQuery`); `page`/`page_size` are
///   query params. The predicate is compiled to a parameterised query (no SQL
///   is built from client strings) and row-level monitor ACL is applied.
/// - Requires a valid JWT.
#[utoipa::path(
    post,
    path = "/api/v3/filters/preview",
    params(
        ("page" = Option<u64>, Query, description = "Page number (1-indexed)", example = 1),
        ("page_size" = Option<u64>, Query, description = "Items per page (max 1000)", example = 25)
    ),
    request_body = FilterQuery,
    responses((status = 200, description = "Events matching the filter", body = PaginatedEventsResponse)),
    tag = "Filters",
    security(("jwt" = []))
)]
pub async fn preview_filter(
    State(state): State<AppState>,
    scope: MonitorScope,
    Query(params): Query<PaginationParams>,
    Json(query): Json<FilterQuery>,
) -> AppResult<Json<PaginatedEventsResponse>> {
    let result = crate::service::filters::preview(&state, &query, &params, &scope).await?;
    Ok(Json(result))
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
    claims: UserClaims,
) -> AppResult<axum::http::StatusCode> {
    crate::service::filters::delete(&state, id, &claims).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
