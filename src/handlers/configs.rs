use axum::{
    extract::{Path, Query, State},
    Json,
};
use tracing::info;

use crate::dto::request::config::ConfigQueryParams;
use crate::dto::response::config::{CategoryCountResponse, PaginatedConfigsResponse};
use crate::dto::{request::config::UpdateConfigRequest, response::config::ConfigResponse};
use crate::error::AppResult;
use crate::server::state::AppState;
use crate::service;

/// List all configuration key/value entries from the ZoneMinder `Config` table.
///
/// - Requires a valid JWT.
/// - Returns a paginated set of config entries including metadata such as type, category,
///   default value and readonly flags.
#[utoipa::path(
    get,
    path = "/api/v3/configs",
    params(
        ("page" = Option<u64>, Query, description = "Page number (1-indexed)", example = 1),
        ("page_size" = Option<u64>, Query, description = "Items per page (max 1000)", example = 25),
        ("category" = Option<String>, Query, description = "Exact match on Category column"),
        ("search" = Option<String>, Query, description = "Substring match on Name column")
    ),
    responses(
        (status = 200, description = "Paginated list of config entries", body = PaginatedConfigsResponse),
        (status = 401, description = "Unauthorized", body = crate::error::AppResponseError),
        (status = 500, description = "Internal server error", body = crate::error::AppResponseError)
    ),
    security(("jwt" = [])),
    tag = "Config"
)]
pub async fn list_configs(
    State(state): State<AppState>,
    Query(params): Query<ConfigQueryParams>,
) -> AppResult<Json<PaginatedConfigsResponse>> {
    info!("Listing config entries with pagination");
    let result = service::config::list_filtered(&state, &params).await?;
    Ok(Json(PaginatedConfigsResponse::from(result)))
}

/// List distinct config categories with their entry counts.
///
/// - Requires a valid JWT.
/// - Returns an array of category names with the number of config entries in each.
#[utoipa::path(
    get,
    path = "/api/v3/configs/categories",
    responses(
        (status = 200, description = "List of categories with counts", body = Vec<CategoryCountResponse>),
        (status = 401, description = "Unauthorized", body = crate::error::AppResponseError),
        (status = 500, description = "Internal server error", body = crate::error::AppResponseError)
    ),
    security(("jwt" = [])),
    tag = "Config"
)]
pub async fn list_categories(
    State(state): State<AppState>,
) -> AppResult<Json<Vec<CategoryCountResponse>>> {
    info!("Listing config categories");
    let categories = service::config::list_categories(&state).await?;
    Ok(Json(categories))
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
