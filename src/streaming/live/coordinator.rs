//! Live streaming coordinator
//!
//! Manages the lifecycle of live streaming sessions, coordinating between
//! the source router and output protocols (HLS, WebRTC, MSE).

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tracing::{debug, info, warn};

use crate::streaming::hls::HlsSessionManager;
use crate::streaming::source::router::ReaderHealth;
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

        // Create the source WITHOUT starting the reader — this gives us a chance
        // to subscribe before any packets flow, so we never miss SPS/PPS.
        let source = self
            .source_router
            .create_source(monitor_id)
            .await
            .map_err(|e| CoordinatorError::SourceError(e.to_string()))?;

        // Subscribe BEFORE starting the reader — guarantees we see all packets
        let video_rx = source.subscribe_video();
        let reader_health_rx = source.subscribe_reader_health();

        // NOW start the FIFO reader
        self.source_router
            .start_reader(monitor_id)
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

        // Store session BEFORE spawning the processing task so the task
        // can find and update it (status, packet counts). Status starts as
        // Starting and transitions to Active when the first packet arrives.
        {
            let session = LiveSession::new(monitor_id, config.clone());
            let mut sessions = self.sessions.write().await;
            sessions.insert(monitor_id, session);
        }

        // Start processing task with pre-subscribed receivers
        let task_handle =
            self.start_processing_task(monitor_id, config, video_rx, reader_health_rx);

        // Store the task handle
        {
            let mut sessions = self.sessions.write().await;
            if let Some(session) = sessions.get_mut(&monitor_id) {
                session.task_handle = Some(task_handle);
            }
        }

        info!("Started live streaming session for monitor {}", monitor_id);
        Ok(())
    }

    /// Start the packet processing task with a pre-subscribed receiver.
    ///
    /// The receiver must be created BEFORE the FIFO reader starts to guarantee
    /// no packets (especially initial SPS/PPS) are lost.
    ///
    /// The task transitions the session from Starting → Active on first packet,
    /// or Starting → Error if no packets arrive within the startup timeout.
    fn start_processing_task(
        &self,
        monitor_id: u32,
        config: LiveStreamConfig,
        video_rx: tokio::sync::broadcast::Receiver<crate::streaming::source::fifo::FifoPacket>,
        reader_health_rx: tokio::sync::watch::Receiver<ReaderHealth>,
    ) -> JoinHandle<()> {
        /// Maximum time to wait for the first video packet before marking the
        /// session as errored. Covers FIFO open retries + ZoneMinder startup.
        const STARTUP_TIMEOUT: Duration = Duration::from_secs(15);

        let hls_manager = self.hls_manager.clone();
        let sessions = Arc::clone(&self.sessions);

        tokio::spawn(async move {
            info!("Starting packet processing task for monitor {}", monitor_id);
            let mut video_rx = video_rx;
            let mut reader_health_rx = reader_health_rx;
            let mut received_first_packet = false;
            let startup_deadline = tokio::time::Instant::now() + STARTUP_TIMEOUT;

            loop {
                // Until the first packet arrives, race recv against a startup
                // timeout so we detect dead readers (missing FIFO, ZM not running).
                // Also monitor reader health — if the watch sender is dropped
                // (reader task exited), we stop instead of hanging forever on the
                // broadcast channel (which never closes while MonitorSource lives).
                let recv_result = if !received_first_packet {
                    tokio::select! {
                        result = video_rx.recv() => result,
                        () = tokio::time::sleep_until(startup_deadline) => {
                            warn!(
                                "No video packets received within {}s for monitor {}, marking session as error",
                                STARTUP_TIMEOUT.as_secs(),
                                monitor_id,
                            );
                            let mut sessions_guard = sessions.write().await;
                            if let Some(session) = sessions_guard.get_mut(&monitor_id) {
                                session.status = SessionStatus::Error;
                            }
                            return;
                        }
                        health = reader_health_rx.changed() => {
                            match health {
                                Ok(()) => {
                                    // Reader health changed — log it but keep waiting for packets
                                    let state = *reader_health_rx.borrow();
                                    debug!(
                                        "Reader health for monitor {}: {:?}",
                                        monitor_id, state
                                    );
                                    continue;
                                }
                                Err(_) => {
                                    // Watch sender dropped — reader task exited
                                    warn!(
                                        "Reader task exited for monitor {}, marking session as error",
                                        monitor_id,
                                    );
                                    let mut sessions_guard = sessions.write().await;
                                    if let Some(session) = sessions_guard.get_mut(&monitor_id) {
                                        session.status = SessionStatus::Error;
                                    }
                                    return;
                                }
                            }
                        }
                    }
                } else {
                    tokio::select! {
                        result = video_rx.recv() => result,
                        health = reader_health_rx.changed() => {
                            match health {
                                Ok(()) => {
                                    let state = *reader_health_rx.borrow();
                                    debug!(
                                        "Reader health for monitor {}: {:?}",
                                        monitor_id, state
                                    );
                                    continue;
                                }
                                Err(_) => {
                                    warn!(
                                        "Reader task exited for monitor {}, stopping processing",
                                        monitor_id,
                                    );
                                    break;
                                }
                            }
                        }
                    }
                };

                match recv_result {
                    Ok(packet) => {
                        // Transition Starting → Active on first packet
                        if !received_first_packet {
                            received_first_packet = true;
                            let mut sessions_guard = sessions.write().await;
                            if let Some(session) = sessions_guard.get_mut(&monitor_id) {
                                session.status = SessionStatus::Active;
                            }
                            info!(
                                "First video packet received for monitor {}, session active",
                                monitor_id
                            );
                        }

                        // Process through HLS if enabled
                        if config.enable_hls {
                            if let Some(hls) = &hls_manager {
                                if let Err(e) = hls.process_packet(&packet).await {
                                    debug!("HLS process error for monitor {}: {}", monitor_id, e);
                                    let mut sessions_guard = sessions.write().await;
                                    if let Some(session) = sessions_guard.get_mut(&monitor_id) {
                                        session.errors += 1;
                                    }
                                }
                            }
                        }

                        // Update packet count
                        {
                            let mut sessions_guard = sessions.write().await;
                            if let Some(session) = sessions_guard.get_mut(&monitor_id) {
                                session.packets_processed += 1;
                            }
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        // Lagged means packets ARE flowing — we just missed some
                        if !received_first_packet {
                            received_first_packet = true;
                            let mut sessions_guard = sessions.write().await;
                            if let Some(session) = sessions_guard.get_mut(&monitor_id) {
                                session.status = SessionStatus::Active;
                            }
                            info!(
                                "Video packets flowing for monitor {} (lagged {}), session active",
                                monitor_id, n
                            );
                        } else {
                            warn!(
                                "Processing task lagged {} packets for monitor {}",
                                n, monitor_id
                            );
                        }
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
