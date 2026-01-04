//! HLS session management
//!
//! Manages active HLS streaming sessions, coordinating segmentation,
//! storage, and playlist generation for each monitor.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, error, info, warn};

use super::playlist::{MediaPlaylist, PlaylistGenerator, SegmentRef};
use super::segmenter::{FMP4Segment, HlsSegmenter, InitSegment};
use super::storage::{HlsStorage, SegmentInfo, StorageConfig, StorageError};
use crate::configure::streaming::HlsConfig;
use crate::streaming::source::fifo::{FifoPacket, VideoCodec};

/// HLS session for a single monitor
pub struct HlsSession {
    monitor_id: u32,
    segmenter: HlsSegmenter,
    playlist: MediaPlaylist,
    playlist_generator: PlaylistGenerator,
    started_at: Instant,
    segment_count: u64,
    bytes_written: u64,
    viewer_count: usize,
    /// Channel for notifying about new segments
    segment_tx: broadcast::Sender<u64>,
}

impl HlsSession {
    /// Create a new HLS session
    pub fn new(monitor_id: u32, config: &HlsConfig, base_url: &str) -> Self {
        let target_duration = Duration::from_secs(config.segment_duration_seconds as u64);
        let segmenter = HlsSegmenter::new(monitor_id, target_duration);

        let mut playlist_generator = PlaylistGenerator::new(
            monitor_id,
            base_url,
            config.segment_duration_seconds,
            config.playlist_size as usize,
        );

        if config.ll_hls_enabled {
            playlist_generator =
                playlist_generator.with_ll_hls(config.partial_segment_ms as f64 / 1000.0);
        }

        let playlist = playlist_generator.generate_media_playlist();
        let (segment_tx, _) = broadcast::channel(16);

        Self {
            monitor_id,
            segmenter,
            playlist,
            playlist_generator,
            started_at: Instant::now(),
            segment_count: 0,
            bytes_written: 0,
            viewer_count: 0,
            segment_tx,
        }
    }

    /// Process a packet from FIFO
    pub fn process_packet(&mut self, packet: &FifoPacket) -> Option<FMP4Segment> {
        // Update codec if detected
        if self.segmenter.sequence() == 0 {
            self.segmenter.set_codec(packet.codec);
        }

        // Process NAL unit
        self.segmenter
            .process_nal(&packet.data, packet.timestamp_us, packet.is_keyframe)
    }

    /// Handle a completed segment
    pub fn on_segment_complete(&mut self, segment: &FMP4Segment) {
        self.segment_count += 1;
        self.bytes_written += segment.data.len() as u64;

        // Add to playlist
        let segment_ref = SegmentRef {
            sequence: segment.sequence,
            duration: segment.duration.as_secs_f64(),
            uri: format!("segment_{:05}.m4s", segment.sequence),
            is_independent: segment.is_keyframe,
            parts: vec![],
        };

        self.playlist.add_segment(segment_ref);
        self.playlist
            .trim_to_size(self.playlist_generator.playlist_size);

        // Notify waiters
        let _ = self.segment_tx.send(segment.sequence);
    }

    /// Generate init segment if ready
    pub fn generate_init_segment(&mut self) -> Option<InitSegment> {
        self.segmenter.generate_init_segment()
    }

    /// Get cached init segment
    pub fn get_init_segment(&self) -> Option<&InitSegment> {
        self.segmenter.get_init_segment()
    }

    /// Generate current playlist
    pub fn generate_playlist(&self) -> String {
        self.playlist.generate()
    }

    /// Subscribe to segment notifications
    pub fn subscribe(&self) -> broadcast::Receiver<u64> {
        self.segment_tx.subscribe()
    }

    /// Increment viewer count
    pub fn add_viewer(&mut self) {
        self.viewer_count += 1;
    }

    /// Decrement viewer count
    pub fn remove_viewer(&mut self) {
        self.viewer_count = self.viewer_count.saturating_sub(1);
    }

    /// Get session statistics
    pub fn stats(&self) -> HlsSessionStats {
        HlsSessionStats {
            monitor_id: self.monitor_id,
            uptime: self.started_at.elapsed(),
            segment_count: self.segment_count,
            bytes_written: self.bytes_written,
            viewer_count: self.viewer_count,
            current_sequence: self.segmenter.sequence(),
            has_init_segment: self.segmenter.get_init_segment().is_some(),
        }
    }

    /// Get monitor ID
    pub fn monitor_id(&self) -> u32 {
        self.monitor_id
    }
}

/// Session statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct HlsSessionStats {
    pub monitor_id: u32,
    #[serde(with = "duration_serde")]
    pub uptime: Duration,
    pub segment_count: u64,
    pub bytes_written: u64,
    pub viewer_count: usize,
    pub current_sequence: u64,
    pub has_init_segment: bool,
}

mod duration_serde {
    use serde::{Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        duration.as_secs_f64().serialize(serializer)
    }
}

/// Manages all HLS sessions
pub struct HlsSessionManager {
    sessions: Arc<RwLock<HashMap<u32, HlsSession>>>,
    storage: Arc<HlsStorage>,
    config: HlsConfig,
    base_url: String,
}

impl HlsSessionManager {
    /// Create a new session manager
    pub fn new(config: HlsConfig, base_url: &str) -> Self {
        let storage_config = StorageConfig {
            base_path: config.storage.path.clone().into(),
            retention: Duration::from_secs(config.storage.retention_minutes as u64 * 60),
            cleanup_interval: Duration::from_secs(60),
            max_segments_per_stream: (config.storage.retention_minutes as usize * 60)
                / config.segment_duration_seconds as usize,
        };

        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            storage: Arc::new(HlsStorage::new(storage_config)),
            config,
            base_url: base_url.to_string(),
        }
    }

    /// Start an HLS session for a monitor
    pub async fn start_session(&self, monitor_id: u32) -> Result<(), HlsError> {
        let mut sessions = self.sessions.write().await;

        if sessions.contains_key(&monitor_id) {
            return Err(HlsError::SessionExists(monitor_id));
        }

        // Initialize storage
        self.storage.init_monitor(monitor_id).await?;

        // Create session
        let session = HlsSession::new(monitor_id, &self.config, &self.base_url);
        sessions.insert(monitor_id, session);

        info!("Started HLS session for monitor {}", monitor_id);
        Ok(())
    }

    /// Stop an HLS session
    pub async fn stop_session(&self, monitor_id: u32) -> Result<(), HlsError> {
        let mut sessions = self.sessions.write().await;

        if sessions.remove(&monitor_id).is_none() {
            return Err(HlsError::SessionNotFound(monitor_id));
        }

        // Optionally clean up storage
        // self.storage.remove_monitor(monitor_id).await?;

        info!("Stopped HLS session for monitor {}", monitor_id);
        Ok(())
    }

    /// Check if a session exists
    pub async fn has_session(&self, monitor_id: u32) -> bool {
        let sessions = self.sessions.read().await;
        sessions.contains_key(&monitor_id)
    }

    /// Process a packet for a session
    pub async fn process_packet(&self, packet: &FifoPacket) -> Result<(), HlsError> {
        let monitor_id = packet.monitor_id;

        let segment = {
            let mut sessions = self.sessions.write().await;
            let session = sessions
                .get_mut(&monitor_id)
                .ok_or(HlsError::SessionNotFound(monitor_id))?;

            // Try to generate init segment if we don't have one
            if session.get_init_segment().is_none() {
                if let Some(init) = session.generate_init_segment() {
                    self.storage
                        .store_init_segment(monitor_id, &init.data)
                        .await?;
                }
            }

            session.process_packet(packet)
        };

        // If a segment was produced, store it
        if let Some(segment) = segment {
            self.storage
                .store_segment(
                    monitor_id,
                    segment.sequence,
                    &segment.data,
                    segment.duration,
                )
                .await?;

            let mut sessions = self.sessions.write().await;
            if let Some(session) = sessions.get_mut(&monitor_id) {
                session.on_segment_complete(&segment);
            }
        }

        Ok(())
    }

    /// Get the current playlist for a monitor
    pub async fn get_playlist(&self, monitor_id: u32) -> Result<String, HlsError> {
        let sessions = self.sessions.read().await;
        let session = sessions
            .get(&monitor_id)
            .ok_or(HlsError::SessionNotFound(monitor_id))?;

        Ok(session.generate_playlist())
    }

    /// Get the master playlist for a monitor
    pub async fn get_master_playlist(&self, monitor_id: u32) -> Result<String, HlsError> {
        let sessions = self.sessions.read().await;
        let session = sessions
            .get(&monitor_id)
            .ok_or(HlsError::SessionNotFound(monitor_id))?;

        let master = session.playlist_generator.generate_master_playlist();
        Ok(master.generate())
    }

    /// Get init segment data
    pub async fn get_init_segment(&self, monitor_id: u32) -> Result<Vec<u8>, HlsError> {
        self.storage
            .read_init_segment(monitor_id)
            .await
            .map_err(HlsError::from)
    }

    /// Get segment data
    pub async fn get_segment(&self, monitor_id: u32, sequence: u64) -> Result<Vec<u8>, HlsError> {
        self.storage
            .read_segment(monitor_id, sequence)
            .await
            .map_err(HlsError::from)
    }

    /// Subscribe to new segments for LL-HLS blocking reload
    pub async fn subscribe(&self, monitor_id: u32) -> Result<broadcast::Receiver<u64>, HlsError> {
        let sessions = self.sessions.read().await;
        let session = sessions
            .get(&monitor_id)
            .ok_or(HlsError::SessionNotFound(monitor_id))?;

        Ok(session.subscribe())
    }

    /// Wait for a specific segment (for LL-HLS blocking reload)
    pub async fn wait_for_segment(
        &self,
        monitor_id: u32,
        target_sequence: u64,
        timeout: Duration,
    ) -> Result<(), HlsError> {
        let mut receiver = self.subscribe(monitor_id).await?;

        // Check if already available
        {
            let sessions = self.sessions.read().await;
            if let Some(session) = sessions.get(&monitor_id) {
                if session.segmenter.sequence() > target_sequence {
                    return Ok(());
                }
            }
        }

        // Wait for segment
        let deadline = Instant::now() + timeout;

        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Err(HlsError::Timeout);
            }

            match tokio::time::timeout(remaining, receiver.recv()).await {
                Ok(Ok(seq)) if seq >= target_sequence => return Ok(()),
                Ok(Ok(_)) => continue,
                Ok(Err(_)) => return Err(HlsError::SessionNotFound(monitor_id)),
                Err(_) => return Err(HlsError::Timeout),
            }
        }
    }

    /// Get session statistics
    pub async fn get_stats(&self, monitor_id: u32) -> Result<HlsSessionStats, HlsError> {
        let sessions = self.sessions.read().await;
        let session = sessions
            .get(&monitor_id)
            .ok_or(HlsError::SessionNotFound(monitor_id))?;

        Ok(session.stats())
    }

    /// Get all active sessions
    pub async fn list_sessions(&self) -> Vec<u32> {
        let sessions = self.sessions.read().await;
        sessions.keys().copied().collect()
    }

    /// Add viewer to session
    pub async fn add_viewer(&self, monitor_id: u32) -> Result<(), HlsError> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(&monitor_id)
            .ok_or(HlsError::SessionNotFound(monitor_id))?;
        session.add_viewer();
        Ok(())
    }

    /// Remove viewer from session
    pub async fn remove_viewer(&self, monitor_id: u32) -> Result<(), HlsError> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(&monitor_id)
            .ok_or(HlsError::SessionNotFound(monitor_id))?;
        session.remove_viewer();
        Ok(())
    }

    /// Get storage reference
    pub fn storage(&self) -> &Arc<HlsStorage> {
        &self.storage
    }

    /// Start the background cleanup task
    pub fn start_cleanup_task(&self) -> tokio::task::JoinHandle<()> {
        Arc::clone(&self.storage).start_cleanup_task()
    }
}

/// HLS errors
#[derive(Debug, thiserror::Error)]
pub enum HlsError {
    #[error("HLS session already exists for monitor {0}")]
    SessionExists(u32),

    #[error("HLS session not found for monitor {0}")]
    SessionNotFound(u32),

    #[error("Storage error: {0}")]
    StorageError(#[from] StorageError),

    #[error("Timeout waiting for segment")]
    Timeout,

    #[error("Init segment not ready")]
    InitNotReady,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> HlsConfig {
        HlsConfig {
            enabled: true,
            segment_duration_seconds: 2,
            playlist_size: 6,
            ll_hls_enabled: false,
            partial_segment_ms: 200,
            storage: crate::configure::streaming::HlsStorageConfig {
                path: "/tmp/hls-test".to_string(),
                retention_minutes: 5,
            },
        }
    }

    #[test]
    fn test_session_creation() {
        let config = test_config();
        let session = HlsSession::new(1, &config, "/api/v3/hls/1");

        assert_eq!(session.monitor_id(), 1);
        assert_eq!(session.segment_count, 0);
    }

    #[test]
    fn test_session_stats() {
        let config = test_config();
        let session = HlsSession::new(1, &config, "/api/v3/hls/1");

        let stats = session.stats();
        assert_eq!(stats.monitor_id, 1);
        assert_eq!(stats.segment_count, 0);
        assert!(!stats.has_init_segment);
    }

    #[test]
    fn test_playlist_generation() {
        let config = test_config();
        let session = HlsSession::new(1, &config, "/api/v3/hls/1");

        let playlist = session.generate_playlist();
        assert!(playlist.contains("#EXTM3U"));
        assert!(playlist.contains("#EXT-X-TARGETDURATION:2"));
    }

    #[tokio::test]
    async fn test_session_manager_creation() {
        let config = test_config();
        let manager = HlsSessionManager::new(config, "/api/v3/hls");

        let sessions = manager.list_sessions().await;
        assert!(sessions.is_empty());
    }
}
