use std::collections::HashMap;
use std::io::Read as _;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tokio::io::unix::AsyncFd;
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
    pub timestamp_us: i64,
    pub data: Vec<u8>,
    pub is_keyframe: bool,
    pub codec: VideoCodec,
}

/// Maximum NAL accumulation buffer size (4 MB) before forced reset
const MAX_NAL_BUF_SIZE: usize = 4 * 1024 * 1024;
/// Read buffer size for FIFO reads
const FIFO_READ_BUF_SIZE: usize = 32768;

/// Reader for ZoneMinder's video FIFO
///
/// ZoneMinder writes raw Annex B H.264/H.265 byte streams to the FIFO
/// (no framing headers). This reader accumulates bytes and extracts
/// complete NAL units by scanning for start codes (00 00 00 01 / 00 00 01).
pub struct ZmFifoReader {
    monitor_id: u32,
    video_path: PathBuf,
    audio_path: Option<PathBuf>,
    video_reader: Option<AsyncFd<std::fs::File>>,
    codec: VideoCodec,
    config: ZoneMinderConfig,
    #[allow(dead_code)]
    broadcast_capacity: usize,
    /// Broadcast channel for distributing packets to multiple consumers
    tx: broadcast::Sender<FifoPacket>,
    /// Accumulation buffer for raw Annex B byte stream
    nal_buf: Vec<u8>,
    /// Reader start time for generating monotonic timestamps
    start_time: Instant,
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
        let video_path;
        let audio_path;
        let mut detected_codec = VideoCodec::Unknown;

        // Check if using new ZoneMinder format (base path is /run/zm)
        // New format: /run/zm/video_fifo_{id}.{codec}
        // Old format: /dev/shm/{id}-v.fifo
        let is_new_format = config.fifo_base_path == "/run/zm"
            || config.video_fifo_suffix.starts_with("/video_fifo_");

        if is_new_format {
            // New ZoneMinder format - try to detect codec by checking file existence
            let possible_video_extensions = ["h264", "hevc", "h265"];
            let mut found_video = None;

            for ext in &possible_video_extensions {
                let path = PathBuf::from(&config.fifo_base_path)
                    .join(format!("video_fifo_{}.{}", monitor_id, ext));
                if path.exists() {
                    found_video = Some(path);
                    detected_codec = match *ext {
                        "h264" => VideoCodec::H264,
                        "hevc" | "h265" => VideoCodec::H265,
                        _ => VideoCodec::Unknown,
                    };
                    break;
                }
            }

            // If no file found, default to h264
            video_path = found_video.unwrap_or_else(|| {
                PathBuf::from(&config.fifo_base_path)
                    .join(format!("video_fifo_{}.h264", monitor_id))
            });

            // Audio FIFO path: {base_path}/audio_fifo_{monitor_id}.{codec}
            audio_path = if !config.audio_fifo_suffix.is_empty() {
                let possible_audio_extensions = ["aac", "pcm_alaw"];
                let mut found_audio = None;
                for ext in &possible_audio_extensions {
                    let path = PathBuf::from(&config.fifo_base_path)
                        .join(format!("audio_fifo_{}.{}", monitor_id, ext));
                    if path.exists() {
                        found_audio = Some(path);
                        break;
                    }
                }
                // Default to aac if not found
                Some(found_audio.unwrap_or_else(|| {
                    PathBuf::from(&config.fifo_base_path)
                        .join(format!("audio_fifo_{}.aac", monitor_id))
                }))
            } else {
                None
            };
        } else {
            // Old custom format: {base_path}/{monitor_id}{suffix}
            video_path = PathBuf::from(&config.fifo_base_path)
                .join(format!("{}{}", monitor_id, config.video_fifo_suffix));

            audio_path = if !config.audio_fifo_suffix.is_empty() {
                Some(
                    PathBuf::from(&config.fifo_base_path)
                        .join(format!("{}{}", monitor_id, config.audio_fifo_suffix)),
                )
            } else {
                None
            };
        }

        let (tx, _rx) = broadcast::channel(broadcast_capacity);

        Self {
            monitor_id,
            video_path,
            audio_path,
            video_reader: None,
            codec: detected_codec,
            config,
            broadcast_capacity,
            tx,
            nal_buf: Vec::with_capacity(FIFO_READ_BUF_SIZE * 2),
            start_time: Instant::now(),
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
            events_dir: "/var/lib/zoneminder/events".to_string(),
        };

        Self::new(monitor_id, config)
    }

    /// Open the FIFO for reading
    /// Returns error if FIFO doesn't exist or can't be opened
    ///
    /// Uses `O_RDWR` to avoid two problems with `O_RDONLY` on named pipes:
    /// 1. `O_RDONLY` blocks until a writer opens the pipe
    /// 2. `O_RDONLY` causes EOF when the writer disconnects
    ///
    /// With `O_RDWR`, the process holds both ends, so open returns immediately
    /// and reads block only until data is available (no spurious EOF).
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

        // Open with O_RDWR | O_NONBLOCK:
        // - O_RDWR: prevents blocking on open and spurious EOF when writer closes
        // - O_NONBLOCK: enables non-blocking reads so tokio's AsyncFd can poll via epoll
        let video_path = self.video_path.clone();
        let std_file = tokio::task::spawn_blocking(move || {
            std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .custom_flags(libc::O_NONBLOCK)
                .open(&video_path)
        })
        .await
        .map_err(|e| FifoError::OpenError(std::io::Error::other(e)))?
        .map_err(FifoError::OpenError)?;

        let async_fd = AsyncFd::new(std_file).map_err(FifoError::OpenError)?;
        self.video_reader = Some(async_fd);

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
    ///
    /// Reads raw bytes from the FIFO using AsyncFd for epoll-based readiness
    /// notification. This avoids blocking tokio's thread pool on FIFO reads
    /// and ensures timeouts can cancel pending reads.
    async fn read_packet_internal(&mut self) -> Result<FifoPacket, FifoError> {
        let async_fd = self.video_reader.as_ref().ok_or(FifoError::NotCapturing)?;

        loop {
            // Try to extract a complete NAL unit from the accumulation buffer
            if let Some(nal_data) = extract_next_nal(&mut self.nal_buf) {
                // Detect codec if not yet known
                if self.codec == VideoCodec::Unknown {
                    self.codec = Self::detect_codec(&nal_data);
                    if self.codec != VideoCodec::Unknown {
                        info!(
                            "Detected codec for monitor {}: {}",
                            self.monitor_id,
                            self.codec.as_str()
                        );
                    }
                }

                let is_keyframe = Self::is_keyframe(&nal_data, self.codec);
                let timestamp_us = self.start_time.elapsed().as_micros() as i64;

                debug!(
                    "Read NAL for monitor {}: {} bytes, keyframe: {}, codec: {}",
                    self.monitor_id,
                    nal_data.len(),
                    is_keyframe,
                    self.codec.as_str()
                );

                return Ok(FifoPacket {
                    monitor_id: self.monitor_id,
                    timestamp_us,
                    data: nal_data,
                    is_keyframe,
                    codec: self.codec,
                });
            }

            // Need more data — wait for FIFO to become readable via epoll,
            // then perform a non-blocking read. This is cancel-safe: if the
            // timeout fires, the future is dropped without leaving a blocked
            // thread in the pool.
            let n = loop {
                let mut guard = async_fd.readable().await?;
                let mut buf = [0u8; FIFO_READ_BUF_SIZE];
                match guard.try_io(|fd| fd.get_ref().read(&mut buf)) {
                    Ok(Ok(0)) => return Err(FifoError::Closed),
                    Ok(Ok(n)) => {
                        self.nal_buf.extend_from_slice(&buf[..n]);
                        break n;
                    }
                    Ok(Err(e)) => return Err(FifoError::OpenError(e)),
                    Err(_would_block) => continue, // spurious wakeup, retry
                }
            };

            debug!("Read {} bytes from FIFO for monitor {}", n, self.monitor_id);

            // Prevent unbounded buffer growth (e.g., corrupt stream with no start codes)
            if self.nal_buf.len() > MAX_NAL_BUF_SIZE {
                warn!(
                    "NAL buffer overflow ({} bytes) for monitor {}, resetting",
                    self.nal_buf.len(),
                    self.monitor_id
                );
                self.nal_buf.clear();
            }
        }
    }

    /// Subscribe to receive packets via broadcast channel
    pub fn subscribe(&self) -> broadcast::Receiver<FifoPacket> {
        self.tx.subscribe()
    }

    /// Start a background task that reads from FIFO and broadcasts packets
    /// Returns a JoinHandle for the background task
    pub fn start_reader_task(mut self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            info!("Starting FIFO reader task for monitor {}", self.monitor_id);

            // Open the FIFO
            if let Err(e) = self.open().await {
                error!("Failed to open FIFO for monitor {}: {}", self.monitor_id, e);
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
                        error!(
                            "Error reading from FIFO for monitor {}: {}",
                            self.monitor_id, e
                        );
                        // Small delay before retrying
                        tokio::time::sleep(Duration::from_millis(self.config.reconnect_delay_ms))
                            .await;
                    }
                }
            }

            info!("FIFO reader task stopped for monitor {}", self.monitor_id);
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
        if (32..=34).contains(&h265_nal_type) {
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

/// Find an Annex B start code in `buf` starting at position `from`.
/// Returns `(position, start_code_length)` where length is 3 or 4.
fn find_start_code(buf: &[u8], from: usize) -> Option<(usize, usize)> {
    if buf.len() < from + 3 {
        return None;
    }
    let mut i = from;
    while i + 2 < buf.len() {
        if buf[i] == 0x00 && buf[i + 1] == 0x00 {
            if buf[i + 2] == 0x01 {
                // Check for 4-byte start code (00 00 00 01)
                if i > 0 && buf[i - 1] == 0x00 {
                    // Prefer reporting the 4-byte version (backtrack one byte)
                    // but only if caller's `from` allows it
                    if i > from {
                        return Some((i - 1, 4));
                    }
                }
                return Some((i, 3));
            }
            // Check 00 00 00 01 starting at i
            if i + 3 < buf.len() && buf[i + 2] == 0x00 && buf[i + 3] == 0x01 {
                return Some((i, 4));
            }
        }
        i += 1;
    }
    None
}

/// Extract the next complete NAL unit from `buf`.
///
/// Scans for two consecutive Annex B start codes. Returns the bytes from
/// the first start code up to (but not including) the second. Bytes before
/// the first start code are discarded (handles joining mid-stream).
fn extract_next_nal(buf: &mut Vec<u8>) -> Option<Vec<u8>> {
    // Find the first start code
    let (first_pos, first_len) = find_start_code(buf, 0)?;

    // Discard any bytes before the first start code (mid-stream garbage)
    if first_pos > 0 {
        buf.drain(..first_pos);
        // Adjust: first start code is now at position 0
        return extract_next_nal(buf);
    }

    // Find the second start code (marks end of first NAL unit)
    let search_from = first_len;
    if let Some((second_pos, _)) = find_start_code(buf, search_from) {
        let nal = buf[..second_pos].to_vec();
        buf.drain(..second_pos);
        Some(nal)
    } else {
        // Only one start code found — need more data
        None
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
            let mut reader = ZmFifoReader::with_capacity(
                monitor_id,
                self.config.clone(),
                self.broadcast_capacity,
            );
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
        assert!(!ZmFifoReader::is_keyframe(&h264_non_idr, VideoCodec::H264));
    }

    #[test]
    fn test_is_h265_keyframe() {
        // H.265 IDR frame (type 19) - keyframe
        let h265_idr = vec![0x00, 0x00, 0x00, 0x01, 0x26, 0x01, 0xAF, 0x08];
        assert!(ZmFifoReader::is_keyframe(&h265_idr, VideoCodec::H265));

        // H.265 non-IRAP frame (type 1) - not keyframe
        let h265_non_irap = vec![0x00, 0x00, 0x00, 0x01, 0x02, 0x01, 0xD0, 0x00];
        assert!(!ZmFifoReader::is_keyframe(&h265_non_irap, VideoCodec::H265));
    }

    #[test]
    fn test_config_default() {
        let config = ZoneMinderConfig::default();
        assert_eq!(config.fifo_base_path, "/run/zm");
        assert_eq!(config.video_fifo_suffix, "/video_fifo_");
        assert_eq!(config.audio_fifo_suffix, "/audio_fifo_");
        assert_eq!(config.fifo_read_timeout_ms, 5000);
        assert_eq!(config.reconnect_delay_ms, 1000);
    }

    #[test]
    fn test_fifo_reader_creation() {
        // Use a non-existent base path to avoid environment-dependent codec detection
        let config = ZoneMinderConfig {
            fifo_base_path: "/tmp/zm_test_nonexistent".to_string(),
            video_fifo_suffix: "/video_fifo_".to_string(),
            ..ZoneMinderConfig::default()
        };
        let reader = ZmFifoReader::new(1, config);

        assert_eq!(reader.monitor_id(), 1);
        assert_eq!(reader.codec(), VideoCodec::Unknown);
        assert_eq!(
            reader.video_path(),
            Path::new("/tmp/zm_test_nonexistent/video_fifo_1.h264")
        );
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

    #[test]
    fn test_find_start_code_4byte() {
        let buf = [0x00, 0x00, 0x00, 0x01, 0x67, 0x42];
        let result = find_start_code(&buf, 0);
        assert_eq!(result, Some((0, 4)));
    }

    #[test]
    fn test_find_start_code_3byte() {
        let buf = [0xFF, 0x00, 0x00, 0x01, 0x67, 0x42];
        let result = find_start_code(&buf, 0);
        assert_eq!(result, Some((1, 3)));
    }

    #[test]
    fn test_find_start_code_offset() {
        // Start code at position 5
        let buf = [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0x00, 0x00, 0x00, 0x01, 0x67];
        let result = find_start_code(&buf, 3);
        assert_eq!(result, Some((5, 4)));
    }

    #[test]
    fn test_find_start_code_none() {
        let buf = [0xAA, 0xBB, 0xCC, 0xDD];
        assert_eq!(find_start_code(&buf, 0), None);
    }

    #[test]
    fn test_extract_next_nal_single() {
        // Two NAL units: SPS then PPS
        let mut buf = vec![
            0x00, 0x00, 0x00, 0x01, 0x67, 0x42, 0x00, 0x1F, // NAL 1 (SPS)
            0x00, 0x00, 0x00, 0x01, 0x68, 0xCE, 0x3C, 0x80, // NAL 2 (PPS)
        ];
        let nal = extract_next_nal(&mut buf).unwrap();
        // First NAL: start code + SPS data
        assert_eq!(nal, vec![0x00, 0x00, 0x00, 0x01, 0x67, 0x42, 0x00, 0x1F]);
        // Buffer should now start with second NAL
        assert_eq!(buf[0..4], [0x00, 0x00, 0x00, 0x01]);
        assert_eq!(buf[4], 0x68);
    }

    #[test]
    fn test_extract_next_nal_discards_leading_garbage() {
        // Garbage bytes before first start code (joining mid-stream)
        let mut buf = vec![
            0xAA, 0xBB, 0xCC, // garbage
            0x00, 0x00, 0x00, 0x01, 0x67, 0x42, // NAL 1
            0x00, 0x00, 0x00, 0x01, 0x68, 0xCE, // NAL 2
        ];
        let nal = extract_next_nal(&mut buf).unwrap();
        assert_eq!(nal, vec![0x00, 0x00, 0x00, 0x01, 0x67, 0x42]);
    }

    #[test]
    fn test_extract_next_nal_incomplete() {
        // Only one start code — need more data
        let mut buf = vec![0x00, 0x00, 0x00, 0x01, 0x67, 0x42, 0x00, 0x1F];
        assert!(extract_next_nal(&mut buf).is_none());
        // Buffer should be preserved
        assert_eq!(buf.len(), 8);
    }

    #[test]
    fn test_extract_next_nal_3byte_start_codes() {
        let mut buf = vec![
            0x00, 0x00, 0x01, 0x67, 0x42, // NAL 1 (3-byte start code)
            0x00, 0x00, 0x01, 0x68, 0xCE, // NAL 2
        ];
        let nal = extract_next_nal(&mut buf).unwrap();
        assert_eq!(nal, vec![0x00, 0x00, 0x01, 0x67, 0x42]);
    }
}
