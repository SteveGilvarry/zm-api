use axum::{
    extract::{Path, State},
    Json,
};
use tracing::info;

use crate::dto::{request::config::UpdateConfigRequest, response::config::ConfigResponse};
use crate::error::AppResult;
use crate::server::state::AppState;
use crate::service;

/// List all configuration key/value entries from the ZoneMinder `Config` table.
///
/// - Requires a valid JWT.
/// - Returns the complete set of config entries including metadata such as type, category,
///   default value and readonly flags.
#[utoipa::path(
    get,
    path = "/api/v3/configs",
    responses(
        (status = 200, description = "List all config entries", body = Vec<ConfigResponse>),
        (status = 401, description = "Unauthorized", body = crate::error::AppResponseError),
        (status = 500, description = "Internal server error", body = crate::error::AppResponseError)
    ),
    security(("jwt" = [])),
    tag = "Config"
)]
pub async fn list_configs(State(state): State<AppState>) -> AppResult<Json<Vec<ConfigResponse>>> {
    info!("Listing all config entries");
    let list = service::config::list_all(&state).await?;
    Ok(Json(list))
}

/// Get a single configuration entry by its `name`.
///
/// - Requires a valid JWT.
/// - Responds with 404 if the entry does not exist.
#[utoipa::path(
    get,
    path = "/api/v3/configs/{name}",
    params(("name" = String, Path, description = "Config name")),
    responses(
        (status = 200, description = "Get config by name", body = ConfigResponse),
        (status = 404, description = "Config not found", body = crate::error::AppResponseError),
        (status = 401, description = "Unauthorized", body = crate::error::AppResponseError)
    ),
    security(("jwt" = [])),
    tag = "Config"
)]
pub async fn get_config(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> AppResult<Json<ConfigResponse>> {
    info!("Get config by name: {}", name);
    let item = service::config::get_by_name(&state, &name).await?;
    Ok(Json(item))
}

/// Update a configuration value by `name`.
///
/// - Requires a valid JWT.
/// - Fails with 403 when attempting to update a readonly config key.
/// - Responds with 404 if the entry does not exist.
#[utoipa::path(
    put,
    path = "/api/v3/configs/{name}",
    params(("name" = String, Path, description = "Config name")),
    request_body = UpdateConfigRequest,
    responses(
        (status = 200, description = "Updated config", body = ConfigResponse),
        (status = 403, description = "Config is read-only", body = crate::error::AppResponseError),
        (status = 404, description = "Config not found", body = crate::error::AppResponseError),
        (status = 401, description = "Unauthorized", body = crate::error::AppResponseError)
    ),
    security(("jwt" = [])),
    tag = "Config"
)]
pub async fn update_config(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(req): Json<UpdateConfigRequest>,
) -> AppResult<Json<ConfigResponse>> {
    info!("Update config {}", name);
    let updated = service::config::update_value(&state, &name, req).await?;
    Ok(Json(updated))
}
