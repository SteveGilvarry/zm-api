//! HTTP handlers for a monitor's zm-next processing graph (the "free graph").
//!
//! `GET` returns the stored graph (monitor view access required). `PUT`/`DELETE`
//! replace/remove it and require **unrestricted** monitor access (reconfiguring
//! a monitor's AI pipeline is admin-equivalent, mirroring `create_monitor`).

use axum::extract::{Path, State};
use axum::Json;
use serde_json::Value;
use tracing::{info, warn};

use crate::dto::response::monitor_pipeline::MonitorPipelineResponse;
use crate::error::{AppError, AppResponseError, AppResult};
use crate::server::state::AppState;
use crate::service;
use crate::service::monitor_acl::MonitorScope;

#[utoipa::path(
    get,
    path = "/api/v3/monitors/{id}/pipeline",
    params(("id" = u32, Path, description = "Monitor identifier")),
    responses(
        (status = 200, description = "The monitor's stored processing graph", body = MonitorPipelineResponse),
        (status = 401, description = "Unauthorized", body = AppResponseError),
        (status = 404, description = "Monitor or stored pipeline not found", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(("jwt" = [])),
    tag = "Monitors"
)]
pub async fn get_monitor_pipeline(
    State(state): State<AppState>,
    Path(id): Path<u32>,
    scope: MonitorScope,
) -> AppResult<Json<MonitorPipelineResponse>> {
    info!("Viewing zm-next pipeline graph for monitor {id}.");
    Ok(Json(
        service::monitor_pipeline::get(&state, id, &scope).await?,
    ))
}

#[utoipa::path(
    put,
    path = "/api/v3/monitors/{id}/pipeline",
    params(("id" = u32, Path, description = "Monitor identifier")),
    responses(
        (status = 200, description = "Graph replaced", body = MonitorPipelineResponse),
        (status = 400, description = "Invalid graph", body = AppResponseError),
        (status = 401, description = "Unauthorized", body = AppResponseError),
        (status = 403, description = "Caller's monitor access is restricted", body = AppResponseError),
        (status = 404, description = "Monitor not found", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(("jwt" = [])),
    tag = "Monitors"
)]
pub async fn put_monitor_pipeline(
    State(state): State<AppState>,
    Path(id): Path<u32>,
    scope: MonitorScope,
    Json(graph): Json<Value>,
) -> AppResult<Json<MonitorPipelineResponse>> {
    // Reconfiguring a monitor's AI pipeline is admin-equivalent (see create_monitor).
    if scope.is_restricted() {
        return Err(AppError::PermissionDeniedError(
            "editing a monitor pipeline requires unrestricted monitor access".to_string(),
        ));
    }
    info!("Replacing zm-next pipeline graph for monitor {id}.");
    match service::monitor_pipeline::replace(&state, id, graph, &scope).await {
        Ok(resp) => Ok(Json(resp)),
        Err(e) => {
            warn!("Failed to replace monitor {id} pipeline graph: {e:?}.");
            Err(e)
        }
    }
}

#[utoipa::path(
    delete,
    path = "/api/v3/monitors/{id}/pipeline",
    params(("id" = u32, Path, description = "Monitor identifier")),
    responses(
        (status = 200, description = "Graph removed (reverts to the default pipeline)"),
        (status = 401, description = "Unauthorized", body = AppResponseError),
        (status = 403, description = "Caller's monitor access is restricted", body = AppResponseError),
        (status = 404, description = "Monitor not found", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(("jwt" = [])),
    tag = "Monitors"
)]
pub async fn delete_monitor_pipeline(
    State(state): State<AppState>,
    Path(id): Path<u32>,
    scope: MonitorScope,
) -> AppResult<Json<()>> {
    if scope.is_restricted() {
        return Err(AppError::PermissionDeniedError(
            "deleting a monitor pipeline requires unrestricted monitor access".to_string(),
        ));
    }
    info!("Deleting zm-next pipeline graph for monitor {id}.");
    service::monitor_pipeline::delete(&state, id, &scope).await?;
    Ok(Json(()))
}

#[utoipa::path(
    post,
    path = "/api/v3/monitors/{id}/zmnext",
    params(("id" = u32, Path, description = "Monitor identifier")),
    responses(
        (status = 200, description = "Monitor switched to zm-next; default graph materialized", body = MonitorPipelineResponse),
        (status = 400, description = "UseZmNext column missing (fork migration required)", body = AppResponseError),
        (status = 401, description = "Unauthorized", body = AppResponseError),
        (status = 403, description = "Caller's monitor access is restricted", body = AppResponseError),
        (status = 404, description = "Monitor not found", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(("jwt" = [])),
    tag = "Monitors"
)]
pub async fn enable_monitor_zmnext(
    State(state): State<AppState>,
    Path(id): Path<u32>,
    scope: MonitorScope,
) -> AppResult<Json<MonitorPipelineResponse>> {
    if scope.is_restricted() {
        return Err(AppError::PermissionDeniedError(
            "switching a monitor to zm-next requires unrestricted monitor access".to_string(),
        ));
    }
    info!("Switching monitor {id} to zm-next.");
    match service::monitor_pipeline::enable_zmnext(&state, id, &scope).await {
        Ok(resp) => Ok(Json(resp)),
        Err(e) => {
            warn!("Failed to switch monitor {id} to zm-next: {e:?}.");
            Err(e)
        }
    }
}

#[utoipa::path(
    delete,
    path = "/api/v3/monitors/{id}/zmnext",
    params(("id" = u32, Path, description = "Monitor identifier")),
    responses(
        (status = 200, description = "Monitor reverted to legacy zmc/zma (graph kept dormant)"),
        (status = 400, description = "UseZmNext column missing", body = AppResponseError),
        (status = 401, description = "Unauthorized", body = AppResponseError),
        (status = 403, description = "Caller's monitor access is restricted", body = AppResponseError),
        (status = 404, description = "Monitor not found", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(("jwt" = [])),
    tag = "Monitors"
)]
pub async fn disable_monitor_zmnext(
    State(state): State<AppState>,
    Path(id): Path<u32>,
    scope: MonitorScope,
) -> AppResult<Json<()>> {
    if scope.is_restricted() {
        return Err(AppError::PermissionDeniedError(
            "reverting a monitor from zm-next requires unrestricted monitor access".to_string(),
        ));
    }
    info!("Reverting monitor {id} from zm-next.");
    service::monitor_pipeline::disable_zmnext(&state, id, &scope).await?;
    Ok(Json(()))
}
