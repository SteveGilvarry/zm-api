use crate::dto::request::snapshots_events::CreateSnapshotEventRequest;
use crate::dto::response::snapshots_events::PaginatedSnapshotEventsResponse;
use crate::dto::response::SnapshotEventResponse;
use crate::dto::PaginationParams;
use crate::error::AppResult;
use crate::server::state::AppState;
use axum::{
    extract::{Path, Query, State},
    Json,
};

/// List all snapshot-event associations with pagination.
#[utoipa::path(
    get,
    path = "/api/v3/snapshots-events",
    params(
        ("page" = Option<u64>, Query, description = "Page number (1-indexed)", example = 1),
        ("page_size" = Option<u64>, Query, description = "Items per page (max 1000)", example = 25)
    ),
    responses((status = 200, description = "Paginated list of snapshot-event associations", body = PaginatedSnapshotEventsResponse)),
    tag = "Snapshots Events",
    security(("jwt" = []))
)]
pub async fn list_snapshot_events(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> AppResult<Json<PaginatedSnapshotEventsResponse>> {
    let result = crate::service::snapshots_events::list_paginated(&state, &params).await?;
    Ok(Json(PaginatedSnapshotEventsResponse::from(result)))
}

/// Get a snapshot-event association by id.
#[utoipa::path(
    get,
    path = "/api/v3/snapshots-events/{id}",
    params(("id" = u32, Path, description = "Association ID")),
    responses((status = 200, description = "Snapshot-event association detail", body = SnapshotEventResponse)),
    tag = "Snapshots Events",
    security(("jwt" = []))
)]
pub async fn get_snapshot_event(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<Json<SnapshotEventResponse>> {
    let item = crate::service::snapshots_events::get_by_id(&state, id).await?;
    Ok(Json(item))
}

/// Create a new snapshot-event association.
#[utoipa::path(
    post,
    path = "/api/v3/snapshots-events",
    request_body = CreateSnapshotEventRequest,
    responses((status = 201, description = "Created snapshot-event association", body = SnapshotEventResponse)),
    tag = "Snapshots Events",
    security(("jwt" = []))
)]
pub async fn create_snapshot_event(
    State(state): State<AppState>,
    Json(req): Json<CreateSnapshotEventRequest>,
) -> AppResult<(axum::http::StatusCode, Json<SnapshotEventResponse>)> {
    let item = crate::service::snapshots_events::create(&state, req).await?;
    Ok((axum::http::StatusCode::CREATED, Json(item)))
}

/// Delete a snapshot-event association by id.
#[utoipa::path(
    delete,
    path = "/api/v3/snapshots-events/{id}",
    params(("id" = u32, Path, description = "Association ID")),
    responses((status = 204, description = "Deleted snapshot-event association")),
    tag = "Snapshots Events",
    security(("jwt" = []))
)]
pub async fn delete_snapshot_event(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<axum::http::StatusCode> {
    crate::service::snapshots_events::delete(&state, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
