use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};
use urlencoding::encode;

/// go2rtc stream information returned from the API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Go2RtcStream {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub producers: Option<Vec<Go2RtcProducer>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consumers: Option<Vec<Go2RtcConsumer>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Go2RtcProducer {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub medias: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Go2RtcConsumer {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote_addr: Option<String>,
}

/// Endpoints returned after registering a stream
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamEndpoints {
    pub stream_name: String,
    pub webrtc_url: String,
    pub webrtc_api_url: String,
    pub hls_url: String,
    pub mjpeg_url: String,
    pub mse_ws_url: String,
}

/// go2rtc HTTP client for managing RTSP stream registration
pub struct Go2RtcClient {
    http_client: Client,
    base_url: String,
    timeout: Duration,
    retry_attempts: u32,
}

impl Go2RtcClient {
    /// Create a new go2rtc client
    ///
    /// # Arguments
    /// * `base_url` - The base URL of the go2rtc server (e.g., "http://localhost:1984")
    /// * `timeout_seconds` - HTTP request timeout in seconds
    /// * `retry_attempts` - Number of retry attempts for failed requests
    pub fn new(base_url: &str, timeout_seconds: u64, retry_attempts: u32) -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(timeout_seconds))
            .build()
            .expect("Failed to build HTTP client");

        info!(
            "Initialized go2rtc client: base_url={}, timeout={}s, retry_attempts={}",
            base_url, timeout_seconds, retry_attempts
        );

        Self {
            http_client,
            base_url: base_url.trim_end_matches('/').to_string(),
            timeout: Duration::from_secs(timeout_seconds),
            retry_attempts,
        }
    }

    /// Register an RTSP stream with go2rtc
    ///
    /// Registers a new stream source with go2rtc using the PUT /api/streams endpoint.
    /// The stream can then be accessed via various protocols (WebRTC, HLS, MJPEG, MSE).
    ///
    /// # Arguments
    /// * `stream_name` - Unique name for the stream (e.g., "zm1" for monitor 1)
    /// * `rtsp_url` - Full RTSP URL including credentials (e.g., "rtsp://user:pass@host:port")
    ///
    /// # Returns
    /// * `Ok(StreamEndpoints)` - URLs for accessing the stream via different protocols
    /// * `Err(Go2RtcError)` - If registration fails
    ///
    /// # Example
    /// ```no_run
    /// # use zm_api::client::go2rtc::Go2RtcClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Go2RtcClient::new("http://localhost:1984", 10, 3);
    /// let endpoints = client.register_stream(
    ///     "zm1",
    ///     "rtsp://admin:pass@192.168.1.100:554"
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn register_stream(
        &self,
        stream_name: &str,
        rtsp_url: &str,
    ) -> Result<StreamEndpoints, Go2RtcError> {
        info!(
            "Registering stream: name={}, rtsp_url={}",
            stream_name,
            // Mask credentials in logs
            self.mask_credentials(rtsp_url)
        );

        self.request_with_retry(|| async {
            // PUT /api/streams?src={rtsp_url}&name={stream_name}
            // URL-encode both the stream name and RTSP URL
            let encoded_rtsp_url = encode(rtsp_url);
            let encoded_stream_name = encode(stream_name);

            let url = format!(
                "{}/api/streams?src={}&name={}",
                self.base_url, encoded_rtsp_url, encoded_stream_name
            );

            debug!("PUT {}", self.mask_credentials(&url));

            let response = self
                .http_client
                .put(&url)
                .send()
                .await
                .map_err(Go2RtcError::HttpError)?;

            let status = response.status();

            if status.is_success() {
                info!("Successfully registered stream: {}", stream_name);

                // Build endpoint URLs using helper method
                let endpoints = self.build_endpoints(stream_name);

                Ok(endpoints)
            } else {
                let error_body = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "unknown".to_string());
                warn!(
                    "Failed to register stream {}: status={}, body={}",
                    stream_name, status, error_body
                );
                Err(Go2RtcError::ApiError {
                    status: status.as_u16(),
                    message: error_body,
                })
            }
        })
        .await
    }

    /// Get stream information from go2rtc
    ///
    /// Retrieves information about a specific stream, including its producers and consumers.
    ///
    /// # Arguments
    /// * `stream_name` - Name of the stream to query
    ///
    /// # Returns
    /// * `Ok(Some(Go2RtcStream))` - Stream information if found
    /// * `Ok(None)` - Stream not found
    /// * `Err(Go2RtcError)` - If the request fails
    pub async fn get_stream(&self, stream_name: &str) -> Result<Option<Go2RtcStream>, Go2RtcError> {
        debug!("Getting stream info: {}", stream_name);

        self.request_with_retry(|| async {
            // GET /api/streams returns all streams as a map
            let url = format!("{}/api/streams", self.base_url);

            debug!("GET {}", url);

            let response = self
                .http_client
                .get(&url)
                .send()
                .await
                .map_err(Go2RtcError::HttpError)?;

            let status = response.status();

            if status.is_success() {
                let streams: HashMap<String, Go2RtcStream> =
                    response.json().await.map_err(Go2RtcError::HttpError)?;

                Ok(streams.get(stream_name).cloned())
            } else {
                let error_body = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "unknown".to_string());
                warn!(
                    "Failed to get stream {}: status={}, body={}",
                    stream_name, status, error_body
                );
                Err(Go2RtcError::ApiError {
                    status: status.as_u16(),
                    message: error_body,
                })
            }
        })
        .await
    }

    /// List all registered streams
    ///
    /// Retrieves information about all streams currently registered with go2rtc.
    ///
    /// # Returns
    /// * `Ok(HashMap<String, Go2RtcStream>)` - Map of stream names to stream information
    /// * `Err(Go2RtcError)` - If the request fails
    pub async fn list_streams(&self) -> Result<HashMap<String, Go2RtcStream>, Go2RtcError> {
        debug!("Listing all streams");

        self.request_with_retry(|| async {
            let url = format!("{}/api/streams", self.base_url);

            debug!("GET {}", url);

            let response = self
                .http_client
                .get(&url)
                .send()
                .await
                .map_err(Go2RtcError::HttpError)?;

            let status = response.status();

            if status.is_success() {
                let streams: HashMap<String, Go2RtcStream> =
                    response.json().await.map_err(Go2RtcError::HttpError)?;

                debug!("Retrieved {} streams", streams.len());
                Ok(streams)
            } else {
                let error_body = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "unknown".to_string());
                warn!(
                    "Failed to list streams: status={}, body={}",
                    status, error_body
                );
                Err(Go2RtcError::ApiError {
                    status: status.as_u16(),
                    message: error_body,
                })
            }
        })
        .await
    }

    /// Delete a stream registration from go2rtc
    ///
    /// Removes a stream from go2rtc. This does not affect the underlying RTSP source.
    ///
    /// # Arguments
    /// * `stream_name` - Name of the stream to delete
    ///
    /// # Returns
    /// * `Ok(())` - Stream deleted successfully
    /// * `Err(Go2RtcError)` - If deletion fails
    pub async fn delete_stream(&self, stream_name: &str) -> Result<(), Go2RtcError> {
        info!("Deleting stream: {}", stream_name);

        self.request_with_retry(|| async {
            // DELETE /api/streams?src={stream_name}
            // URL-encode the stream name
            let encoded_stream_name = encode(stream_name);
            let url = format!("{}/api/streams?src={}", self.base_url, encoded_stream_name);

            debug!("DELETE {}", url);

            let response = self
                .http_client
                .delete(&url)
                .send()
                .await
                .map_err(Go2RtcError::HttpError)?;

            let status = response.status();

            if status.is_success() {
                info!("Successfully deleted stream: {}", stream_name);
                Ok(())
            } else if status.as_u16() == 404 {
                warn!("Stream not found: {}", stream_name);
                Err(Go2RtcError::StreamNotFound(stream_name.to_string()))
            } else {
                let error_body = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "unknown".to_string());
                warn!(
                    "Failed to delete stream {}: status={}, body={}",
                    stream_name, status, error_body
                );
                Err(Go2RtcError::ApiError {
                    status: status.as_u16(),
                    message: error_body,
                })
            }
        })
        .await
    }

    /// Health check - verify go2rtc is responding
    ///
    /// Performs a simple GET request to the streams endpoint to verify connectivity.
    ///
    /// # Returns
    /// * `Ok(true)` - go2rtc is healthy and responding
    /// * `Ok(false)` - go2rtc is unreachable or not responding correctly
    /// * `Err(Go2RtcError)` - If an error occurs during the health check
    pub async fn health_check(&self) -> Result<bool, Go2RtcError> {
        debug!("Performing health check");

        let url = format!("{}/api/streams", self.base_url);

        match self.http_client.get(&url).send().await {
            Ok(response) => {
                let is_healthy = response.status().is_success();
                if is_healthy {
                    info!("go2rtc health check passed");
                } else {
                    warn!("go2rtc health check failed: status={}", response.status());
                }
                Ok(is_healthy)
            }
            Err(e) => {
                error!("go2rtc health check failed: {}", e);
                Ok(false)
            }
        }
    }

    /// Build stream endpoints for a given stream name
    ///
    /// Constructs all the endpoint URLs for accessing a stream via different protocols.
    /// The stream name will be URL-encoded in the endpoint URLs.
    ///
    /// # Arguments
    /// * `stream_name` - The name of the stream (will be URL-encoded)
    ///
    /// # Returns
    /// * `StreamEndpoints` - Structure containing all endpoint URLs
    fn build_endpoints(&self, stream_name: &str) -> StreamEndpoints {
        let encoded_stream_name = encode(stream_name);

        StreamEndpoints {
            stream_name: stream_name.to_string(),
            webrtc_url: format!("{}/webrtc.html?src={}", self.base_url, encoded_stream_name),
            webrtc_api_url: format!("{}/api/webrtc?src={}", self.base_url, encoded_stream_name),
            hls_url: format!(
                "{}/api/stream.m3u8?src={}",
                self.base_url, encoded_stream_name
            ),
            mjpeg_url: format!(
                "{}/api/stream.mjpeg?src={}",
                self.base_url, encoded_stream_name
            ),
            mse_ws_url: format!("{}/api/ws?src={}", self.base_url, encoded_stream_name),
        }
    }

    /// Internal method with exponential backoff retry logic
    ///
    /// Retries failed operations with exponential backoff (1s, 2s, 4s, 8s, ...)
    ///
    /// # Type Parameters
    /// * `T` - Return type of the operation
    /// * `F` - Function type that creates the future
    /// * `Fut` - Future type returned by the function
    ///
    /// # Arguments
    /// * `operation` - Async function to retry
    async fn request_with_retry<T, F, Fut>(&self, operation: F) -> Result<T, Go2RtcError>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T, Go2RtcError>>,
    {
        let mut attempt = 0;
        let mut delay = Duration::from_secs(1);

        loop {
            attempt += 1;

            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    // Don't retry on client errors (4xx) except 429 (rate limit)
                    if let Go2RtcError::ApiError { status, .. } = &e {
                        if *status >= 400 && *status < 500 && *status != 429 {
                            debug!("Not retrying client error: {}", e);
                            return Err(e);
                        }
                    }

                    if attempt >= self.retry_attempts {
                        error!(
                            "Operation failed after {} attempts: {}",
                            self.retry_attempts, e
                        );
                        return Err(Go2RtcError::ConnectionFailed {
                            attempts: self.retry_attempts,
                        });
                    }

                    warn!(
                        "Request failed (attempt {}/{}): {}. Retrying in {:?}",
                        attempt, self.retry_attempts, e, delay
                    );

                    sleep(delay).await;

                    // Exponential backoff with cap at 16 seconds
                    delay = std::cmp::min(delay * 2, Duration::from_secs(16));
                }
            }
        }
    }

    /// Mask credentials in URLs for logging
    ///
    /// Replaces username:password@ with ***:***@ in RTSP URLs
    fn mask_credentials(&self, url: &str) -> String {
        if let Some(idx) = url.find("://") {
            let protocol = &url[..idx + 3];
            let rest = &url[idx + 3..];

            if let Some(at_idx) = rest.find('@') {
                let after_at = &rest[at_idx..];
                format!("{}***:***{}", protocol, after_at)
            } else {
                url.to_string()
            }
        } else {
            url.to_string()
        }
    }
}

impl Clone for Go2RtcClient {
    fn clone(&self) -> Self {
        Self {
            http_client: self.http_client.clone(),
            base_url: self.base_url.clone(),
            timeout: self.timeout,
            retry_attempts: self.retry_attempts,
        }
    }
}

/// Error types for go2rtc client operations
#[derive(Debug, thiserror::Error)]
pub enum Go2RtcError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("go2rtc returned error status: {status} - {message}")]
    ApiError { status: u16, message: String },

    #[error("Stream not found: {0}")]
    StreamNotFound(String),

    #[error("Connection failed after {attempts} attempts")]
    ConnectionFailed { attempts: u32 },

    #[error("Timeout waiting for response")]
    Timeout,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_credentials() {
        let client = Go2RtcClient::new("http://localhost:1984", 10, 3);

        let rtsp_url = "rtsp://admin:password123@192.168.1.100:554/stream";
        let masked = client.mask_credentials(rtsp_url);
        assert_eq!(masked, "rtsp://***:***@192.168.1.100:554/stream");

        let http_url = "http://user:pass@example.com/api";
        let masked = client.mask_credentials(http_url);
        assert_eq!(masked, "http://***:***@example.com/api");

        let no_creds = "rtsp://192.168.1.100:554/stream";
        let masked = client.mask_credentials(no_creds);
        assert_eq!(masked, "rtsp://192.168.1.100:554/stream");
    }

    #[test]
    fn test_client_creation() {
        let client = Go2RtcClient::new("http://localhost:1984", 10, 3);
        assert_eq!(client.base_url, "http://localhost:1984");
        assert_eq!(client.timeout, Duration::from_secs(10));
        assert_eq!(client.retry_attempts, 3);
    }

    #[test]
    fn test_client_creation_trailing_slash() {
        let client = Go2RtcClient::new("http://localhost:1984/", 10, 3);
        assert_eq!(client.base_url, "http://localhost:1984");
    }
}
