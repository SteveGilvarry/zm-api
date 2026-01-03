use axum::extract::{Path, State};
use axum::Json;
use tracing::{info, warn};

use crate::dto::response::{MessageResponse, VersionResponse};
use crate::error::AppResult;
use crate::server::state::AppState;
use crate::service;

// Health check.
#[utoipa::path(
    get,
    path = "/api/v3/server/health_check",
    responses(
        (status = 200, description = "check service is up", body = MessageResponse)
    ),
    tag = "Server"
)]
pub async fn health_check() -> AppResult<Json<MessageResponse>> {
    Ok(Json(MessageResponse::new("Ok")))
}

#[utoipa::path(
    get,
    path = "/api/v3/host/getVersion",
    responses(
        (status = 200, description = "Get ZoneMinder and API version information", body = VersionResponse),
        (status = 500, description = "Internal server error", body = MessageResponse)
    ),
    tag = "Server"
)]
pub async fn get_version(State(state): State<AppState>) -> AppResult<Json<VersionResponse>> {
    info!("Handling request to get ZoneMinder version information");
    match service::server::get_version(&state).await {
        Ok(version_info) => {
            info!("Successfully retrieved version information");
            Ok(Json(version_info))
        }
        Err(e) => {
            warn!("Failed to get version information: {:?}", e);
            Err(e)
        }
    }
}

/// Control ZoneMinder daemon state (restart, stop, start)
#[utoipa::path(
    post,
    path = "/api/v3/states/change/{action}",
    params(
        ("action" = String, Path, description = "Action to perform: restart, stop, or start")
    ),
    responses(
        (status = 200, description = "State changed successfully", body = MessageResponse),
        (status = 400, description = "Invalid action", body = MessageResponse),
        (status = 500, description = "Failed to change state", body = MessageResponse)
    ),
    tag = "Server",
    summary = "Control ZoneMinder daemon state",
    description = "- Changes the ZoneMinder system state (restart/stop/start).\n- Requires a valid JWT with admin permissions.",
    security(("jwt" = []))
)]
pub async fn change_state(
    State(state): State<AppState>,
    Path(action): Path<String>,
) -> AppResult<Json<MessageResponse>> {
    info!("Handling request to change ZoneMinder state: {}", action);

    let message = match action.to_lowercase().as_str() {
        "restart" => {
            service::server::restart_zoneminder(&state).await?;
            "ZoneMinder restarted successfully"
        }
        "stop" => {
            service::server::stop_zoneminder(&state).await?;
            "ZoneMinder stopped successfully"
        }
        "start" => {
            service::server::start_zoneminder(&state).await?;
            "ZoneMinder started successfully"
        }
        _ => {
            return Err(crate::error::AppError::BadRequestError(format!(
                "Invalid action '{}'. Valid actions are: restart, stop, start",
                action
            )));
        }
    };

    Ok(Json(MessageResponse::new(message)))
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_check_handler() {
        assert_eq!(health_check().await.unwrap().0.message(), "Ok");
    }
}
