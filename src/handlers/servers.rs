use crate::dto::request::CreateServerRequest;
use crate::dto::response::ServerResponse;
use crate::error::AppResult;
use crate::server::state::AppState;
use axum::{
    extract::{Path, State},
    Json,
};
use serde::Deserialize;

/// List registered ZoneMinder servers.
///
/// - Requires a valid JWT.
#[utoipa::path(
    get,
    path = "/api/v3/servers",
    responses((status = 200, description = "List servers", body = serde_json::Value)),
    tag = "Servers",
    security(("jwt" = []))
)]
pub async fn list_servers(State(state): State<AppState>) -> AppResult<Json<Vec<ServerResponse>>> {
    let items = crate::service::servers::list_all(&state).await?;
    Ok(Json(items))
}

/// Get a server by id.
///
/// - Requires a valid JWT; responds 404 if not found.
#[utoipa::path(
    get,
    path = "/api/v3/servers/{id}",
    params(("id" = u32, Path, description = "Server ID")),
    responses((status = 200, description = "Server detail", body = serde_json::Value)),
    tag = "Servers",
    security(("jwt" = []))
)]
pub async fn get_server(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<Json<ServerResponse>> {
    let item = crate::service::servers::get_by_id(&state, id).await?;
    Ok(Json(item))
}

/// Register a new server record.
///
/// - Accepts optional hostname/port/status; status is provided as a string and mapped to enum.
/// - Requires a valid JWT.
#[utoipa::path(
    post,
    path = "/api/v3/servers",
    request_body = CreateServerRequest,
    responses((status = 201, description = "Created server", body = ServerResponse)),
    tag = "Servers",
    security(("jwt" = []))
)]
pub async fn create_server(
    State(state): State<AppState>,
    Json(req): Json<CreateServerRequest>,
) -> AppResult<(axum::http::StatusCode, Json<ServerResponse>)> {
    let item = crate::service::servers::create(&state, req).await?;
    Ok((axum::http::StatusCode::CREATED, Json(item)))
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateServerRequest {
    pub name: Option<String>,
    pub hostname: Option<String>,
    pub port: Option<u32>,
    pub status: Option<String>,
}

/// Update server fields (partial update).
///
/// - Applies provided fields; status string is mapped to the DB enum.
/// - Requires a valid JWT.
#[utoipa::path(
    patch,
    path = "/api/v3/servers/{id}",
    params(("id" = u32, Path, description = "Server ID")),
    request_body = UpdateServerRequest,
    responses((status = 200, description = "Updated server", body = ServerResponse)),
    tag = "Servers",
    security(("jwt" = []))
)]
pub async fn update_server(
    Path(id): Path<u32>,
    State(state): State<AppState>,
    Json(req): Json<UpdateServerRequest>,
) -> AppResult<Json<ServerResponse>> {
    let item =
        crate::service::servers::update(&state, id, req.name, req.hostname, req.port, req.status)
            .await?;
    Ok(Json(item))
}

/// Delete a server by id.
///
/// - Responds 204 on success, 404 if not found.
/// - Requires a valid JWT.
#[utoipa::path(
    delete,
    path = "/api/v3/servers/{id}",
    params(("id" = u32, Path, description = "Server ID")),
    responses((status = 204, description = "Deleted server")),
    tag = "Servers",
    security(("jwt" = []))
)]
pub async fn delete_server(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> AppResult<axum::http::StatusCode> {
    crate::service::servers::delete(&state, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
