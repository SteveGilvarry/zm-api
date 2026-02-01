//! Streaming service layer for managing go2rtc stream access and registration
//!
//! This service supports two modes:
//!
//! 1. **ZoneMinder go2rtc mode** (default): ZoneMinder 1.38+ has built-in go2rtc
//!    integration. In this mode, we query ZM's go2rtc for existing streams
//!    rather than registering our own.
//!
//! 2. **Direct RTSP mode**: For standalone go2rtc installations or when ZM's
//!    go2rtc doesn't have the stream, we can register the camera's RTSP URL
//!    directly with go2rtc.
//!
//! Configuration in `settings/*.toml`:
//! - `streaming.source.prefer_direct_rtsp`: If true, always use direct camera RTSP
//! - `streaming.go2rtc.auto_register`: If true, register streams that don't exist

use tracing::{debug, info, warn};

use crate::client::go2rtc::{Go2RtcClient, Go2RtcError, Go2RtcStream, StreamEndpoints};
use crate::error::{AppError, AppResult, Resource, ResourceType};
use crate::server::state::AppState;
use crate::service::monitor as monitor_service;

/// Stream naming conventions that ZoneMinder's go2rtc might use
const ZM_STREAM_NAME_PATTERNS: &[fn(u32) -> String] = &[
    // ZM 1.38 uses just the monitor ID as a string
    |id| id.to_string(),
    // Our convention: "zm{id}"
    |id| format!("zm{}", id),
    // Alternative: "monitor_{id}"
    |id| format!("monitor_{}", id),
];

/// Initialize a Go2RtcClient from the application configuration
#[allow(clippy::result_large_err)]
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

/// Build an RTSP URL with embedded credentials
///
/// Takes an existing RTSP URL and injects credentials if needed.
/// If the URL already contains credentials, returns it unchanged.
fn build_rtsp_url_with_credentials(rtsp_url: &str, user: &str, pass: &str) -> String {
    // If no credentials, return URL as-is
    if user.is_empty() && pass.is_empty() {
        return rtsp_url.to_string();
    }

    // Parse the URL to inject credentials
    match url::Url::parse(rtsp_url) {
        Ok(mut parsed) => {
            // Only inject credentials if URL doesn't already have them
            if parsed.username().is_empty() {
                let _ = parsed.set_username(user);
                let _ = parsed.set_password(Some(pass));
            }
            parsed.to_string()
        }
        Err(_) => {
            // If parsing fails, return original URL
            rtsp_url.to_string()
        }
    }
}

/// Get the primary stream name we use for registration
fn get_stream_name(monitor_id: u32) -> String {
    format!("zm{}", monitor_id)
}

/// Find an existing stream in go2rtc by trying multiple naming conventions
///
/// ZoneMinder's go2rtc may use different stream naming than we do.
/// This function tries several patterns to find an existing stream.
async fn find_existing_stream(
    client: &Go2RtcClient,
    monitor_id: u32,
) -> Result<Option<(String, Go2RtcStream)>, Go2RtcError> {
    for pattern_fn in ZM_STREAM_NAME_PATTERNS {
        let stream_name = pattern_fn(monitor_id);
        debug!("Checking for existing stream with name: {}", stream_name);

        match client.get_stream(&stream_name).await? {
            Some(stream_info) => {
                info!(
                    "Found existing stream for monitor {} as '{}'",
                    monitor_id, stream_name
                );
                return Ok(Some((stream_name, stream_info)));
            }
            None => continue,
        }
    }

    debug!("No existing stream found for monitor {}", monitor_id);
    Ok(None)
}

/// Get streaming endpoints for a monitor
///
/// This is the primary entry point for getting stream access. It handles:
/// 1. Checking if ZM's go2rtc already has the stream registered
/// 2. Auto-registering if configured and stream doesn't exist
/// 3. Respecting `prefer_direct_rtsp` configuration
///
/// # Arguments
/// * `state` - Application state
/// * `monitor_id` - ID of the monitor
///
/// # Returns
/// * `Ok(StreamEndpoints)` - URLs for WebRTC, HLS, etc.
/// * `Err(AppError)` - If stream access fails
pub async fn get_stream(state: &AppState, monitor_id: u32) -> AppResult<StreamEndpoints> {
    info!("Getting stream for monitor {}", monitor_id);

    let client = get_go2rtc_client(state)?;
    let source_config = &state.config.streaming.source;
    let go2rtc_config = &state.config.streaming.go2rtc;

    // Step 1: Check for existing stream (ZM's go2rtc may have it)
    // Skip this if prefer_direct_rtsp is true
    if !source_config.prefer_direct_rtsp {
        match find_existing_stream(&client, monitor_id).await {
            Ok(Some((stream_name, _stream_info))) => {
                info!(
                    "Using existing go2rtc stream '{}' for monitor {}",
                    stream_name, monitor_id
                );
                return Ok(client.build_endpoints(&stream_name));
            }
            Ok(None) => {
                debug!(
                    "No existing stream found for monitor {}, will try to register",
                    monitor_id
                );
            }
            Err(e) => {
                warn!(
                    "Error checking for existing stream for monitor {}: {}",
                    monitor_id, e
                );
                // Continue to try registration
            }
        }
    }

    // Step 2: Register stream if auto_register is enabled
    if go2rtc_config.auto_register {
        return register_direct_stream(state, monitor_id).await;
    }

    // Step 3: No stream found and auto_register is disabled
    Err(AppError::NotFoundError(Resource {
        details: vec![
            ("monitor_id".to_string(), monitor_id.to_string()),
            (
                "hint".to_string(),
                "Stream not found in go2rtc. Enable auto_register or check ZM go2rtc config"
                    .to_string(),
            ),
        ],
        resource_type: ResourceType::Monitor,
    }))
}

/// Register a stream using the camera's direct RTSP URL
///
/// This fetches the monitor's RTSP URL from the database and registers
/// it with go2rtc. Use this when:
/// - ZM's go2rtc doesn't have the stream
/// - You want to bypass ZM and connect directly to the camera
/// - Using a standalone go2rtc instance
///
/// # Arguments
/// * `state` - Application state
/// * `monitor_id` - ID of the monitor
///
/// # Returns
/// * `Ok(StreamEndpoints)` - URLs for the registered stream
pub async fn register_direct_stream(
    state: &AppState,
    monitor_id: u32,
) -> AppResult<StreamEndpoints> {
    info!("Registering direct RTSP stream for monitor {}", monitor_id);

    let client = get_go2rtc_client(state)?;

    // Fetch monitor streaming details
    let monitor_details = monitor_service::get_streaming_details(state, monitor_id).await?;

    // Get the actual RTSP URL from monitor configuration
    let base_rtsp_url = monitor_details.rtsp_url.ok_or_else(|| {
        AppError::BadRequestError(format!(
            "Monitor {} has no RTSP URL configured (Path field is empty)",
            monitor_id
        ))
    })?;

    // Build RTSP URL with credentials if needed
    let rtsp_url = build_rtsp_url_with_credentials(
        &base_rtsp_url,
        &monitor_details.user,
        &monitor_details.pass,
    );

    let stream_name = get_stream_name(monitor_id);

    debug!(
        "Registering stream '{}' with direct camera RTSP (credentials {})",
        stream_name,
        if monitor_details.user.is_empty() {
            "not provided"
        } else {
            "embedded"
        }
    );

    // Register stream with go2rtc
    let endpoints = client
        .register_stream(&stream_name, &rtsp_url)
        .await
        .map_err(|e| map_go2rtc_error(e, monitor_id))?;

    info!(
        "Successfully registered direct stream for monitor {} as '{}'",
        monitor_id, stream_name
    );

    Ok(endpoints)
}

/// Register a ZoneMinder monitor stream with go2rtc (legacy alias)
///
/// This is an alias for `register_direct_stream` for backward compatibility.
#[allow(dead_code)]
pub async fn register_monitor_stream(
    state: &AppState,
    monitor_id: u32,
) -> AppResult<StreamEndpoints> {
    register_direct_stream(state, monitor_id).await
}

/// Get streaming endpoints for an already-registered stream
///
/// Returns endpoints for a stream that's already in go2rtc.
/// Use `get_stream` for automatic stream discovery/registration.
pub async fn get_stream_endpoints(state: &AppState, monitor_id: u32) -> AppResult<StreamEndpoints> {
    debug!("Getting stream endpoints for monitor {}", monitor_id);

    let client = get_go2rtc_client(state)?;

    // Try to find existing stream with any naming convention
    match find_existing_stream(&client, monitor_id).await {
        Ok(Some((stream_name, _))) => Ok(client.build_endpoints(&stream_name)),
        Ok(None) => Err(AppError::NotFoundError(Resource {
            details: vec![("monitor_id".to_string(), monitor_id.to_string())],
            resource_type: ResourceType::Monitor,
        })),
        Err(e) => Err(map_go2rtc_error(e, monitor_id)),
    }
}

/// Delete a stream registration from go2rtc
///
/// Removes a monitor's stream from go2rtc. This only affects streams
/// we registered (with our naming convention), not ZM's native streams.
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
    pub monitor_id: Option<u32>,
    pub stream_name: String,
    pub stream_info: Go2RtcStream,
    /// Whether this stream was registered by us vs ZM's native go2rtc
    pub is_zm_native: bool,
}

/// List all active streams in go2rtc
///
/// Returns information about all streams, attempting to identify which
/// are ZoneMinder monitor streams vs other sources.
pub async fn list_active_streams(state: &AppState) -> AppResult<Vec<ActiveStreamInfo>> {
    debug!("Listing all active streams");

    let client = get_go2rtc_client(state)?;

    let all_streams = client
        .list_streams()
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to list streams: {}", e)))?;

    let active_streams: Vec<ActiveStreamInfo> = all_streams
        .into_iter()
        .map(|(stream_name, stream_info)| {
            // Try to extract monitor ID from various naming patterns
            let (monitor_id, is_zm_native) = parse_stream_name(&stream_name);

            ActiveStreamInfo {
                monitor_id,
                stream_name,
                stream_info,
                is_zm_native,
            }
        })
        .collect();

    info!("Found {} active streams in go2rtc", active_streams.len());

    Ok(active_streams)
}

/// Parse a stream name to extract monitor ID and determine if it's ZM-native
fn parse_stream_name(stream_name: &str) -> (Option<u32>, bool) {
    // Try "zm{id}" format (our convention)
    if let Some(id_str) = stream_name.strip_prefix("zm") {
        if let Ok(id) = id_str.parse::<u32>() {
            return (Some(id), false); // Our registration, not ZM native
        }
    }

    // Try "monitor_{id}" format
    if let Some(id_str) = stream_name.strip_prefix("monitor_") {
        if let Ok(id) = id_str.parse::<u32>() {
            return (Some(id), true); // Likely ZM native
        }
    }

    // Try plain numeric ID (ZM 1.38 native format)
    if let Ok(id) = stream_name.parse::<u32>() {
        return (Some(id), true); // ZM native
    }

    // Unknown format
    (None, false)
}

/// Ensure a stream is ready for use (legacy alias for get_stream)
pub async fn ensure_stream_ready(state: &AppState, monitor_id: u32) -> AppResult<StreamEndpoints> {
    get_stream(state, monitor_id).await
}

/// Map Go2RtcError to AppError with context
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
    fn test_build_rtsp_url_with_credentials() {
        let url = build_rtsp_url_with_credentials(
            "rtsp://192.168.1.100:554/stream1",
            "admin",
            "password123",
        );
        assert_eq!(url, "rtsp://admin:password123@192.168.1.100:554/stream1");
    }

    #[test]
    fn test_build_rtsp_url_no_credentials() {
        let url = build_rtsp_url_with_credentials("rtsp://camera.local:554/live", "", "");
        assert_eq!(url, "rtsp://camera.local:554/live");
    }

    #[test]
    fn test_build_rtsp_url_existing_credentials() {
        let url = build_rtsp_url_with_credentials(
            "rtsp://existing:creds@camera.local:554/live",
            "new_user",
            "new_pass",
        );
        // Should keep existing credentials
        assert!(url.contains("existing:creds@"));
    }

    #[test]
    fn test_build_rtsp_url_with_path() {
        let url = build_rtsp_url_with_credentials(
            "rtsp://192.168.1.50:554/cam/realmonitor?channel=1&subtype=0",
            "admin",
            "admin123",
        );
        assert!(url.starts_with("rtsp://admin:admin123@192.168.1.50:554/"));
        assert!(url.contains("realmonitor"));
    }

    #[test]
    fn test_get_stream_name() {
        assert_eq!(get_stream_name(1), "zm1");
        assert_eq!(get_stream_name(42), "zm42");
        assert_eq!(get_stream_name(1000), "zm1000");
    }

    #[test]
    fn test_parse_stream_name_zm_format() {
        let (id, native) = parse_stream_name("zm1");
        assert_eq!(id, Some(1));
        assert!(!native); // Our format, not ZM native

        let (id, native) = parse_stream_name("zm42");
        assert_eq!(id, Some(42));
        assert!(!native);
    }

    #[test]
    fn test_parse_stream_name_numeric() {
        // ZM 1.38 uses plain numeric IDs
        let (id, native) = parse_stream_name("1");
        assert_eq!(id, Some(1));
        assert!(native); // ZM native format

        let (id, native) = parse_stream_name("123");
        assert_eq!(id, Some(123));
        assert!(native);
    }

    #[test]
    fn test_parse_stream_name_monitor_format() {
        let (id, native) = parse_stream_name("monitor_5");
        assert_eq!(id, Some(5));
        assert!(native);
    }

    #[test]
    fn test_parse_stream_name_unknown() {
        let (id, native) = parse_stream_name("webcam");
        assert_eq!(id, None);
        assert!(!native);

        let (id, native) = parse_stream_name("front_door");
        assert_eq!(id, None);
        assert!(!native);
    }

    #[test]
    fn test_stream_name_patterns() {
        // Verify our patterns generate expected names
        let patterns: Vec<String> = ZM_STREAM_NAME_PATTERNS.iter().map(|f| f(5)).collect();

        assert!(patterns.contains(&"5".to_string()));
        assert!(patterns.contains(&"zm5".to_string()));
        assert!(patterns.contains(&"monitor_5".to_string()));
    }
}
