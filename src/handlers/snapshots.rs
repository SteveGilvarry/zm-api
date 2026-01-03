use crate::dto::request::snapshots::{CreateSnapshotRequest, UpdateSnapshotRequest};
use crate::dto::response::SnapshotResponse;
use crate::error::AppResult;
use crate::server::state::AppState;
use axum::{
    extract::{Path, State},
    Json,
};

/// List all snapshots.
///
/// - Requires a valid JWT.
#[utoipa::path(
    get,
    path = "/api/v3/snapshots",
    responses((status = 200, description = "List snapshots", body = [SnapshotResponse])),
    tag = "Snapshots",
    security(("jwt" = []))
)]
pub async fn list_snapshots(
    State(state): State<AppState>,
) -> AppResult<Json<Vec<SnapshotResponse>>> {
    let items = crate::service::snapshots::list_all(&state).await?;
    Ok(Json(items))
}

/// Get a snapshot by id.
///
/// - Requires a valid JWT; responds 404 if not found.
#[utoipa::path(
    get,
    path = "/api/v3/snapshots/{id}",
    params(("id" = u32, Path, description = "Snapshot ID")),
    responses((status = 200, description = "Snapshot detail", body = SnapshotResponse)),
    tag = "Snapshots",
    security(("jwt" = []))
)]
pub async fn get_snapshot(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<Json<SnapshotResponse>> {
    let item = crate::service::snapshots::get_by_id(&state, id).await?;
    Ok(Json(item))
}

/// Create a new snapshot entry.
///
/// - Requires a valid JWT.
#[utoipa::path(
    post,
    path = "/api/v3/snapshots",
    request_body = CreateSnapshotRequest,
    responses((status = 201, description = "Created snapshot", body = SnapshotResponse)),
    tag = "Snapshots",
    security(("jwt" = []))
)]
pub async fn create_snapshot(
    State(state): State<AppState>,
    Json(req): Json<CreateSnapshotRequest>,
) -> AppResult<(axum::http::StatusCode, Json<SnapshotResponse>)> {
    let item = crate::service::snapshots::create(&state, req).await?;
    Ok((axum::http::StatusCode::CREATED, Json(item)))
}

/// Update a snapshot entry.
///
/// - Partial update.
/// - Requires a valid JWT.
#[utoipa::path(
    patch,
    path = "/api/v3/snapshots/{id}",
    params(("id" = u32, Path, description = "Snapshot ID")),
    request_body = UpdateSnapshotRequest,
    responses((status = 200, description = "Updated snapshot", body = SnapshotResponse)),
    tag = "Snapshots",
    security(("jwt" = []))
)]
pub async fn update_snapshot(
    Path(id): Path<u32>,
    State(state): State<AppState>,
    Json(req): Json<UpdateSnapshotRequest>,
) -> AppResult<Json<SnapshotResponse>> {
    let item = crate::service::snapshots::update(&state, id, req).await?;
    Ok(Json(item))
}

/// Delete a snapshot by id.
///
/// - Responds 204 on success, 404 if not found.
/// - Requires a valid JWT.
#[utoipa::path(
    delete,
    path = "/api/v3/snapshots/{id}",
    params(("id" = u32, Path, description = "Snapshot ID")),
    responses((status = 204, description = "Deleted snapshot")),
    tag = "Snapshots",
    security(("jwt" = []))
)]
pub async fn delete_snapshot(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<axum::http::StatusCode> {
    crate::service::snapshots::delete(&state, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
