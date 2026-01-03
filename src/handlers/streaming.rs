use crate::dto::response::StreamEndpoints;
use crate::service::streaming;
use crate::{
    error::{AppResponseError, AppResult},
    server::state::AppState,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use tracing::info;

/// Register a ZoneMinder stream in go2rtc by pushing an RTSP source
#[utoipa::path(
    put,
    path = "/api/v3/streams/{id}",
    operation_id = "registerStream",
    tag = "Streaming",
    params(
        ("id" = u32, Path, description = "Monitor ID")
    ),
    responses(
        (status = 200, description = "Stream registered successfully", body = StreamEndpoints),
        (status = 400, description = "Bad request", body = AppResponseError),
        (status = 401, description = "Unauthorized", body = AppResponseError),
        (status = 503, description = "go2rtc service unavailable", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn register_stream(
    State(state): State<AppState>,
    Path(id): Path<u32>,
) -> AppResult<Json<StreamEndpoints>> {
    info!("Registering stream for monitor ID: {}", id);

    let endpoints = streaming::register_monitor_stream(&state, id).await?;

    // Convert to DTO response format
    let response = StreamEndpoints {
        webrtc: endpoints.webrtc_url,
        webrtc_api: endpoints.webrtc_api_url,
        hls: endpoints.hls_url,
        mjpeg: endpoints.mjpeg_url,
    };

    Ok(Json(response))
}

/// Return the streaming endpoints for a given stream name
#[utoipa::path(
    get,
    path = "/api/v3/streams/{id}",
    operation_id = "getStream",
    tag = "Streaming",
    params(
        ("id" = u32, Path, description = "Monitor ID")
    ),
    responses(
        (status = 200, description = "Stream endpoints retrieved successfully", body = StreamEndpoints),
        (status = 404, description = "Stream not found", body = AppResponseError),
        (status = 401, description = "Unauthorized", body = AppResponseError),
        (status = 503, description = "go2rtc service unavailable", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn get_stream(
    State(state): State<AppState>,
    Path(id): Path<u32>,
) -> AppResult<Json<StreamEndpoints>> {
    info!("Getting stream endpoints for monitor ID: {}", id);

    let endpoints = streaming::get_stream_endpoints(&state, id).await?;

    // Convert to DTO response format
    let response = StreamEndpoints {
        webrtc: endpoints.webrtc_url,
        webrtc_api: endpoints.webrtc_api_url,
        hls: endpoints.hls_url,
        mjpeg: endpoints.mjpeg_url,
    };

    Ok(Json(response))
}

/// Delete a stream registration from go2rtc
#[utoipa::path(
    delete,
    path = "/api/v3/streams/{id}",
    operation_id = "deleteStream",
    tag = "Streaming",
    params(
        ("id" = u32, Path, description = "Monitor ID")
    ),
    responses(
        (status = 200, description = "Stream deleted successfully", body = String),
        (status = 404, description = "Stream not found", body = AppResponseError),
        (status = 401, description = "Unauthorized", body = AppResponseError),
        (status = 503, description = "go2rtc service unavailable", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn delete_stream(
    State(state): State<AppState>,
    Path(id): Path<u32>,
) -> AppResult<(StatusCode, &'static str)> {
    info!("Deleting stream for monitor ID: {}", id);

    streaming::delete_stream(&state, id).await?;

    info!("Successfully deleted stream for monitor ID: {}", id);
    Ok((StatusCode::OK, "deleted"))
}
