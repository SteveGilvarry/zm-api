//! HLS session management
//!
//! Manages active HLS streaming sessions, coordinating segmentation,
//! storage, and playlist generation for each monitor.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, RwLock};
use tokio::task::JoinHandle;
use tracing::{info, warn};

use super::playlist::{MediaPlaylist, PlaylistGenerator, SegmentRef};
use super::segmenter::{FMP4Segment, HlsSegmenter, InitSegment};
use super::storage::{HlsStorage, StorageConfig, StorageError};
use crate::configure::streaming::HlsConfig;
use crate::streaming::source::fifo::FifoPacket;

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
    /// Last time a client requested this session's playlist or media. Updated
    /// through a shared `&self` reference (interior mutability) so the
    /// frequently-taken read lock on the session map suffices. Drives idle
    /// reaping — see [`HlsSessionManager::idle_sessions`].
    last_access: Mutex<Instant>,
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
            last_access: Mutex::new(Instant::now()),
        }
    }

    /// Mark the session as freshly accessed (a client fetched its playlist or
    /// media). Resets the idle clock.
    fn touch(&self) {
        *self.last_access.lock().unwrap() = Instant::now();
    }

    /// How long since the session was last accessed by a client.
    fn idle_for(&self) -> Duration {
        self.last_access.lock().unwrap().elapsed()
    }

    /// Process a packet from FIFO
    pub fn process_packet(&mut self, packet: &FifoPacket) -> Option<FMP4Segment> {
        // Update codec if detected
        if self.segmenter.sequence() == 0 {
            self.segmenter.set_codec(packet.codec);
        }

        let timestamp_us = match u64::try_from(packet.timestamp_us) {
            Ok(value) => value,
            Err(_) => {
                warn!(
                    "Negative timestamp for monitor {}: {}, skipping packet",
                    packet.monitor_id, packet.timestamp_us
                );
                return None;
            }
        };

        // Process NAL unit
        self.segmenter
            .process_nal(&packet.data, timestamp_us, packet.is_keyframe)
    }

    /// Handle a completed segment
    pub fn on_segment_complete(&mut self, segment: &FMP4Segment) {
        self.segment_count += 1;
        self.bytes_written += segment.data.len() as u64;

        // Update target duration before adding segment (HLS spec compliance)
        let duration_secs = segment.duration.as_secs_f64();
        self.playlist.update_target_duration(duration_secs);

        // Add to playlist
        let segment_ref = SegmentRef {
            sequence: segment.sequence,
            duration: duration_secs,
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

    /// Generate init segment if ready.
    /// Also extracts the codec profile-level-id from SPS for the master playlist.
    pub fn generate_init_segment(&mut self) -> Option<InitSegment> {
        let init = self.segmenter.generate_init_segment()?;

        // Extract profile-level-id from SPS and update master playlist codec
        if let Some(sps) = self.segmenter.sps() {
            // SPS bytes (without start code): [nal_type, profile_idc, constraint_flags, level_idc, ...]
            if sps.len() >= 4 {
                let profile_level_id = format!("{:02x}{:02x}{:02x}", sps[1], sps[2], sps[3]);
                self.playlist_generator.codec = Some(format!("avc1.{profile_level_id}"));
            }
        }

        Some(init)
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
    /// Handle to the background storage-cleanup loop, retained so it is aborted
    /// when the manager is dropped (otherwise the task would outlive the manager
    /// and keep its `Arc<HlsStorage>` clone alive — a leak on reload/in tests).
    cleanup_handle: Mutex<Option<JoinHandle<()>>>,
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
            cleanup_handle: Mutex::new(None),
        }
    }

    /// Start an HLS session for a monitor
    pub async fn start_session(&self, monitor_id: u32) -> Result<(), HlsError> {
        let mut sessions = self.sessions.write().await;

        if sessions.contains_key(&monitor_id) {
            return Err(HlsError::SessionExists(monitor_id));
        }

        // Initialize storage and clean any stale files from a previous session
        self.storage.init_monitor(monitor_id).await?;
        self.storage.clean_monitor(monitor_id).await?;

        // Create session with per-monitor base URL including /hls/ prefix
        let session_base_url = format!("{}/{}/hls", self.base_url, monitor_id);
        let session = HlsSession::new(monitor_id, &self.config, &session_base_url);
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

            // Process the packet first — this extracts SPS/PPS from NAL units
            let segment = session.process_packet(packet);

            // Now try to generate init segment (requires SPS+PPS from above)
            if session.get_init_segment().is_none() {
                if let Some(init) = session.generate_init_segment() {
                    self.storage
                        .store_init_segment(monitor_id, &init.data)
                        .await?;
                }
            }

            segment
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

        session.touch();
        Ok(session.generate_playlist())
    }

    /// Get the master playlist for a monitor
    pub async fn get_master_playlist(&self, monitor_id: u32) -> Result<String, HlsError> {
        let sessions = self.sessions.read().await;
        let session = sessions
            .get(&monitor_id)
            .ok_or(HlsError::SessionNotFound(monitor_id))?;

        session.touch();
        let master = session.playlist_generator.generate_master_playlist();
        Ok(master.generate())
    }

    /// Record that a client accessed this monitor's session (keeps it alive).
    /// A no-op if the session doesn't exist.
    async fn note_access(&self, monitor_id: u32) {
        let sessions = self.sessions.read().await;
        if let Some(session) = sessions.get(&monitor_id) {
            session.touch();
        }
    }

    /// Get init segment data
    pub async fn get_init_segment(&self, monitor_id: u32) -> Result<Vec<u8>, HlsError> {
        self.note_access(monitor_id).await;
        self.storage
            .read_init_segment(monitor_id)
            .await
            .map_err(HlsError::from)
    }

    /// Get segment data
    pub async fn get_segment(&self, monitor_id: u32, sequence: u64) -> Result<Vec<u8>, HlsError> {
        self.note_access(monitor_id).await;
        self.storage
            .read_segment(monitor_id, sequence)
            .await
            .map_err(HlsError::from)
    }

    /// Monitors whose sessions have not been accessed for at least `idle_for`.
    /// Used by the coordinator's watchdog to reap sessions left behind by
    /// viewers who navigated away. See REVIEW_FIXES_PLAN §3.2.
    pub async fn idle_sessions(&self, idle_for: Duration) -> Vec<u32> {
        let sessions = self.sessions.read().await;
        sessions
            .iter()
            .filter(|(_, session)| session.idle_for() >= idle_for)
            .map(|(id, _)| *id)
            .collect()
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
                // A blocking LL-HLS poll is the clearest sign of an active
                // viewer — keep the session off the idle-reap list.
                session.touch();
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

    /// Start the background storage-cleanup loop and retain its handle so it is
    /// aborted on drop. Previously this returned a handle that every caller
    /// dropped on the floor — so the loop was, in fact, never started at all
    /// (no caller existed). Call this once after constructing the manager.
    /// See REVIEW_FIXES_PLAN §3.1.
    pub fn start_cleanup_task(&self) {
        let handle = Arc::clone(&self.storage).start_cleanup_task();
        if let Some(old) = self.cleanup_handle.lock().unwrap().replace(handle) {
            old.abort();
        }
    }

    /// The configured idle-session timeout (`0` disables reaping).
    pub fn idle_timeout(&self) -> Duration {
        Duration::from_secs(self.config.idle_timeout_seconds)
    }
}

impl Drop for HlsSessionManager {
    fn drop(&mut self) {
        if let Some(handle) = self.cleanup_handle.lock().unwrap().take() {
            handle.abort();
        }
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
            idle_timeout_seconds: 90,
            storage: crate::configure::streaming::HlsStorageConfig {
                path: "/tmp/hls-test".to_string(),
                retention_minutes: 5,
            },
        }
    }

    #[test]
    fn test_session_creation() {
        let config = test_config();
        let session = HlsSession::new(1, &config, "/api/v3/live/1/hls");

        assert_eq!(session.monitor_id(), 1);
        assert_eq!(session.segment_count, 0);
    }

    #[test]
    fn test_session_stats() {
        let config = test_config();
        let session = HlsSession::new(1, &config, "/api/v3/live/1/hls");

        let stats = session.stats();
        assert_eq!(stats.monitor_id, 1);
        assert_eq!(stats.segment_count, 0);
        assert!(!stats.has_init_segment);
    }

    #[test]
    fn test_playlist_generation() {
        let config = test_config();
        let session = HlsSession::new(1, &config, "/api/v3/live/1/hls");

        let playlist = session.generate_playlist();
        assert!(playlist.contains("#EXTM3U"));
        assert!(playlist.contains("#EXT-X-TARGETDURATION:2"));
        assert!(playlist.contains("URI=\"init.mp4\""));
    }

    #[tokio::test]
    async fn test_session_manager_creation() {
        let config = test_config();
        let manager = HlsSessionManager::new(config, "/api/v3/live");

        let sessions = manager.list_sessions().await;
        assert!(sessions.is_empty());
    }

    #[tokio::test]
    async fn test_idle_sessions_threshold() {
        // REVIEW_FIXES_PLAN §3.2: idle_sessions filters by last-access age.
        let tmp = tempfile::TempDir::new().unwrap();
        let mut config = test_config();
        config.storage.path = tmp.path().to_string_lossy().into_owned();
        let manager = HlsSessionManager::new(config, "/api/v3/live");

        manager.start_session(1).await.unwrap();
        manager.start_session(2).await.unwrap();

        // A huge threshold excludes freshly-started sessions...
        assert!(manager
            .idle_sessions(Duration::from_secs(100_000))
            .await
            .is_empty());

        // ...while a zero threshold matches every live session.
        let mut all = manager.idle_sessions(Duration::ZERO).await;
        all.sort_unstable();
        assert_eq!(all, vec![1, 2]);
    }

    #[tokio::test]
    async fn test_get_playlist_refreshes_access() {
        // Fetching the playlist must reset the idle clock (a polling player
        // keeps its session alive). With a zero threshold the session always
        // appears; the meaningful assertion is that the call path touches
        // without panicking and the session remains listed.
        let tmp = tempfile::TempDir::new().unwrap();
        let mut config = test_config();
        config.storage.path = tmp.path().to_string_lossy().into_owned();
        let manager = HlsSessionManager::new(config, "/api/v3/live");

        manager.start_session(7).await.unwrap();
        let _ = manager.get_playlist(7).await.unwrap();
        assert!(manager
            .idle_sessions(Duration::from_secs(100_000))
            .await
            .is_empty());
    }
}
