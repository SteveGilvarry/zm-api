//! Source Router for unified streaming source management
//!
//! Provides a unified abstraction over FIFO readers that serves all output protocols
//! (WebRTC, HLS, MSE). Manages lazy initialization of monitor sources and handles
//! both video and audio streams.

use std::sync::Arc;
use std::time::Duration;

use dashmap::DashMap;
use tokio::sync::{broadcast, watch, RwLock};
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};

use super::fifo::{FifoError, FifoPacket, VideoCodec, ZmFifoReader};
use crate::configure::streaming::ZoneMinderConfig;

/// Default broadcast channel capacity for source packets
const DEFAULT_SOURCE_CAPACITY: usize = 100;

/// Health state of the FIFO reader task.
///
/// Subscribers (e.g. the coordinator's processing task) can watch this to
/// detect reader failures instead of hanging on a broadcast channel that
/// never closes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReaderHealth {
    /// Reader not started yet
    Idle,
    /// Attempting to open the FIFO
    Opening,
    /// FIFO open and producing packets
    Active,
    /// FIFO closed or errored, attempting reconnect
    Reconnecting,
    /// Reader task exited (watch sender dropped)
    Stopped,
}

/// Represents an active monitor source with video and optional audio streams
pub struct MonitorSource {
    monitor_id: u32,
    /// Broadcast sender for video packets
    video_tx: broadcast::Sender<FifoPacket>,
    /// Broadcast sender for audio packets (if available)
    audio_tx: Option<broadcast::Sender<AudioPacket>>,
    /// Background reader task handle
    reader_handle: RwLock<Option<JoinHandle<()>>>,
    /// Detected video codec
    codec: RwLock<VideoCodec>,
    /// Whether the source is actively reading
    active: RwLock<bool>,
    /// Reader health watch — subscribers get notified on state transitions.
    /// When the reader task exits, the sender is dropped and receivers get Err.
    reader_health_tx: watch::Sender<ReaderHealth>,
    reader_health_rx: watch::Receiver<ReaderHealth>,
}

impl MonitorSource {
    /// Create a new monitor source
    fn new(monitor_id: u32, has_audio: bool) -> Self {
        let (video_tx, _) = broadcast::channel(DEFAULT_SOURCE_CAPACITY);
        let audio_tx = if has_audio {
            let (tx, _) = broadcast::channel(DEFAULT_SOURCE_CAPACITY);
            Some(tx)
        } else {
            None
        };
        let (reader_health_tx, reader_health_rx) = watch::channel(ReaderHealth::Idle);

        Self {
            monitor_id,
            video_tx,
            audio_tx,
            reader_handle: RwLock::new(None),
            codec: RwLock::new(VideoCodec::Unknown),
            active: RwLock::new(false),
            reader_health_tx,
            reader_health_rx,
        }
    }

    /// Subscribe to receive video packets
    pub fn subscribe_video(&self) -> broadcast::Receiver<FifoPacket> {
        self.video_tx.subscribe()
    }

    /// Subscribe to receive audio packets (if available)
    pub fn subscribe_audio(&self) -> Option<broadcast::Receiver<AudioPacket>> {
        self.audio_tx.as_ref().map(|tx| tx.subscribe())
    }

    /// Check if audio is available for this source
    pub fn has_audio(&self) -> bool {
        self.audio_tx.is_some()
    }

    /// Get the monitor ID
    pub fn monitor_id(&self) -> u32 {
        self.monitor_id
    }

    /// Get the detected video codec
    pub async fn codec(&self) -> VideoCodec {
        *self.codec.read().await
    }

    /// Check if the source is actively reading
    pub async fn is_active(&self) -> bool {
        *self.active.read().await
    }

    /// Subscribe to reader health state changes.
    ///
    /// Returns a watch receiver. When the reader task exits, the sender is
    /// dropped and `changed().await` returns `Err`, which the caller should
    /// interpret as `ReaderHealth::Stopped`.
    pub fn subscribe_reader_health(&self) -> watch::Receiver<ReaderHealth> {
        self.reader_health_rx.clone()
    }

    /// Get the number of video subscribers
    pub fn video_subscriber_count(&self) -> usize {
        self.video_tx.receiver_count()
    }

    /// Get the number of audio subscribers
    pub fn audio_subscriber_count(&self) -> Option<usize> {
        self.audio_tx.as_ref().map(|tx| tx.receiver_count())
    }
}

/// Audio packet from FIFO
#[derive(Debug, Clone)]
pub struct AudioPacket {
    pub monitor_id: u32,
    pub timestamp_us: i64,
    pub data: Vec<u8>,
    pub codec: AudioCodec,
}

/// Audio codec types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioCodec {
    Aac,
    G711Alaw,
    G711Ulaw,
    Opus,
    Unknown,
}

impl AudioCodec {
    pub fn as_str(&self) -> &'static str {
        match self {
            AudioCodec::Aac => "AAC",
            AudioCodec::G711Alaw => "G.711 A-law",
            AudioCodec::G711Ulaw => "G.711 u-law",
            AudioCodec::Opus => "Opus",
            AudioCodec::Unknown => "Unknown",
        }
    }
}

/// Router errors
#[derive(Debug, thiserror::Error)]
pub enum RouterError {
    #[error("Monitor {0} source not available")]
    SourceNotAvailable(u32),

    #[error("Monitor {0} FIFO not found")]
    FifoNotFound(u32),

    #[error("Failed to start reader for monitor {0}: {1}")]
    ReaderStartFailed(u32, String),

    #[error("FIFO error: {0}")]
    FifoError(#[from] FifoError),
}

/// Configuration for the source router
#[derive(Debug, Clone)]
pub struct RouterConfig {
    /// ZoneMinder FIFO configuration
    pub zoneminder: ZoneMinderConfig,
    /// Broadcast channel capacity
    pub channel_capacity: usize,
    /// Whether to automatically start readers on subscription
    pub auto_start: bool,
    /// Maximum number of active sources
    pub max_active_sources: usize,
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self {
            zoneminder: ZoneMinderConfig::default(),
            channel_capacity: DEFAULT_SOURCE_CAPACITY,
            auto_start: true,
            max_active_sources: 50,
        }
    }
}

impl RouterConfig {
    /// Create config from ZoneMinder config
    pub fn from_zoneminder(zm_config: ZoneMinderConfig) -> Self {
        Self {
            zoneminder: zm_config,
            ..Default::default()
        }
    }
}

/// Unified source router that manages FIFO readers and serves all output protocols
pub struct SourceRouter {
    /// Active monitor sources
    active_sources: DashMap<u32, Arc<MonitorSource>>,
    /// Configuration
    config: RouterConfig,
}

impl SourceRouter {
    /// Create a new source router with default configuration
    pub fn new() -> Self {
        Self::with_config(RouterConfig::default())
    }

    /// Create a new source router with custom configuration
    pub fn with_config(config: RouterConfig) -> Self {
        Self {
            active_sources: DashMap::new(),
            config,
        }
    }

    /// Create a source router from ZoneMinder configuration
    pub fn from_zoneminder_config(zm_config: ZoneMinderConfig) -> Self {
        Self::with_config(RouterConfig::from_zoneminder(zm_config))
    }

    /// Create a source for a monitor without starting the reader.
    ///
    /// Use this when you need to subscribe to the broadcast channel before
    /// packets start flowing (avoids losing initial SPS/PPS NAL units).
    /// Call `start_reader()` separately after subscribing.
    pub async fn create_source(&self, monitor_id: u32) -> Result<Arc<MonitorSource>, RouterError> {
        // Return existing if already created
        if let Some(source) = self.active_sources.get(&monitor_id) {
            return Ok(source.clone());
        }

        if self.active_sources.len() >= self.config.max_active_sources {
            warn!(
                "Max active sources ({}) reached, cannot add monitor {}",
                self.config.max_active_sources, monitor_id
            );
            return Err(RouterError::SourceNotAvailable(monitor_id));
        }

        let reader = ZmFifoReader::new(monitor_id, self.config.zoneminder.clone());
        if !reader.fifo_exists() {
            return Err(RouterError::FifoNotFound(monitor_id));
        }

        let has_audio = reader.audio_path().is_some_and(|p| p.exists());
        let source = Arc::new(MonitorSource::new(monitor_id, has_audio));
        self.active_sources.insert(monitor_id, source.clone());

        info!(
            "Created source for monitor {} (audio: {})",
            monitor_id, has_audio
        );
        Ok(source)
    }

    /// Get an existing source for a monitor without creating or starting one.
    ///
    /// Returns `None` if no source is currently active for this monitor.
    /// Useful for piggybacking on an already-running reader (e.g. snapshots).
    pub fn get_existing_source(&self, monitor_id: u32) -> Option<Arc<MonitorSource>> {
        self.active_sources.get(&monitor_id).map(|s| s.clone())
    }

    /// Get or create a source for a monitor - lazy initialization
    ///
    /// This will create a MonitorSource if one doesn't exist, and optionally
    /// start the background reader task if auto_start is enabled.
    pub async fn get_source(&self, monitor_id: u32) -> Result<Arc<MonitorSource>, RouterError> {
        // Check if already active
        if let Some(source) = self.active_sources.get(&monitor_id) {
            return Ok(source.clone());
        }

        let source = self.create_source(monitor_id).await?;

        // Start the reader if auto_start is enabled
        if self.config.auto_start {
            self.start_reader(monitor_id).await?;
        }

        Ok(source)
    }

    /// Start the background reader task for a monitor
    pub async fn start_reader(&self, monitor_id: u32) -> Result<(), RouterError> {
        let source = self
            .active_sources
            .get(&monitor_id)
            .ok_or(RouterError::SourceNotAvailable(monitor_id))?
            .clone();

        // Check if already active
        if *source.active.read().await {
            debug!("Reader already active for monitor {}", monitor_id);
            return Ok(());
        }

        // Create and start the reader task
        let config = self.config.zoneminder.clone();
        let video_tx = source.video_tx.clone();
        let source_for_task = source.clone();
        // Clone the health sender into the task — when the task exits (or is
        // aborted), this sender is dropped and subscribers see Err from changed().
        let health_tx = source.reader_health_tx.clone();

        let handle = tokio::spawn(async move {
            info!("Starting FIFO reader task for monitor {}", monitor_id);

            // Outer loop: handles reconnection when FIFO closes or errors
            loop {
                let mut reader = ZmFifoReader::new(monitor_id, config.clone());
                let _ = health_tx.send(ReaderHealth::Opening);

                // Open the FIFO (with O_RDWR this won't block)
                match reader.open().await {
                    Ok(()) => {
                        info!("FIFO opened for monitor {}", monitor_id);
                        *source_for_task.active.write().await = true;
                        let _ = health_tx.send(ReaderHealth::Active);
                    }
                    Err(FifoError::NotFound { .. }) => {
                        debug!(
                            "FIFO not found for monitor {}, waiting to retry...",
                            monitor_id
                        );
                        tokio::time::sleep(Duration::from_millis(config.reconnect_delay_ms * 5))
                            .await;
                        continue;
                    }
                    Err(e) => {
                        error!("Failed to open FIFO for monitor {}: {}", monitor_id, e);
                        tokio::time::sleep(Duration::from_millis(config.reconnect_delay_ms)).await;
                        continue;
                    }
                }

                // Inner loop: read packets until FIFO closes or unrecoverable error
                loop {
                    match reader.read_packet().await {
                        Ok(packet) => {
                            // Update codec if detected
                            if packet.codec != VideoCodec::Unknown {
                                let mut codec_guard = source_for_task.codec.write().await;
                                if *codec_guard == VideoCodec::Unknown {
                                    *codec_guard = packet.codec;
                                    info!(
                                        "Detected codec for monitor {}: {}",
                                        monitor_id,
                                        packet.codec.as_str()
                                    );
                                }
                            }

                            // Broadcast the packet (ignore errors if no receivers)
                            if video_tx.send(packet).is_err() {
                                // No receivers - this is fine, just means no one is subscribed
                                debug!("No receivers for monitor {}", monitor_id);
                            }
                        }
                        Err(FifoError::Timeout { .. }) => {
                            // Timeout is expected when no data is available
                            debug!("Read timeout for monitor {}, continuing...", monitor_id);
                            continue;
                        }
                        Err(FifoError::Closed) => {
                            warn!("FIFO closed for monitor {}, will reconnect...", monitor_id);
                            break; // Break inner loop to reconnect
                        }
                        Err(e) => {
                            error!("Error reading from FIFO for monitor {}: {}", monitor_id, e);
                            tokio::time::sleep(Duration::from_millis(config.reconnect_delay_ms))
                                .await;
                            break; // Break inner loop to reconnect with fresh reader
                        }
                    }
                }

                // Signal reconnecting before the delay
                *source_for_task.active.write().await = false;
                let _ = health_tx.send(ReaderHealth::Reconnecting);

                // Brief delay before reconnecting
                tokio::time::sleep(Duration::from_millis(config.reconnect_delay_ms)).await;
            }

            // Reached if the task is aborted — health_tx is dropped, signaling Stopped
        });

        *source.reader_handle.write().await = Some(handle);
        Ok(())
    }

    /// Stop the reader for a monitor
    pub async fn stop_reader(&self, monitor_id: u32) -> Result<(), RouterError> {
        let source = self
            .active_sources
            .get(&monitor_id)
            .ok_or(RouterError::SourceNotAvailable(monitor_id))?;

        let mut handle_guard = source.reader_handle.write().await;
        if let Some(handle) = handle_guard.take() {
            handle.abort();
            info!("Stopped reader for monitor {}", monitor_id);
        }

        *source.active.write().await = false;
        Ok(())
    }

    /// Remove a source completely
    pub async fn remove_source(&self, monitor_id: u32) -> Result<(), RouterError> {
        // Stop the reader first
        if self.active_sources.contains_key(&monitor_id) {
            self.stop_reader(monitor_id).await?;
        }

        self.active_sources.remove(&monitor_id);
        info!("Removed source for monitor {}", monitor_id);
        Ok(())
    }

    /// Subscribe to video packets for a monitor
    ///
    /// This is a convenience method that gets or creates the source and subscribes.
    pub async fn subscribe_video(
        &self,
        monitor_id: u32,
    ) -> Result<broadcast::Receiver<FifoPacket>, RouterError> {
        let source = self.get_source(monitor_id).await?;
        Ok(source.subscribe_video())
    }

    /// Subscribe to audio packets for a monitor
    pub async fn subscribe_audio(
        &self,
        monitor_id: u32,
    ) -> Result<Option<broadcast::Receiver<AudioPacket>>, RouterError> {
        let source = self.get_source(monitor_id).await?;
        Ok(source.subscribe_audio())
    }

    /// Check if a monitor's FIFO is available (without creating a source)
    pub fn is_available(&self, monitor_id: u32) -> bool {
        let reader = ZmFifoReader::new(monitor_id, self.config.zoneminder.clone());
        reader.fifo_exists()
    }

    /// Get the list of active monitor IDs
    pub fn active_monitor_ids(&self) -> Vec<u32> {
        self.active_sources
            .iter()
            .map(|entry| *entry.key())
            .collect()
    }

    /// Get the number of active sources
    pub fn active_source_count(&self) -> usize {
        self.active_sources.len()
    }

    /// Get statistics for all active sources
    pub async fn stats(&self) -> Vec<SourceStats> {
        let mut stats = Vec::new();

        for entry in self.active_sources.iter() {
            let source = entry.value();
            stats.push(SourceStats {
                monitor_id: source.monitor_id,
                codec: source.codec().await,
                active: source.is_active().await,
                video_subscribers: source.video_subscriber_count(),
                audio_subscribers: source.audio_subscriber_count(),
                has_audio: source.has_audio(),
            });
        }

        stats
    }

    /// Get statistics for a specific monitor
    pub async fn get_source_stats(&self, monitor_id: u32) -> Option<SourceStats> {
        let source = self.active_sources.get(&monitor_id)?;
        Some(SourceStats {
            monitor_id: source.monitor_id,
            codec: source.codec().await,
            active: source.is_active().await,
            video_subscribers: source.video_subscriber_count(),
            audio_subscribers: source.audio_subscriber_count(),
            has_audio: source.has_audio(),
        })
    }
}

impl Default for SourceRouter {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics for a single source
#[derive(Debug, Clone, serde::Serialize)]
pub struct SourceStats {
    pub monitor_id: u32,
    pub codec: VideoCodec,
    pub active: bool,
    pub video_subscribers: usize,
    pub audio_subscribers: Option<usize>,
    pub has_audio: bool,
}

// Implement Serialize for VideoCodec (needed for SourceStats)
impl serde::Serialize for VideoCodec {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_config_default() {
        let config = RouterConfig::default();
        assert!(config.auto_start);
        assert_eq!(config.channel_capacity, 100);
        assert_eq!(config.max_active_sources, 50);
    }

    #[test]
    fn test_router_creation() {
        let router = SourceRouter::new();
        assert_eq!(router.active_source_count(), 0);
        assert!(router.active_monitor_ids().is_empty());
    }

    #[test]
    fn test_audio_codec_as_str() {
        assert_eq!(AudioCodec::Aac.as_str(), "AAC");
        assert_eq!(AudioCodec::G711Alaw.as_str(), "G.711 A-law");
        assert_eq!(AudioCodec::G711Ulaw.as_str(), "G.711 u-law");
        assert_eq!(AudioCodec::Opus.as_str(), "Opus");
        assert_eq!(AudioCodec::Unknown.as_str(), "Unknown");
    }

    #[test]
    fn test_monitor_source_creation() {
        let source = MonitorSource::new(1, true);
        assert_eq!(source.monitor_id(), 1);
        assert!(source.has_audio());
        assert_eq!(source.video_subscriber_count(), 0);
    }

    #[test]
    fn test_monitor_source_without_audio() {
        let source = MonitorSource::new(2, false);
        assert!(!source.has_audio());
        assert!(source.subscribe_audio().is_none());
    }

    #[tokio::test]
    async fn test_monitor_source_initial_state() {
        let source = MonitorSource::new(1, false);
        assert!(!source.is_active().await);
        assert_eq!(source.codec().await, VideoCodec::Unknown);
    }

    #[test]
    fn test_is_available_nonexistent() {
        let router = SourceRouter::new();
        // This will return false for a non-existent monitor
        // since the FIFO won't exist
        assert!(!router.is_available(99999));
    }
}
