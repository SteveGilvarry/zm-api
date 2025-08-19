use axum::extract::State;
use axum::Json;
use tracing::{info, warn};

use crate::error::AppResult;
use crate::server::state::AppState;
use crate::dto::*;
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
        },
        Err(e) => {
            warn!("Failed to get version information: {:?}", e);
            Err(e)
        }
    }
}

#[cfg(test)]
pub mod tests {
  use super::*;

  #[tokio::test]
  async fn test_health_check_handler() {
    assert_eq!(health_check().await.unwrap().0.message(), "Ok");
  }
}
