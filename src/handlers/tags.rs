use axum::{extract::{Path, State}, Json};
use crate::dto::response::TagResponse;
use crate::dto::request::tags::{CreateTagRequest, UpdateTagRequest};
use crate::error::AppResult;
use crate::server::state::AppState;

/// List all tags.
///
/// - Requires a valid JWT.
#[utoipa::path(
    get,
    path = "/api/v3/tags",
    responses((status = 200, description = "List tags", body = [TagResponse])),
    tag = "Tags",
    security(("jwt" = []))
)]
pub async fn list_tags(State(state): State<AppState>) -> AppResult<Json<Vec<TagResponse>>> {
    let items = crate::service::tags::list_all(&state).await?;
    Ok(Json(items))
}

/// Get a tag by id.
///
/// - Requires a valid JWT; responds 404 if not found.
#[utoipa::path(
    get,
    path = "/api/v3/tags/{id}",
    params(("id" = u64, Path, description = "Tag ID")),
    responses((status = 200, description = "Tag detail", body = TagResponse)),
    tag = "Tags",
    security(("jwt" = []))
)]
pub async fn get_tag(Path(id): Path<u64>, State(state): State<AppState>) -> AppResult<Json<TagResponse>> {
    let item = crate::service::tags::get_by_id(&state, id).await?;
    Ok(Json(item))
}

/// Create a new tag entry.
///
/// - Requires a valid JWT.
/// - Name must be unique.
#[utoipa::path(
    post,
    path = "/api/v3/tags",
    request_body = CreateTagRequest,
    responses((status = 201, description = "Created tag", body = TagResponse)),
    tag = "Tags",
    security(("jwt" = []))
)]
pub async fn create_tag(State(state): State<AppState>, Json(req): Json<CreateTagRequest>) -> AppResult<(axum::http::StatusCode, Json<TagResponse>)> {
    let item = crate::service::tags::create(&state, req).await?;
    Ok((axum::http::StatusCode::CREATED, Json(item)))
}

/// Update a tag entry.
///
/// - Partial update.
/// - Requires a valid JWT.
#[utoipa::path(
    patch,
    path = "/api/v3/tags/{id}",
    params(("id" = u64, Path, description = "Tag ID")),
    request_body = UpdateTagRequest,
    responses((status = 200, description = "Updated tag", body = TagResponse)),
    tag = "Tags",
    security(("jwt" = []))
)]
pub async fn update_tag(Path(id): Path<u64>, State(state): State<AppState>, Json(req): Json<UpdateTagRequest>) -> AppResult<Json<TagResponse>> {
    let item = crate::service::tags::update(&state, id, req).await?;
    Ok(Json(item))
}

/// Delete a tag by id.
///
/// - Responds 204 on success, 404 if not found.
/// - Requires a valid JWT.
#[utoipa::path(
    delete,
    path = "/api/v3/tags/{id}",
    params(("id" = u64, Path, description = "Tag ID")),
    responses((status = 204, description = "Deleted tag")),
    tag = "Tags",
    security(("jwt" = []))
)]
pub async fn delete_tag(Path(id): Path<u64>, State(state): State<AppState>) -> AppResult<axum::http::StatusCode> {
    crate::service::tags::delete(&state, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
