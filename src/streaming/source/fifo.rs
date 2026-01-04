use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, BufReader};
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};

use crate::configure::streaming::ZoneMinderConfig;

/// Default broadcast channel capacity for FIFO packets
const DEFAULT_BROADCAST_CAPACITY: usize = 100;

/// Video codec detected from FIFO data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoCodec {
    H264,
    H265,
    Unknown,
}

impl VideoCodec {
    pub fn as_str(&self) -> &'static str {
        match self {
            VideoCodec::H264 => "H264",
            VideoCodec::H265 => "H265",
            VideoCodec::Unknown => "Unknown",
        }
    }
}

/// Audio codec detected from FIFO data
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

/// A packet read from the FIFO
#[derive(Debug, Clone)]
pub struct FifoPacket {
    pub monitor_id: u32,
    pub timestamp_us: u64,
    pub data: Vec<u8>,
    pub is_keyframe: bool,
    pub codec: VideoCodec,
}

/// Reader for ZoneMinder's video FIFO
pub struct ZmFifoReader {
    monitor_id: u32,
    video_path: PathBuf,
    audio_path: Option<PathBuf>,
    video_reader: Option<BufReader<File>>,
    codec: VideoCodec,
    config: ZoneMinderConfig,
    broadcast_capacity: usize,
    /// Broadcast channel for distributing packets to multiple consumers
    tx: broadcast::Sender<FifoPacket>,
}

impl ZmFifoReader {
    /// Create a new FIFO reader for a monitor
    ///
    /// # Arguments
    /// * `monitor_id` - The ZoneMinder monitor ID
    /// * `config` - Configuration for FIFO paths and behavior
    pub fn new(monitor_id: u32, config: ZoneMinderConfig) -> Self {
        Self::with_capacity(monitor_id, config, DEFAULT_BROADCAST_CAPACITY)
    }

    /// Create a new FIFO reader with custom broadcast capacity
    ///
    /// # Arguments
    /// * `monitor_id` - The ZoneMinder monitor ID
    /// * `config` - Configuration for FIFO paths and behavior
    /// * `broadcast_capacity` - Channel capacity for packet broadcasting
    pub fn with_capacity(
        monitor_id: u32,
        config: ZoneMinderConfig,
        broadcast_capacity: usize,
    ) -> Self {
        // Construct paths: /dev/shm/{monitor_id}-v.fifo
        let video_path = PathBuf::from(&config.fifo_base_path)
            .join(format!("{}{}", monitor_id, config.video_fifo_suffix));

        let audio_path = if !config.audio_fifo_suffix.is_empty() {
            Some(
                PathBuf::from(&config.fifo_base_path)
                    .join(format!("{}{}", monitor_id, config.audio_fifo_suffix)),
            )
        } else {
            None
        };

        let (tx, _rx) = broadcast::channel(broadcast_capacity);

        Self {
            monitor_id,
            video_path,
            audio_path,
            video_reader: None,
            codec: VideoCodec::Unknown,
            config,
            broadcast_capacity,
            tx,
        }
    }

    /// Create a new FIFO reader with custom paths
    ///
    /// # Arguments
    /// * `monitor_id` - The ZoneMinder monitor ID
    /// * `fifo_base_path` - Base path like "/dev/shm"
    /// * `video_suffix` - FIFO suffix like "-v.fifo"
    /// * `audio_suffix` - Audio FIFO suffix (optional)
    pub fn with_custom_paths(
        monitor_id: u32,
        fifo_base_path: &str,
        video_suffix: &str,
        audio_suffix: Option<&str>,
    ) -> Self {
        let config = ZoneMinderConfig {
            enabled: true,
            fifo_base_path: fifo_base_path.to_string(),
            video_fifo_suffix: video_suffix.to_string(),
            audio_fifo_suffix: audio_suffix.unwrap_or("").to_string(),
            fifo_read_timeout_ms: 5000,
            reconnect_delay_ms: 1000,
        };

        Self::new(monitor_id, config)
    }

    /// Open the FIFO for reading
    /// Returns error if FIFO doesn't exist or can't be opened
    pub async fn open(&mut self) -> Result<(), FifoError> {
        if !self.fifo_exists() {
            return Err(FifoError::NotFound {
                path: self.video_path.clone(),
            });
        }

        info!(
            "Opening FIFO for monitor {}: {}",
            self.monitor_id,
            self.video_path.display()
        );

        // Open the video FIFO
        let file = File::open(&self.video_path).await?;
        self.video_reader = Some(BufReader::new(file));

        info!("Successfully opened FIFO for monitor {}", self.monitor_id);
        Ok(())
    }

    /// Check if FIFOs exist for this monitor
    pub fn fifo_exists(&self) -> bool {
        self.video_path.exists()
    }

    /// Read a single packet from the video FIFO
    /// This will block until data is available or timeout
    pub async fn read_packet(&mut self) -> Result<FifoPacket, FifoError> {
        // Read with timeout
        let timeout = Duration::from_millis(self.config.fifo_read_timeout_ms);
        let timeout_ms = self.config.fifo_read_timeout_ms;

        tokio::time::timeout(timeout, self.read_packet_internal())
            .await
            .map_err(|_| FifoError::Timeout { timeout_ms })?
    }

    /// Internal packet reading logic
    async fn read_packet_internal(&mut self) -> Result<FifoPacket, FifoError> {
        let reader = self
            .video_reader
            .as_mut()
            .ok_or(FifoError::NotCapturing)?;
        // Read 4 bytes: NAL unit length (big-endian u32)
        let length = reader.read_u32().await?;

        if length == 0 {
            return Err(FifoError::Closed);
        }

        // Read 4 bytes: timestamp in microseconds (big-endian u32)
        // Note: ZoneMinder may use u32 or u64 depending on configuration
        let timestamp_us = reader.read_u32().await? as u64;

        // Read `length` bytes: NAL unit data
        let mut data = vec![0u8; length as usize];
        reader.read_exact(&mut data).await?;

        // Detect codec from NAL unit if not yet detected
        if self.codec == VideoCodec::Unknown {
            self.codec = Self::detect_codec(&data);
            info!(
                "Detected codec for monitor {}: {}",
                self.monitor_id,
                self.codec.as_str()
            );
        }

        // Check if this is a keyframe
        let is_keyframe = Self::is_keyframe(&data, self.codec);

        debug!(
            "Read packet for monitor {}: {} bytes, timestamp: {}us, keyframe: {}, codec: {}",
            self.monitor_id,
            data.len(),
            timestamp_us,
            is_keyframe,
            self.codec.as_str()
        );

        Ok(FifoPacket {
            monitor_id: self.monitor_id,
            timestamp_us,
            data,
            is_keyframe,
            codec: self.codec,
        })
    }

    /// Subscribe to receive packets via broadcast channel
    pub fn subscribe(&self) -> broadcast::Receiver<FifoPacket> {
        self.tx.subscribe()
    }

    /// Start a background task that reads from FIFO and broadcasts packets
    /// Returns a JoinHandle for the background task
    pub fn start_reader_task(mut self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            info!(
                "Starting FIFO reader task for monitor {}",
                self.monitor_id
            );

            // Open the FIFO
            if let Err(e) = self.open().await {
                error!(
                    "Failed to open FIFO for monitor {}: {}",
                    self.monitor_id, e
                );
                return;
            }

            // Read packets in a loop
            loop {
                match self.read_packet().await {
                    Ok(packet) => {
                        // Broadcast the packet to all subscribers
                        // Ignore send errors (no receivers is ok)
                        let _ = self.tx.send(packet);
                    }
                    Err(FifoError::Timeout { .. }) => {
                        // Timeout is expected when no data is available
                        debug!(
                            "Read timeout for monitor {}, continuing...",
                            self.monitor_id
                        );
                        continue;
                    }
                    Err(FifoError::Closed) => {
                        warn!("FIFO closed for monitor {}, exiting", self.monitor_id);
                        break;
                    }
                    Err(e) => {
                        error!("Error reading from FIFO for monitor {}: {}", self.monitor_id, e);
                        // Small delay before retrying
                        tokio::time::sleep(Duration::from_millis(
                            self.config.reconnect_delay_ms,
                        ))
                        .await;
                    }
                }
            }

            info!(
                "FIFO reader task stopped for monitor {}",
                self.monitor_id
            );
        })
    }

    /// Detect codec from NAL unit header
    ///
    /// H.264: NAL unit type is in bits 0-4 of first byte after start code
    /// H.265: NAL unit type is in bits 1-6 of first byte after start code
    fn detect_codec(nal_data: &[u8]) -> VideoCodec {
        if nal_data.len() < 5 {
            return VideoCodec::Unknown;
        }

        // Check for NAL start code (0x00 0x00 0x00 0x01)
        let start_code_offset = if nal_data.starts_with(&[0x00, 0x00, 0x00, 0x01]) {
            4
        } else if nal_data.starts_with(&[0x00, 0x00, 0x01]) {
            3
        } else {
            0
        };

        if start_code_offset == 0 || nal_data.len() <= start_code_offset {
            return VideoCodec::Unknown;
        }

        let first_byte = nal_data[start_code_offset];

        // H.264 NAL unit types:
        // Bits 0-4: NAL type
        // Type 7 = SPS (Sequence Parameter Set) - H.264 specific
        // Type 8 = PPS (Picture Parameter Set) - H.264 specific
        let h264_nal_type = first_byte & 0x1F;
        if h264_nal_type == 7 || h264_nal_type == 8 || h264_nal_type == 5 {
            return VideoCodec::H264;
        }

        // H.265 NAL unit types:
        // Bits 1-6: NAL type (bit 0 is forbidden_zero_bit)
        let h265_nal_type = (first_byte >> 1) & 0x3F;
        // VPS = 32, SPS = 33, PPS = 34 (H.265 specific)
        if h265_nal_type >= 32 && h265_nal_type <= 34 {
            return VideoCodec::H265;
        }

        // Default to H.264 as it's more common in ZoneMinder
        VideoCodec::H264
    }

    /// Check if NAL unit is a keyframe
    ///
    /// H.264: Type 5 = IDR (Instantaneous Decoder Refresh) - keyframe
    /// H.265: Types 16-21 = IRAP (Intra Random Access Point) - keyframe
    fn is_keyframe(nal_data: &[u8], codec: VideoCodec) -> bool {
        if nal_data.len() < 5 {
            return false;
        }

        // Check for NAL start code
        let start_code_offset = if nal_data.starts_with(&[0x00, 0x00, 0x00, 0x01]) {
            4
        } else if nal_data.starts_with(&[0x00, 0x00, 0x01]) {
            3
        } else {
            0
        };

        if start_code_offset == 0 || nal_data.len() <= start_code_offset {
            return false;
        }

        let first_byte = nal_data[start_code_offset];

        match codec {
            VideoCodec::H264 => {
                // H.264: NAL type 5 is IDR frame (keyframe)
                let nal_type = first_byte & 0x1F;
                nal_type == 5
            }
            VideoCodec::H265 => {
                // H.265: NAL types 16-21 are IRAP frames (keyframes)
                let nal_type = (first_byte >> 1) & 0x3F;
                (16..=21).contains(&nal_type)
            }
            VideoCodec::Unknown => false,
        }
    }

    /// Get the current codec
    pub fn codec(&self) -> VideoCodec {
        self.codec
    }

    /// Get the monitor ID
    pub fn monitor_id(&self) -> u32 {
        self.monitor_id
    }

    /// Get the video FIFO path
    pub fn video_path(&self) -> &Path {
        &self.video_path
    }

    /// Get the audio FIFO path (if configured)
    pub fn audio_path(&self) -> Option<&Path> {
        self.audio_path.as_deref()
    }
}

/// Errors that can occur when reading from FIFO
#[derive(Debug, thiserror::Error)]
pub enum FifoError {
    #[error("FIFO not found: {path}")]
    NotFound { path: PathBuf },

    #[error("Failed to open FIFO: {0}")]
    OpenError(#[from] std::io::Error),

    #[error("Read timeout after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },

    #[error("FIFO closed by writer")]
    Closed,

    #[error("Invalid packet format")]
    InvalidFormat,

    #[error("Monitor not capturing")]
    NotCapturing,
}

/// Manager for multiple FIFO readers
pub struct FifoManager {
    readers: HashMap<u32, ZmFifoReader>,
    config: ZoneMinderConfig,
    broadcast_capacity: usize,
}

impl FifoManager {
    /// Create a new FIFO manager
    pub fn new(config: ZoneMinderConfig) -> Self {
        Self::with_capacity(config, DEFAULT_BROADCAST_CAPACITY)
    }

    /// Create a new FIFO manager with custom broadcast capacity
    pub fn with_capacity(config: ZoneMinderConfig, broadcast_capacity: usize) -> Self {
        Self {
            readers: HashMap::new(),
            config,
            broadcast_capacity,
        }
    }

    /// Create a FIFO manager with default configuration
    pub fn with_defaults() -> Self {
        Self::new(ZoneMinderConfig::default())
    }

    /// Get or create a FIFO reader for a monitor
    pub async fn get_reader(&mut self, monitor_id: u32) -> Result<&mut ZmFifoReader, FifoError> {
        if !self.readers.contains_key(&monitor_id) {
            let mut reader =
                ZmFifoReader::with_capacity(monitor_id, self.config.clone(), self.broadcast_capacity);
            reader.open().await?;
            self.readers.insert(monitor_id, reader);
        }

        Ok(self.readers.get_mut(&monitor_id).unwrap())
    }

    /// Subscribe to packets from a specific monitor
    pub async fn subscribe(
        &mut self,
        monitor_id: u32,
    ) -> Result<broadcast::Receiver<FifoPacket>, FifoError> {
        let reader = self.get_reader(monitor_id).await?;
        Ok(reader.subscribe())
    }

    /// Check if a monitor's FIFO is available
    pub fn is_available(&self, monitor_id: u32) -> bool {
        if let Some(reader) = self.readers.get(&monitor_id) {
            reader.fifo_exists()
        } else {
            let reader = ZmFifoReader::new(monitor_id, self.config.clone());
            reader.fifo_exists()
        }
    }

    /// Remove a reader from the manager
    pub fn remove_reader(&mut self, monitor_id: u32) -> Option<ZmFifoReader> {
        self.readers.remove(&monitor_id)
    }

    /// Get the number of active readers
    pub fn active_readers(&self) -> usize {
        self.readers.len()
    }

    /// Get all monitor IDs with active readers
    pub fn active_monitor_ids(&self) -> Vec<u32> {
        self.readers.keys().copied().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_h264_codec() {
        // H.264 SPS NAL unit (type 7)
        let h264_sps = vec![0x00, 0x00, 0x00, 0x01, 0x67, 0x42, 0x00, 0x1F];
        assert_eq!(ZmFifoReader::detect_codec(&h264_sps), VideoCodec::H264);

        // H.264 PPS NAL unit (type 8)
        let h264_pps = vec![0x00, 0x00, 0x00, 0x01, 0x68, 0xCE, 0x3C, 0x80];
        assert_eq!(ZmFifoReader::detect_codec(&h264_pps), VideoCodec::H264);

        // H.264 IDR frame (type 5)
        let h264_idr = vec![0x00, 0x00, 0x00, 0x01, 0x65, 0x88, 0x84, 0x00];
        assert_eq!(ZmFifoReader::detect_codec(&h264_idr), VideoCodec::H264);
    }

    #[test]
    fn test_detect_h265_codec() {
        // H.265 VPS NAL unit (type 32)
        let h265_vps = vec![0x00, 0x00, 0x00, 0x01, 0x40, 0x01, 0x0C, 0x01];
        assert_eq!(ZmFifoReader::detect_codec(&h265_vps), VideoCodec::H265);

        // H.265 SPS NAL unit (type 33)
        let h265_sps = vec![0x00, 0x00, 0x00, 0x01, 0x42, 0x01, 0x01, 0x01];
        assert_eq!(ZmFifoReader::detect_codec(&h265_sps), VideoCodec::H265);
    }

    #[test]
    fn test_is_h264_keyframe() {
        // H.264 IDR frame (type 5) - keyframe
        let h264_idr = vec![0x00, 0x00, 0x00, 0x01, 0x65, 0x88, 0x84, 0x00];
        assert!(ZmFifoReader::is_keyframe(&h264_idr, VideoCodec::H264));

        // H.264 non-IDR frame (type 1) - not keyframe
        let h264_non_idr = vec![0x00, 0x00, 0x00, 0x01, 0x41, 0x9A, 0x21, 0x58];
        assert!(!ZmFifoReader::is_keyframe(
            &h264_non_idr,
            VideoCodec::H264
        ));
    }

    #[test]
    fn test_is_h265_keyframe() {
        // H.265 IDR frame (type 19) - keyframe
        let h265_idr = vec![0x00, 0x00, 0x00, 0x01, 0x26, 0x01, 0xAF, 0x08];
        assert!(ZmFifoReader::is_keyframe(&h265_idr, VideoCodec::H265));

        // H.265 non-IRAP frame (type 1) - not keyframe
        let h265_non_irap = vec![0x00, 0x00, 0x00, 0x01, 0x02, 0x01, 0xD0, 0x00];
        assert!(!ZmFifoReader::is_keyframe(
            &h265_non_irap,
            VideoCodec::H265
        ));
    }

    #[test]
    fn test_config_default() {
        let config = ZoneMinderConfig::default();
        assert_eq!(config.fifo_base_path, "/dev/shm");
        assert_eq!(config.video_fifo_suffix, "-v.fifo");
        assert_eq!(config.audio_fifo_suffix, "-a.fifo");
        assert_eq!(config.fifo_read_timeout_ms, 5000);
        assert_eq!(config.reconnect_delay_ms, 1000);
    }

    #[test]
    fn test_fifo_reader_creation() {
        let config = ZoneMinderConfig::default();
        let reader = ZmFifoReader::new(1, config);

        assert_eq!(reader.monitor_id(), 1);
        assert_eq!(reader.codec(), VideoCodec::Unknown);
        assert_eq!(reader.video_path(), Path::new("/dev/shm/1-v.fifo"));
        assert_eq!(reader.audio_path(), Some(Path::new("/dev/shm/1-a.fifo")));
    }

    #[test]
    fn test_fifo_reader_custom_paths() {
        let reader = ZmFifoReader::with_custom_paths(42, "/tmp", ".video", Some(".audio"));

        assert_eq!(reader.monitor_id(), 42);
        assert_eq!(reader.video_path(), Path::new("/tmp/42.video"));
        assert_eq!(reader.audio_path(), Some(Path::new("/tmp/42.audio")));
    }

    #[test]
    fn test_fifo_manager_creation() {
        let manager = FifoManager::with_defaults();
        assert_eq!(manager.active_readers(), 0);
        assert!(manager.active_monitor_ids().is_empty());
    }

    #[test]
    fn test_codec_as_str() {
        assert_eq!(VideoCodec::H264.as_str(), "H264");
        assert_eq!(VideoCodec::H265.as_str(), "H265");
        assert_eq!(VideoCodec::Unknown.as_str(), "Unknown");
    }

    #[test]
    fn test_audio_codec_as_str() {
        assert_eq!(AudioCodec::Aac.as_str(), "AAC");
        assert_eq!(AudioCodec::G711Alaw.as_str(), "G.711 A-law");
        assert_eq!(AudioCodec::G711Ulaw.as_str(), "G.711 u-law");
        assert_eq!(AudioCodec::Opus.as_str(), "Opus");
        assert_eq!(AudioCodec::Unknown.as_str(), "Unknown");
    }
}
