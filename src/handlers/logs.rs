use axum::{extract::{Path, State}, Json};
use crate::dto::response::LogResponse;
use crate::error::AppResult;
use crate::server::state::AppState;

/// List recent log entries ordered by newest first.
///
/// - Defaults to a capped list (service uses a limit of 200 entries).
/// - Requires a valid JWT.
#[utoipa::path(
    get,
    path = "/api/v3/logs",
    responses((status = 200, description = "List logs", body = serde_json::Value)),
    tag = "Logs",
    security(("jwt" = []))
)]
pub async fn list_logs(State(state): State<AppState>) -> AppResult<Json<Vec<LogResponse>>> {
    let items = crate::service::logs::list_recent(&state, 200).await?;
    Ok(Json(items))
}

/// Get a single log entry by id.
///
/// - Requires a valid JWT.
#[utoipa::path(
    get,
    path = "/api/v3/logs/{id}",
    params(("id" = u32, Path, description = "Log ID")),
    responses((status = 200, description = "Log detail", body = serde_json::Value)),
    tag = "Logs",
    security(("jwt" = []))
)]
pub async fn get_log(Path(id): Path<u32>, State(state): State<AppState>) -> AppResult<Json<LogResponse>> {
    let item = crate::service::logs::get_by_id(&state, id).await?;
    Ok(Json(item))
}

// No POST for logs; logs are system-generated.
