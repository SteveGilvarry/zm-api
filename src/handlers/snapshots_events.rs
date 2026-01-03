use crate::dto::request::snapshots_events::CreateSnapshotEventRequest;
use crate::dto::response::SnapshotEventResponse;
use crate::error::AppResult;
use crate::server::state::AppState;
use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SnapshotEventQuery {
    snapshot_id: Option<u32>,
    event_id: Option<u64>,
}

/// List all snapshot-event associations.
#[utoipa::path(
    get,
    path = "/api/v3/snapshots-events",
    params(
        ("snapshot_id" = Option<u32>, Query, description = "Filter by snapshot ID"),
        ("event_id" = Option<u64>, Query, description = "Filter by event ID")
    ),
    responses((status = 200, description = "List snapshot-event associations", body = [SnapshotEventResponse])),
    tag = "Snapshots Events",
    security(("jwt" = []))
)]
pub async fn list_snapshot_events(
    Query(params): Query<SnapshotEventQuery>,
    State(state): State<AppState>,
) -> AppResult<Json<Vec<SnapshotEventResponse>>> {
    let items =
        crate::service::snapshots_events::list_all(&state, params.snapshot_id, params.event_id)
            .await?;
    Ok(Json(items))
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
