use crate::dto::request::object_types::{CreateObjectTypeRequest, UpdateObjectTypeRequest};
use crate::dto::response::ObjectTypeResponse;
use crate::error::AppResult;
use crate::server::state::AppState;
use axum::{
    extract::{Path, State},
    Json,
};

/// List all object types.
#[utoipa::path(
    get,
    path = "/api/v3/object-types",
    responses((status = 200, description = "List object types", body = [ObjectTypeResponse])),
    tag = "Object Types",
    security(("jwt" = []))
)]
pub async fn list_object_types(
    State(state): State<AppState>,
) -> AppResult<Json<Vec<ObjectTypeResponse>>> {
    let items = crate::service::object_types::list_all(&state).await?;
    Ok(Json(items))
}

/// Get an object type by id.
#[utoipa::path(
    get,
    path = "/api/v3/object-types/{id}",
    params(("id" = i32, Path, description = "Object Type ID")),
    responses((status = 200, description = "Object type detail", body = ObjectTypeResponse)),
    tag = "Object Types",
    security(("jwt" = []))
)]
pub async fn get_object_type(
    Path(id): Path<i32>,
    State(state): State<AppState>,
) -> AppResult<Json<ObjectTypeResponse>> {
    let item = crate::service::object_types::get_by_id(&state, id).await?;
    Ok(Json(item))
}

/// Create a new object type.
#[utoipa::path(
    post,
    path = "/api/v3/object-types",
    request_body = CreateObjectTypeRequest,
    responses((status = 201, description = "Created object type", body = ObjectTypeResponse)),
    tag = "Object Types",
    security(("jwt" = []))
)]
pub async fn create_object_type(
    State(state): State<AppState>,
    Json(req): Json<CreateObjectTypeRequest>,
) -> AppResult<(axum::http::StatusCode, Json<ObjectTypeResponse>)> {
    let item = crate::service::object_types::create(&state, req).await?;
    Ok((axum::http::StatusCode::CREATED, Json(item)))
}

/// Update an object type.
#[utoipa::path(
    patch,
    path = "/api/v3/object-types/{id}",
    params(("id" = i32, Path, description = "Object Type ID")),
    request_body = UpdateObjectTypeRequest,
    responses((status = 200, description = "Updated object type", body = ObjectTypeResponse)),
    tag = "Object Types",
    security(("jwt" = []))
)]
pub async fn update_object_type(
    Path(id): Path<i32>,
    State(state): State<AppState>,
    Json(req): Json<UpdateObjectTypeRequest>,
) -> AppResult<Json<ObjectTypeResponse>> {
    let item = crate::service::object_types::update(&state, id, req).await?;
    Ok(Json(item))
}

/// Delete an object type by id.
#[utoipa::path(
    delete,
    path = "/api/v3/object-types/{id}",
    params(("id" = i32, Path, description = "Object Type ID")),
    responses((status = 204, description = "Deleted object type")),
    tag = "Object Types",
    security(("jwt" = []))
)]
pub async fn delete_object_type(
    Path(id): Path<i32>,
    State(state): State<AppState>,
) -> AppResult<axum::http::StatusCode> {
    crate::service::object_types::delete(&state, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
