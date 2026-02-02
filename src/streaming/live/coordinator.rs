//! Live streaming coordinator
//!
//! Manages the lifecycle of live streaming sessions, coordinating between
//! the source router and output protocols (HLS, WebRTC, MSE).

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};

use crate::streaming::hls::HlsSessionManager;
use crate::streaming::source::SourceRouter;

/// Status of a live streaming session
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionStatus {
    Starting,
    Active,
    Stopping,
    Stopped,
    Error,
}

/// Statistics for a live streaming session
#[derive(Debug, Clone, serde::Serialize)]
pub struct LiveSessionStats {
    pub monitor_id: u32,
    pub status: SessionStatus,
    pub packets_processed: u64,
    pub errors: u64,
    pub uptime_seconds: f64,
    pub hls_enabled: bool,
    pub webrtc_enabled: bool,
    pub mse_enabled: bool,
}

/// Configuration for which protocols to enable
#[derive(Debug, Clone)]
pub struct LiveStreamConfig {
    pub enable_hls: bool,
    pub enable_webrtc: bool,
    pub enable_mse: bool,
}

impl Default for LiveStreamConfig {
    fn default() -> Self {
        Self {
            enable_hls: true,
            enable_webrtc: false, // Not implemented yet
            enable_mse: false,    // Not implemented yet
        }
    }
}

/// Internal state for a live session
struct LiveSession {
    monitor_id: u32,
    status: SessionStatus,
    config: LiveStreamConfig,
    task_handle: Option<JoinHandle<()>>,
    started_at: std::time::Instant,
    packets_processed: u64,
    errors: u64,
}

impl LiveSession {
    fn new(monitor_id: u32, config: LiveStreamConfig) -> Self {
        Self {
            monitor_id,
            status: SessionStatus::Starting,
            config,
            task_handle: None,
            started_at: std::time::Instant::now(),
            packets_processed: 0,
            errors: 0,
        }
    }

    fn stats(&self) -> LiveSessionStats {
        LiveSessionStats {
            monitor_id: self.monitor_id,
            status: self.status,
            packets_processed: self.packets_processed,
            errors: self.errors,
            uptime_seconds: self.started_at.elapsed().as_secs_f64(),
            hls_enabled: self.config.enable_hls,
            webrtc_enabled: self.config.enable_webrtc,
            mse_enabled: self.config.enable_mse,
        }
    }
}

/// Coordinator errors
#[derive(Debug, thiserror::Error)]
pub enum CoordinatorError {
    #[error("Session already exists for monitor {0}")]
    SessionExists(u32),

    #[error("Session not found for monitor {0}")]
    SessionNotFound(u32),

    #[error("Source not available for monitor {0}")]
    SourceNotAvailable(u32),

    #[error("HLS not configured")]
    HlsNotConfigured,

    #[error("Source error: {0}")]
    SourceError(String),

    #[error("HLS error: {0}")]
    HlsError(String),
}

/// Coordinates live streaming from FIFO sources to output protocols
pub struct LiveStreamCoordinator {
    /// Source router for FIFO access
    source_router: Arc<SourceRouter>,
    /// HLS session manager
    hls_manager: Option<Arc<HlsSessionManager>>,
    /// Active sessions
    sessions: Arc<RwLock<HashMap<u32, LiveSession>>>,
}

impl LiveStreamCoordinator {
    /// Create a new coordinator
    pub fn new(
        source_router: Arc<SourceRouter>,
        hls_manager: Option<Arc<HlsSessionManager>>,
    ) -> Self {
        Self {
            source_router,
            hls_manager,
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start a live streaming session for a monitor
    pub async fn start_session(
        &self,
        monitor_id: u32,
        config: LiveStreamConfig,
    ) -> Result<(), CoordinatorError> {
        // Check if session already exists
        {
            let sessions = self.sessions.read().await;
            if sessions.contains_key(&monitor_id) {
                return Err(CoordinatorError::SessionExists(monitor_id));
            }
        }

        // Check if HLS is enabled and required
        if config.enable_hls && self.hls_manager.is_none() {
            return Err(CoordinatorError::HlsNotConfigured);
        }

        // Check if source is available
        if !self.source_router.is_available(monitor_id) {
            return Err(CoordinatorError::SourceNotAvailable(monitor_id));
        }

        // Get the source (this will start the reader if auto_start is enabled)
        let _source = self
            .source_router
            .get_source(monitor_id)
            .await
            .map_err(|e| CoordinatorError::SourceError(e.to_string()))?;

        // Start HLS session if enabled
        if config.enable_hls {
            if let Some(hls_manager) = &self.hls_manager {
                hls_manager
                    .start_session(monitor_id)
                    .await
                    .map_err(|e| CoordinatorError::HlsError(e.to_string()))?;
            }
        }

        // Create session
        let mut session = LiveSession::new(monitor_id, config.clone());

        // Start processing task
        let task_handle = self.start_processing_task(monitor_id, config);
        session.task_handle = Some(task_handle);
        session.status = SessionStatus::Active;

        // Store session
        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(monitor_id, session);
        }

        info!("Started live streaming session for monitor {}", monitor_id);
        Ok(())
    }

    /// Start the packet processing task
    fn start_processing_task(&self, monitor_id: u32, config: LiveStreamConfig) -> JoinHandle<()> {
        let source_router = Arc::clone(&self.source_router);
        let hls_manager = self.hls_manager.clone();
        let sessions = Arc::clone(&self.sessions);

        tokio::spawn(async move {
            info!("Starting packet processing task for monitor {}", monitor_id);

            // Subscribe to video packets
            let mut video_rx = match source_router.subscribe_video(monitor_id).await {
                Ok(rx) => rx,
                Err(e) => {
                    error!(
                        "Failed to subscribe to video for monitor {}: {}",
                        monitor_id, e
                    );
                    return;
                }
            };

            // Process packets
            loop {
                match video_rx.recv().await {
                    Ok(packet) => {
                        // Process through HLS if enabled
                        if config.enable_hls {
                            if let Some(hls) = &hls_manager {
                                if let Err(e) = hls.process_packet(&packet).await {
                                    debug!("HLS process error for monitor {}: {}", monitor_id, e);
                                    // Increment error count
                                    let mut sessions_guard = sessions.write().await;
                                    if let Some(session) = sessions_guard.get_mut(&monitor_id) {
                                        session.errors += 1;
                                    }
                                }
                            }
                        }

                        // TODO: Process through WebRTC if enabled
                        // TODO: Process through MSE if enabled

                        // Update packet count
                        {
                            let mut sessions_guard = sessions.write().await;
                            if let Some(session) = sessions_guard.get_mut(&monitor_id) {
                                session.packets_processed += 1;
                            }
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        warn!(
                            "Processing task lagged {} packets for monitor {}",
                            n, monitor_id
                        );
                        // Continue processing
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        info!(
                            "Video channel closed for monitor {}, stopping task",
                            monitor_id
                        );
                        break;
                    }
                }
            }

            // Mark session as stopped
            {
                let mut sessions_guard = sessions.write().await;
                if let Some(session) = sessions_guard.get_mut(&monitor_id) {
                    session.status = SessionStatus::Stopped;
                }
            }

            info!("Packet processing task stopped for monitor {}", monitor_id);
        })
    }

    /// Stop a live streaming session
    pub async fn stop_session(&self, monitor_id: u32) -> Result<(), CoordinatorError> {
        let mut sessions = self.sessions.write().await;

        let session = sessions
            .get_mut(&monitor_id)
            .ok_or(CoordinatorError::SessionNotFound(monitor_id))?;

        session.status = SessionStatus::Stopping;

        // Abort the processing task
        if let Some(handle) = session.task_handle.take() {
            handle.abort();
        }

        // Stop HLS session if enabled
        if session.config.enable_hls {
            if let Some(hls_manager) = &self.hls_manager {
                let _ = hls_manager.stop_session(monitor_id).await;
            }
        }

        // Stop the source reader
        let _ = self.source_router.stop_reader(monitor_id).await;

        sessions.remove(&monitor_id);

        info!("Stopped live streaming session for monitor {}", monitor_id);
        Ok(())
    }

    /// Check if a session exists
    pub async fn has_session(&self, monitor_id: u32) -> bool {
        let sessions = self.sessions.read().await;
        sessions.contains_key(&monitor_id)
    }

    /// Get session statistics
    pub async fn get_stats(&self, monitor_id: u32) -> Option<LiveSessionStats> {
        let sessions = self.sessions.read().await;
        sessions.get(&monitor_id).map(|s| s.stats())
    }

    /// List all active sessions
    pub async fn list_sessions(&self) -> Vec<u32> {
        let sessions = self.sessions.read().await;
        sessions.keys().copied().collect()
    }

    /// Get statistics for all sessions
    pub async fn all_stats(&self) -> Vec<LiveSessionStats> {
        let sessions = self.sessions.read().await;
        sessions.values().map(|s| s.stats()).collect()
    }

    /// Get the source router reference
    pub fn source_router(&self) -> &Arc<SourceRouter> {
        &self.source_router
    }

    /// Get the HLS manager reference
    pub fn hls_manager(&self) -> Option<&Arc<HlsSessionManager>> {
        self.hls_manager.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_live_stream_config_default() {
        let config = LiveStreamConfig::default();
        assert!(config.enable_hls);
        assert!(!config.enable_webrtc);
        assert!(!config.enable_mse);
    }

    #[test]
    fn test_session_status_serialize() {
        let status = SessionStatus::Active;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"active\"");
    }

    #[tokio::test]
    async fn test_coordinator_creation() {
        let source_router = Arc::new(SourceRouter::new());
        let coordinator = LiveStreamCoordinator::new(source_router, None);

        assert!(coordinator.list_sessions().await.is_empty());
    }

    #[tokio::test]
    async fn test_session_not_found() {
        let source_router = Arc::new(SourceRouter::new());
        let coordinator = LiveStreamCoordinator::new(source_router, None);

        let result = coordinator.stop_session(99999).await;
        assert!(matches!(
            result,
            Err(CoordinatorError::SessionNotFound(99999))
        ));
    }

    #[tokio::test]
    async fn test_hls_not_configured() {
        let source_router = Arc::new(SourceRouter::new());
        let coordinator = LiveStreamCoordinator::new(source_router, None);

        let config = LiveStreamConfig {
            enable_hls: true,
            enable_webrtc: false,
            enable_mse: false,
        };

        let result = coordinator.start_session(1, config).await;
        assert!(matches!(result, Err(CoordinatorError::HlsNotConfigured)));
    }
}
