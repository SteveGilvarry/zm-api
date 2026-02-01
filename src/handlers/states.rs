use crate::dto::request::states::{CreateStateRequest, UpdateStateRequest};
use crate::dto::response::states::PaginatedStatesResponse;
use crate::dto::response::StateResponse;
use crate::dto::PaginationParams;
use crate::error::AppResult;
use crate::server::state::AppState;
use axum::{
    extract::{Path, Query, State},
    Json,
};

/// List all states with pagination.
///
/// - Requires a valid JWT.
#[utoipa::path(
    get,
    path = "/api/v3/states",
    params(
        ("page" = Option<u64>, Query, description = "Page number (1-indexed)", example = 1),
        ("page_size" = Option<u64>, Query, description = "Items per page (max 1000)", example = 25)
    ),
    responses((status = 200, description = "Paginated list of states", body = PaginatedStatesResponse)),
    tag = "States",
    security(("jwt" = []))
)]
pub async fn list_states(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> AppResult<Json<PaginatedStatesResponse>> {
    let result = crate::service::states::list_paginated(&state, &params).await?;
    Ok(Json(PaginatedStatesResponse::from(result)))
}

/// Get a state by id.
///
/// - Requires a valid JWT; responds 404 if not found.
#[utoipa::path(
    get,
    path = "/api/v3/states/{id}",
    params(("id" = u32, Path, description = "State ID")),
    responses((status = 200, description = "State detail", body = StateResponse)),
    tag = "States",
    security(("jwt" = []))
)]
pub async fn get_state(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<Json<StateResponse>> {
    let item = crate::service::states::get_by_id(&state, id).await?;
    Ok(Json(item))
}

/// Create a new state entry.
///
/// - Requires a valid JWT.
/// - Name must be unique.
#[utoipa::path(
    post,
    path = "/api/v3/states",
    request_body = CreateStateRequest,
    responses((status = 201, description = "Created state", body = StateResponse)),
    tag = "States",
    security(("jwt" = []))
)]
pub async fn create_state(
    State(state): State<AppState>,
    Json(req): Json<CreateStateRequest>,
) -> AppResult<(axum::http::StatusCode, Json<StateResponse>)> {
    let item = crate::service::states::create(&state, req).await?;
    Ok((axum::http::StatusCode::CREATED, Json(item)))
}

/// Update a state entry.
///
/// - Partial update.
/// - Requires a valid JWT.
#[utoipa::path(
    patch,
    path = "/api/v3/states/{id}",
    params(("id" = u32, Path, description = "State ID")),
    request_body = UpdateStateRequest,
    responses((status = 200, description = "Updated state", body = StateResponse)),
    tag = "States",
    security(("jwt" = []))
)]
pub async fn update_state(
    Path(id): Path<u32>,
    State(state): State<AppState>,
    Json(req): Json<UpdateStateRequest>,
) -> AppResult<Json<StateResponse>> {
    let item = crate::service::states::update(&state, id, req).await?;
    Ok(Json(item))
}

/// Delete a state by id.
///
/// - Responds 204 on success, 404 if not found.
/// - Requires a valid JWT.
#[utoipa::path(
    delete,
    path = "/api/v3/states/{id}",
    params(("id" = u32, Path, description = "State ID")),
    responses((status = 204, description = "Deleted state")),
    tag = "States",
    security(("jwt" = []))
)]
pub async fn delete_state(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<axum::http::StatusCode> {
    crate::service::states::delete(&state, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
