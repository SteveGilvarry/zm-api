use crate::dto::response::StreamEndpoints;
use crate::service;
use crate::{
    error::{AppError, AppResponseError, AppResult},
    server::state::AppState,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use reqwest::Client;
use tracing::{info, warn};

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

    // Get monitor details from database
    let monitor_details = service::monitor::get_streaming_details(&state, id).await?;

    // Build rtsp URL without adding the ID at the end
    let rtsp_url = format!(
        "rtsp://{}:{}@{}:{}",
        monitor_details.user, monitor_details.pass, monitor_details.host, monitor_details.port
    );

    // Stream name in go2rtc
    let stream_name = format!("zm{}", id);

    // Call go2rtc API - according to spec: PUT /api/streams?src={source_url}&name={stream_name}
    let client = Client::default();
    let go2rtc_server = "http://192.168.0.35:1984"; // TODO: Move to configuration

    // Try without URL encoding the RTSP URL
    let url = format!(
        "{}/api/streams?src={}&name={}",
        go2rtc_server, rtsp_url, stream_name
    );

    // Debug print the full URL - will show in the logs
    println!("DEBUG - FULL URL: {}", url);

    info!("Calling go2rtc API to register stream: {}", url);
    let resp = client.put(&url).send().await.map_err(|e| {
        warn!("Failed to call go2rtc API: {:?}", e);
        AppError::HttpClientError(e)
    })?;

    if resp.status().is_success() {
        info!("Successfully registered stream for monitor ID: {}", id);

        // Create stream endpoints according to go2rtc API spec
        let endpoints = StreamEndpoints {
            webrtc: format!("{}/webrtc.html?src={}", go2rtc_server, stream_name),
            webrtc_api: format!("{}/api/webrtc?src={}", go2rtc_server, stream_name),
            hls: format!("{}/api/stream.m3u8?src={}", go2rtc_server, stream_name),
            mjpeg: format!("{}/api/stream.mjpeg?src={}", go2rtc_server, stream_name),
        };

        Ok(Json(endpoints))
    } else {
        warn!("go2rtc API returned error status: {}", resp.status());
        Err(AppError::BadRequestError(format!(
            "go2rtc returned {}",
            resp.status()
        )))
    }
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
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn get_stream(
    State(_state): State<AppState>,
    Path(id): Path<u32>,
) -> AppResult<Json<StreamEndpoints>> {
    let stream_name = format!("zm{}", id);
    let go2rtc_server = "http://192.168.0.35:1984"; // TODO: Move to configuration

    info!("Getting stream endpoints for monitor ID: {}", id);

    // First check if the stream exists in go2rtc
    let client = Client::default();
    let api_url = format!("{}/api/streams", go2rtc_server);

    info!("Checking if stream exists in go2rtc");
    let resp = client.get(&api_url).send().await.map_err(|e| {
        warn!("Failed to call go2rtc API: {:?}", e);
        AppError::HttpClientError(e)
    })?;

    if !resp.status().is_success() {
        warn!("go2rtc API returned error status: {}", resp.status());
        return Err(AppError::BadRequestError(format!(
            "go2rtc returned {}",
            resp.status()
        )));
    }

    // Parse the response to see if our stream exists
    let streams: serde_json::Value = resp.json().await.map_err(|e| {
        warn!("Failed to parse go2rtc response: {:?}", e);
        AppError::HttpClientError(e)
    })?;

    // Check if our stream is in the list
    if !streams
        .as_object()
        .is_some_and(|obj| obj.contains_key(&stream_name))
    {
        warn!("Stream '{}' not found in go2rtc", stream_name);
        return Err(AppError::NotFoundError(crate::error::Resource {
            details: vec![("id".to_string(), id.to_string())],
            resource_type: crate::error::ResourceType::Monitor,
        }));
    }

    // Stream exists, return the endpoints with correct URLs according to go2rtc API spec
    let endpoints = StreamEndpoints {
        webrtc: format!("{}/webrtc.html?src={}", go2rtc_server, stream_name),
        webrtc_api: format!("{}/api/webrtc?src={}", go2rtc_server, stream_name),
        hls: format!("{}/api/stream.m3u8?src={}", go2rtc_server, stream_name),
        mjpeg: format!("{}/api/stream.mjpeg?src={}", go2rtc_server, stream_name),
    };

    Ok(Json(endpoints))
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
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn delete_stream(
    State(_state): State<AppState>,
    Path(id): Path<u32>,
) -> AppResult<(StatusCode, &'static str)> {
    let stream_name = format!("zm{}", id);
    let go2rtc_server = "http://192.168.0.35:1984"; // TODO: Move to configuration

    // According to go2rtc spec, DELETE /api/streams?src={stream_name} (not name)
    let url = format!("{}/api/streams?src={}", go2rtc_server, stream_name);

    info!("Deleting stream for monitor ID: {} with URL: {}", id, url);
    let client = Client::default();
    let resp = client.delete(&url).send().await.map_err(|e| {
        warn!("Failed to call go2rtc API: {:?}", e);
        AppError::HttpClientError(e)
    })?;

    if resp.status().is_success() {
        info!("Successfully deleted stream for monitor ID: {}", id);
        Ok((StatusCode::OK, "deleted"))
    } else {
        warn!("go2rtc API returned error status: {}", resp.status());
        Err(AppError::BadRequestError(format!(
            "go2rtc returned {}",
            resp.status()
        )))
    }
}
