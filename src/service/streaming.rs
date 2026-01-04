//! Streaming service layer for managing go2rtc stream registration and lifecycle
//!
//! This service orchestrates:
//! - Stream registration with go2rtc
//! - Stream availability checking
//! - Stream lifecycle management
//! - Integration with ZoneMinder monitor credentials

use tracing::{debug, error, info};

use crate::client::go2rtc::{Go2RtcClient, Go2RtcError, Go2RtcStream, StreamEndpoints};
use crate::error::{AppError, AppResult, Resource, ResourceType};
use crate::server::state::AppState;
use crate::service::monitor as monitor_service;

/// Initialize a Go2RtcClient from the application configuration
///
/// This helper function creates a Go2RtcClient instance using the streaming
/// configuration from AppState. It's used internally by service functions.
///
/// # Arguments
/// * `state` - The application state containing configuration
///
/// # Returns
/// * `AppResult<Go2RtcClient>` - Configured client or error
fn get_go2rtc_client(state: &AppState) -> AppResult<Go2RtcClient> {
    let config = &state.config.streaming.go2rtc;

    if !config.enabled {
        return Err(AppError::ServiceUnavailableError(
            "go2rtc streaming is disabled in configuration".to_string(),
        ));
    }

    Ok(Go2RtcClient::new(
        &config.base_url,
        config.timeout_seconds,
        config.retry_attempts,
    ))
}

/// Build an RTSP URL from monitor streaming details
///
/// Constructs a complete RTSP URL with credentials for a monitor.
/// Format: rtsp://user:pass@host:port/path
///
/// # Arguments
/// * `monitor_id` - ID of the monitor
/// * `user` - RTSP username
/// * `pass` - RTSP password
/// * `host` - RTSP server host
/// * `port` - RTSP server port
///
/// # Returns
/// * `String` - Complete RTSP URL with credentials
fn build_rtsp_url(monitor_id: u32, user: &str, pass: &str, host: &str, port: u16) -> String {
    // URL encode credentials to handle special characters
    let encoded_user = urlencoding::encode(user);
    let encoded_pass = urlencoding::encode(pass);

    // Build RTSP URL with credentials
    // Note: This assumes a default path structure. In production, you may want
    // to make the path configurable or fetch it from monitor configuration.
    format!(
        "rtsp://{}:{}@{}:{}/zm/monitor/{}",
        encoded_user, encoded_pass, host, port, monitor_id
    )
}

/// Generate stream name for a monitor
///
/// Creates a consistent stream name for use with go2rtc.
/// Format: "zm{monitor_id}"
///
/// # Arguments
/// * `monitor_id` - ID of the monitor
///
/// # Returns
/// * `String` - Stream name
fn get_stream_name(monitor_id: u32) -> String {
    format!("zm{}", monitor_id)
}

/// Register a ZoneMinder monitor stream with go2rtc
///
/// This function:
/// 1. Fetches monitor RTSP credentials from the database
/// 2. Builds the RTSP URL
/// 3. Registers the stream with go2rtc
/// 4. Returns streaming endpoints
///
/// # Arguments
/// * `state` - Application state for database and configuration access
/// * `monitor_id` - ID of the monitor to register
///
/// # Returns
/// * `Ok(StreamEndpoints)` - URLs for accessing the stream via different protocols
/// * `Err(AppError)` - If registration fails
///
/// # Example
/// ```ignore
/// let endpoints = register_monitor_stream(&state, 1).await?;
/// println!("WebRTC URL: {}", endpoints.webrtc_url);
/// ```
pub async fn register_monitor_stream(
    state: &AppState,
    monitor_id: u32,
) -> AppResult<StreamEndpoints> {
    info!("Registering stream for monitor {}", monitor_id);

    // Get go2rtc client
    let client = get_go2rtc_client(state)?;

    // Fetch monitor streaming details (credentials, host, port)
    let monitor_details = monitor_service::get_streaming_details(state, monitor_id).await?;

    // Build RTSP URL with credentials
    let rtsp_url = build_rtsp_url(
        monitor_id,
        &monitor_details.user,
        &monitor_details.pass,
        &monitor_details.host,
        monitor_details.port,
    );

    let stream_name = get_stream_name(monitor_id);

    // Register stream with go2rtc
    let endpoints = client
        .register_stream(&stream_name, &rtsp_url)
        .await
        .map_err(|e| map_go2rtc_error(e, monitor_id))?;

    info!(
        "Successfully registered stream for monitor {} as '{}'",
        monitor_id, stream_name
    );

    Ok(endpoints)
}

/// Get streaming endpoints for a monitor
///
/// Returns the streaming endpoints for an already-registered monitor stream.
/// If the stream is not registered, this function will return an error.
/// Use `ensure_stream_ready` if you want automatic registration.
///
/// # Arguments
/// * `state` - Application state
/// * `monitor_id` - ID of the monitor
///
/// # Returns
/// * `Ok(StreamEndpoints)` - Stream endpoints if registered
/// * `Err(AppError::NotFoundError)` - If stream is not registered
pub async fn get_stream_endpoints(state: &AppState, monitor_id: u32) -> AppResult<StreamEndpoints> {
    debug!("Getting stream endpoints for monitor {}", monitor_id);

    let client = get_go2rtc_client(state)?;
    let stream_name = get_stream_name(monitor_id);

    // Check if stream exists
    let stream_info = client
        .get_stream(&stream_name)
        .await
        .map_err(|e| map_go2rtc_error(e, monitor_id))?;

    match stream_info {
        Some(_) => {
            // Stream exists, build endpoints
            let endpoints = client.build_endpoints(&stream_name);
            Ok(endpoints)
        }
        None => Err(AppError::NotFoundError(Resource {
            details: vec![
                ("monitor_id".to_string(), monitor_id.to_string()),
                ("stream_name".to_string(), stream_name),
            ],
            resource_type: ResourceType::Monitor,
        })),
    }
}

/// Delete a stream registration from go2rtc
///
/// Removes a monitor's stream from go2rtc. This does not affect the underlying
/// ZoneMinder monitor or RTSP source.
///
/// # Arguments
/// * `state` - Application state
/// * `monitor_id` - ID of the monitor whose stream should be deleted
///
/// # Returns
/// * `Ok(())` - Stream deleted successfully
/// * `Err(AppError)` - If deletion fails
pub async fn delete_stream(state: &AppState, monitor_id: u32) -> AppResult<()> {
    info!("Deleting stream for monitor {}", monitor_id);

    let client = get_go2rtc_client(state)?;
    let stream_name = get_stream_name(monitor_id);

    client
        .delete_stream(&stream_name)
        .await
        .map_err(|e| map_go2rtc_error(e, monitor_id))?;

    info!(
        "Successfully deleted stream for monitor {} (stream: '{}')",
        monitor_id, stream_name
    );

    Ok(())
}

/// Response for list_active_streams
#[derive(Debug, Clone, serde::Serialize)]
pub struct ActiveStreamInfo {
    pub monitor_id: u32,
    pub stream_name: String,
    pub stream_info: Go2RtcStream,
}

/// List all active streams registered with go2rtc
///
/// Returns information about all streams currently managed by go2rtc,
/// filtering for ZoneMinder monitor streams (those matching "zm*" pattern).
///
/// # Arguments
/// * `state` - Application state
///
/// # Returns
/// * `Ok(Vec<ActiveStreamInfo>)` - List of active stream information
/// * `Err(AppError)` - If listing fails
pub async fn list_active_streams(state: &AppState) -> AppResult<Vec<ActiveStreamInfo>> {
    debug!("Listing all active streams");

    let client = get_go2rtc_client(state)?;

    let all_streams = client
        .list_streams()
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to list streams: {}", e)))?;

    // Filter for ZoneMinder streams (those starting with "zm")
    // and extract monitor IDs
    let active_streams: Vec<ActiveStreamInfo> = all_streams
        .into_iter()
        .filter_map(|(stream_name, stream_info)| {
            if stream_name.starts_with("zm") {
                // Extract monitor ID from stream name "zm{id}"
                stream_name[2..]
                    .parse::<u32>()
                    .ok()
                    .map(|monitor_id| ActiveStreamInfo {
                        monitor_id,
                        stream_name,
                        stream_info,
                    })
            } else {
                None
            }
        })
        .collect();

    info!("Found {} active ZoneMinder streams", active_streams.len());

    Ok(active_streams)
}

/// Ensure a stream is ready for use
///
/// Checks if a stream is registered with go2rtc. If not registered,
/// automatically registers it. This is useful for on-demand stream setup.
///
/// # Arguments
/// * `state` - Application state
/// * `monitor_id` - ID of the monitor
///
/// # Returns
/// * `Ok(StreamEndpoints)` - Stream endpoints (newly registered or existing)
/// * `Err(AppError)` - If ensuring stream readiness fails
///
/// # Example
/// ```ignore
/// // This will register the stream if needed, or return existing endpoints
/// let endpoints = ensure_stream_ready(&state, 1).await?;
/// ```
pub async fn ensure_stream_ready(state: &AppState, monitor_id: u32) -> AppResult<StreamEndpoints> {
    debug!("Ensuring stream is ready for monitor {}", monitor_id);

    let client = get_go2rtc_client(state)?;
    let stream_name = get_stream_name(monitor_id);

    // Check if stream already exists
    match client.get_stream(&stream_name).await {
        Ok(Some(_)) => {
            info!("Stream already registered for monitor {}", monitor_id);
            // Stream exists, build and return endpoints
            Ok(client.build_endpoints(&stream_name))
        }
        Ok(None) => {
            info!(
                "Stream not registered for monitor {}, registering now",
                monitor_id
            );
            // Stream doesn't exist, register it
            register_monitor_stream(state, monitor_id).await
        }
        Err(e) => {
            // Error checking stream status
            error!(
                "Failed to check stream status for monitor {}: {}",
                monitor_id, e
            );
            Err(map_go2rtc_error(e, monitor_id))
        }
    }
}

/// Map Go2RtcError to AppError with context
///
/// Internal helper to convert go2rtc client errors to application errors
/// with appropriate context and error types.
///
/// # Arguments
/// * `error` - The go2rtc error
/// * `monitor_id` - Monitor ID for context
///
/// # Returns
/// * `AppError` - Mapped application error
fn map_go2rtc_error(error: Go2RtcError, monitor_id: u32) -> AppError {
    match error {
        Go2RtcError::StreamNotFound(stream_name) => AppError::NotFoundError(Resource {
            details: vec![
                ("monitor_id".to_string(), monitor_id.to_string()),
                ("stream_name".to_string(), stream_name),
            ],
            resource_type: ResourceType::Monitor,
        }),
        Go2RtcError::ApiError { status, message } => {
            if status >= 500 {
                AppError::ServiceUnavailableError(format!(
                    "go2rtc server error ({}): {}",
                    status, message
                ))
            } else if status == 404 {
                AppError::NotFoundError(Resource {
                    details: vec![("monitor_id".to_string(), monitor_id.to_string())],
                    resource_type: ResourceType::Monitor,
                })
            } else {
                AppError::BadRequestError(format!("go2rtc error ({}): {}", status, message))
            }
        }
        Go2RtcError::ConnectionFailed { attempts } => AppError::ServiceUnavailableError(format!(
            "Failed to connect to go2rtc after {} attempts",
            attempts
        )),
        Go2RtcError::HttpError(e) => {
            AppError::ServiceUnavailableError(format!("go2rtc HTTP error: {}", e))
        }
        Go2RtcError::Timeout => {
            AppError::ServiceUnavailableError("go2rtc request timeout".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_rtsp_url() {
        let url = build_rtsp_url(1, "admin", "password123", "192.168.1.100", 554);
        assert_eq!(
            url,
            "rtsp://admin:password123@192.168.1.100:554/zm/monitor/1"
        );
    }

    #[test]
    fn test_build_rtsp_url_with_special_chars() {
        let url = build_rtsp_url(2, "user@domain", "p@ss:word!", "camera.local", 8554);
        // Should URL encode special characters in credentials
        assert!(url.contains("user%40domain"));
        assert!(url.contains("p%40ss%3Aword%21"));
        assert!(url.contains("camera.local:8554"));
    }

    #[test]
    fn test_get_stream_name() {
        assert_eq!(get_stream_name(1), "zm1");
        assert_eq!(get_stream_name(42), "zm42");
        assert_eq!(get_stream_name(1000), "zm1000");
    }
}
