use axum::{extract::{Path, State}, Json};
use crate::dto::response::ServerStatResponse;
use crate::dto::request::server_stats::CreateServerStatRequest;
use crate::error::AppResult;
use crate::server::state::AppState;

/// List all server stats.
#[utoipa::path(
    get,
    path = "/api/v3/server-stats",
    responses((status = 200, description = "List server stats", body = [ServerStatResponse])),
    tag = "Server Stats",
    security(("jwt" = []))
)]
pub async fn list_server_stats(State(state): State<AppState>) -> AppResult<Json<Vec<ServerStatResponse>>> {
    let items = crate::service::server_stats::list_all(&state).await?;
    Ok(Json(items))
}

/// Get a server stat by id.
#[utoipa::path(
    get,
    path = "/api/v3/server-stats/{id}",
    params(("id" = u32, Path, description = "Server Stat ID")),
    responses((status = 200, description = "Server stat detail", body = ServerStatResponse)),
    tag = "Server Stats",
    security(("jwt" = []))
)]
pub async fn get_server_stat(Path(id): Path<u32>, State(state): State<AppState>) -> AppResult<Json<ServerStatResponse>> {
    let item = crate::service::server_stats::get_by_id(&state, id).await?;
    Ok(Json(item))
}

/// Create a new server stat entry.
#[utoipa::path(
    post,
    path = "/api/v3/server-stats",
    request_body = CreateServerStatRequest,
    responses((status = 201, description = "Created server stat", body = ServerStatResponse)),
    tag = "Server Stats",
    security(("jwt" = []))
)]
pub async fn create_server_stat(State(state): State<AppState>, Json(req): Json<CreateServerStatRequest>) -> AppResult<(axum::http::StatusCode, Json<ServerStatResponse>)> {
    let item = crate::service::server_stats::create(&state, req).await?;
    Ok((axum::http::StatusCode::CREATED, Json(item)))
}

/// Delete a server stat by id.
#[utoipa::path(
    delete,
    path = "/api/v3/server-stats/{id}",
    params(("id" = u32, Path, description = "Server Stat ID")),
    responses((status = 204, description = "Deleted server stat")),
    tag = "Server Stats",
    security(("jwt" = []))
)]
pub async fn delete_server_stat(Path(id): Path<u32>, State(state): State<AppState>) -> AppResult<axum::http::StatusCode> {
    crate::service::server_stats::delete(&state, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
