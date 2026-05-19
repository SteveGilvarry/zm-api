use std::collections::{HashMap, VecDeque};
use std::io::Read as _;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::time::Duration;
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

/// Largest single packet payload accepted from a `ZM` framing header.
/// A header claiming more than this is treated as corrupt / a false match.
///
/// ZoneMinder frames a whole access unit — every slice of one coded picture —
/// into a single `ZM` packet, so this ceiling must comfortably exceed the
/// biggest keyframe a supported camera emits. A 4K (3840×2160) IDR at a high
/// bitrate is several megabytes; the previous 4 MiB ceiling silently rejected
/// those keyframes as "corrupt", resyncing into the middle of their payload
/// and destroying the stream for every consumer. 16 MiB covers 4K with
/// headroom while still rejecting the absurd sizes a corrupt/false header
/// produces.
const MAX_NAL_BUF_SIZE: usize = 16 * 1024 * 1024;
/// Read buffer size for FIFO reads
const FIFO_READ_BUF_SIZE: usize = 32768;

/// Reader for ZoneMinder's video FIFO
///
/// ZoneMinder's `Fifo` does not write a raw Annex B stream — it frames every
/// captured packet with the ASCII line `ZM <byte_count> <pts>\n` followed by
/// exactly `<byte_count>` bytes of Annex B elementary stream (`zm_fifo.cpp`).
///
/// This reader parses that framing: `<pts>` becomes the packet timestamp and
/// NAL extraction is bounded to a single packet's payload, so the `ZM …\n`
/// header bytes can never leak into a NAL unit. Every NAL split from one
/// packet shares that packet's timestamp, which is exactly the access-unit
/// grouping the segmenter and RTP packetizer require for multi-slice frames.
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
    /// Accumulation buffer for bytes read from the FIFO, still carrying
    /// ZoneMinder `ZM <size> <pts>\n` framing.
    nal_buf: Vec<u8>,
    /// NAL units split from the current ZM packet, awaiting emission. Every
    /// NAL in this queue shares `current_pts_us`.
    ready_nals: VecDeque<Vec<u8>>,
    /// Normalized timestamp (microseconds) of the packet that produced the
    /// NALs currently queued in `ready_nals`.
    current_pts_us: i64,
    /// The first ZM `pts` ever observed, subtracted from every later pts so
    /// the session starts near zero (cameras running for hours emit huge
    /// absolute PTS values). `None` until the first packet is parsed.
    base_pts_us: Option<i64>,
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
            ready_nals: VecDeque::new(),
            current_pts_us: 0,
            base_pts_us: None,
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
    /// Parses ZoneMinder's `ZM <size> <pts>\n` framing out of the FIFO byte
    /// stream, then yields one NAL unit per call. Every NAL split from the
    /// same framed packet is returned with that packet's `pts`, so multi-slice
    /// access units stay grouped downstream.
    ///
    /// Reads use `AsyncFd` for epoll-based readiness notification, which
    /// avoids blocking tokio's thread pool and lets timeouts cancel pending
    /// reads (the future is simply dropped).
    async fn read_packet_internal(&mut self) -> Result<FifoPacket, FifoError> {
        let async_fd = self.video_reader.as_ref().ok_or(FifoError::NotCapturing)?;

        loop {
            // 1. Emit a NAL already split from the current ZM packet. Every
            //    NAL of one packet carries that packet's timestamp.
            if let Some(nal_data) = self.ready_nals.pop_front() {
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

                debug!(
                    "Read NAL for monitor {}: {} bytes, keyframe: {}, codec: {}",
                    self.monitor_id,
                    nal_data.len(),
                    is_keyframe,
                    self.codec.as_str()
                );

                return Ok(FifoPacket {
                    monitor_id: self.monitor_id,
                    timestamp_us: self.current_pts_us,
                    data: nal_data,
                    is_keyframe,
                    codec: self.codec,
                });
            }

            // 2. Parse the next `ZM <size> <pts>\n` framed packet from the
            //    accumulation buffer and split its payload into NAL units.
            match parse_zm_frame(&self.nal_buf) {
                ZmFrame::Complete {
                    header_len,
                    payload_size,
                    pts,
                } => {
                    let total = header_len + payload_size;
                    if self.nal_buf.len() >= total {
                        // A real packet is immediately followed by the next
                        // `ZM` header. If the byte after the payload is not
                        // 'Z', this "header" was a false match inside payload
                        // data — skip it and resync.
                        if self.nal_buf.len() > total && self.nal_buf[total] != b'Z' {
                            resync_zm(&mut self.nal_buf);
                            continue;
                        }

                        let body = self.nal_buf[header_len..total].to_vec();
                        self.nal_buf.drain(..total);

                        // Normalize so the session starts near zero.
                        let base = *self.base_pts_us.get_or_insert(pts);
                        self.current_pts_us = pts.saturating_sub(base);
                        self.ready_nals = split_annexb_nals(body).into();
                        continue;
                    }
                    // Header parsed, payload still arriving — read more below.
                }
                ZmFrame::Incomplete => {
                    // A valid but truncated header — read more below.
                }
                ZmFrame::Invalid => {
                    // Not at a packet boundary (FIFO opened mid-stream, or
                    // corruption). Skip to the next `ZM ` marker and retry; if
                    // none is present yet, fall through to read more.
                    if resync_zm(&mut self.nal_buf) {
                        continue;
                    }
                }
            }

            // 3. Need more data — wait for FIFO to become readable via epoll,
            //    then perform a non-blocking read. This is cancel-safe: if the
            //    timeout fires, the future is dropped without leaving a blocked
            //    thread in the pool.
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

            // Prevent unbounded buffer growth (corrupt stream with no usable
            // `ZM` markers). The limit is twice the largest accepted packet so
            // a legitimate near-maximum packet still has room to finish.
            if self.nal_buf.len() > MAX_NAL_BUF_SIZE * 2 {
                warn!(
                    "FIFO buffer overflow ({} bytes) for monitor {}, resetting",
                    self.nal_buf.len(),
                    self.monitor_id
                );
                self.nal_buf.clear();
                self.ready_nals.clear();
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

/// Return the H.264 NAL unit type (bits 0-4 of first byte after start code).
///
/// Accepts both 3-byte (`00 00 01`) and 4-byte (`00 00 00 01`) start codes.
/// Returns `None` if no start code is found.
pub fn h264_nal_type(nal_data: &[u8]) -> Option<u8> {
    let offset = if nal_data.starts_with(&[0, 0, 0, 1]) {
        4
    } else if nal_data.starts_with(&[0, 0, 1]) {
        3
    } else {
        return None;
    };
    nal_data.get(offset).map(|b| b & 0x1F)
}

/// Extract the NAL unit type from an H.265/HEVC NAL unit.
///
/// HEVC uses a two-byte NAL header; the 6-bit `nal_unit_type` occupies bits
/// 1–6 of the first byte (`(byte >> 1) & 0x3F`). Returns `None` when the NAL
/// has no recognizable Annex B start code or is too short. Parameter-set
/// types: VPS = 32, SPS = 33, PPS = 34.
pub fn h265_nal_type(nal_data: &[u8]) -> Option<u8> {
    let offset = if nal_data.starts_with(&[0, 0, 0, 1]) {
        4
    } else if nal_data.starts_with(&[0, 0, 1]) {
        3
    } else {
        return None;
    };
    nal_data.get(offset).map(|b| (b >> 1) & 0x3F)
}

/// Returns `true` if a VCL NAL unit begins a new primary coded picture.
///
/// Access-unit assembly groups every slice of one coded picture together, and
/// the picture's *first* slice is the only reliable boundary marker. This
/// inspects the first bit of the slice (segment) header, which both codecs
/// define as a picture-start marker:
///
/// * **H.264** — the slice header opens with `first_mb_in_slice`, an unsigned
///   Exp-Golomb (`ue(v)`) value. The first slice of a picture has
///   `first_mb_in_slice == 0`, which encodes as the single bit `1` — the MSB
///   of the first slice-header byte is set. Every continuation slice has
///   `first_mb_in_slice > 0`; an Exp-Golomb value greater than zero always
///   begins with a `0` bit, so its MSB is clear. This holds for the very
///   large macroblock indices a 4K picture's later slices carry (e.g. slice
///   24 of a 3840×2160 frame starts well past macroblock 8000). Emulation-
///   prevention bytes (`0x03`) are only ever inserted as the third byte of a
///   `00 00` run, so they can never displace the first slice-header byte.
///
/// * **H.265/HEVC** — the two-byte NAL header is followed by the slice segment
///   header, which opens with the one-bit `first_slice_segment_in_pic_flag`.
///   A set MSB on that byte marks the first slice of the picture.
///
/// Callers are expected to pass VCL NALs only. When the NAL has no
/// recognizable start code, or is too short to inspect, the answer is the
/// conservative `true`: starting a fresh access unit is safer than merging
/// two distinct pictures into one.
pub fn slice_starts_picture(nal_data: &[u8], codec: VideoCodec) -> bool {
    let start_code_len = if nal_data.starts_with(&[0, 0, 0, 1]) {
        4
    } else if nal_data.starts_with(&[0, 0, 1]) {
        3
    } else {
        return true;
    };
    // The slice (segment) header begins immediately after the NAL header:
    // one byte for H.264, two bytes for H.265.
    let nal_header_len = match codec {
        VideoCodec::H265 => 2,
        VideoCodec::H264 | VideoCodec::Unknown => 1,
    };
    match nal_data.get(start_code_len + nal_header_len) {
        Some(&first_slice_header_byte) => first_slice_header_byte & 0x80 != 0,
        None => true,
    }
}

/// Extract `profile-level-id` from an H.264 SPS NAL unit.
///
/// The three bytes immediately after the NAL header in an SPS are
/// `profile_idc`, `constraint_set_flags`, and `level_idc` — exactly the
/// value needed for the SDP `profile-level-id` parameter.
///
/// Returns a 6-character hex string (e.g. `"4d0033"` for Main Profile Level 5.1).
pub fn extract_profile_level_id(nal_data: &[u8]) -> Option<String> {
    let offset = if nal_data.starts_with(&[0, 0, 0, 1]) {
        4
    } else if nal_data.starts_with(&[0, 0, 1]) {
        3
    } else {
        return None;
    };

    // Need NAL header + profile_idc + constraint_flags + level_idc
    if nal_data.len() < offset + 4 {
        return None;
    }

    let nal_type = nal_data[offset] & 0x1F;
    if nal_type != 7 {
        return None; // Not an SPS
    }

    let profile_idc = nal_data[offset + 1];
    let constraint_flags = nal_data[offset + 2];
    let level_idc = nal_data[offset + 3];

    Some(format!(
        "{:02x}{:02x}{:02x}",
        profile_idc, constraint_flags, level_idc
    ))
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

/// Split one ZM packet's Annex B payload into its constituent NAL units.
///
/// Unlike [`extract_next_nal`], the payload is a complete, bounded packet, so
/// the final NAL — which has no trailing start code — is also returned. Each
/// returned NAL keeps its leading start code.
fn split_annexb_nals(mut body: Vec<u8>) -> Vec<Vec<u8>> {
    let mut nals = Vec::new();
    while let Some(nal) = extract_next_nal(&mut body) {
        nals.push(nal);
    }
    // `extract_next_nal` returns `None` only once a single start code remains,
    // having already drained any leading garbage — so whatever is left begins
    // at a start code and is the packet's last NAL.
    if find_start_code(&body, 0).is_some() {
        nals.push(body);
    }
    nals
}

/// Outcome of parsing a ZoneMinder `ZM <size> <pts>\n` framing header at the
/// start of a buffer.
#[derive(Debug, PartialEq, Eq)]
enum ZmFrame {
    /// A complete header was parsed. `header_len` is its byte length;
    /// `payload_size` bytes of Annex B data follow it.
    Complete {
        header_len: usize,
        payload_size: usize,
        pts: i64,
    },
    /// The buffer holds a valid header prefix but is truncated — read more.
    Incomplete,
    /// The buffer does not begin with a usable header — caller must resync.
    Invalid,
}

/// Parse a ZoneMinder FIFO framing header at the start of `buf`.
///
/// ZoneMinder's `Fifo` writes each packet as the ASCII line
/// `ZM <byte_count> <pts>\n` immediately followed by `<byte_count>` raw bytes
/// of Annex B elementary stream (`zm_fifo.cpp`).
fn parse_zm_frame(buf: &[u8]) -> ZmFrame {
    const PREFIX: &[u8] = b"ZM ";

    if buf.len() < PREFIX.len() {
        // Too short to classify: still potentially a header if the bytes so
        // far match the prefix, otherwise definitely not one.
        return if PREFIX.starts_with(buf) {
            ZmFrame::Incomplete
        } else {
            ZmFrame::Invalid
        };
    }
    if &buf[..PREFIX.len()] != PREFIX {
        return ZmFrame::Invalid;
    }
    let mut i = PREFIX.len();

    // payload size: one or more ASCII digits, then a single space
    let size_start = i;
    while i < buf.len() && buf[i].is_ascii_digit() {
        i += 1;
    }
    if i == buf.len() {
        return ZmFrame::Incomplete; // digits may continue past the buffer end
    }
    if i == size_start || buf[i] != b' ' {
        return ZmFrame::Invalid;
    }
    let payload_size: usize = match std::str::from_utf8(&buf[size_start..i])
        .ok()
        .and_then(|s| s.parse().ok())
    {
        Some(n) => n,
        None => return ZmFrame::Invalid,
    };
    i += 1; // consume the space

    // pts: an optional '-' followed by one or more ASCII digits, then '\n'
    let pts_start = i;
    if i < buf.len() && buf[i] == b'-' {
        i += 1;
    }
    let pts_digits_start = i;
    while i < buf.len() && buf[i].is_ascii_digit() {
        i += 1;
    }
    if i == buf.len() {
        return ZmFrame::Incomplete;
    }
    if i == pts_digits_start || buf[i] != b'\n' {
        return ZmFrame::Invalid;
    }
    let pts: i64 = match std::str::from_utf8(&buf[pts_start..i])
        .ok()
        .and_then(|s| s.parse().ok())
    {
        Some(n) => n,
        None => return ZmFrame::Invalid,
    };
    i += 1; // consume the newline

    // Reject absurd sizes: a corrupt header, or a `ZM ` sequence that happened
    // to appear inside payload data.
    if payload_size == 0 || payload_size > MAX_NAL_BUF_SIZE {
        return ZmFrame::Invalid;
    }

    ZmFrame::Complete {
        header_len: i,
        payload_size,
        pts,
    }
}

/// Drop bytes until the start of the next `ZM ` framing header.
///
/// The search starts past index 0, so a header that itself failed to parse is
/// skipped to the *next* candidate. Returns `true` if a candidate was found
/// and the buffer advanced to it (the caller should retry parsing). Returns
/// `false` if none was found; the buffer is trimmed to its last two bytes so a
/// `ZM` prefix split across two FIFO reads is preserved.
fn resync_zm(buf: &mut Vec<u8>) -> bool {
    if let Some(p) = buf.windows(3).skip(1).position(|w| w == b"ZM ") {
        buf.drain(..p + 1); // `+1` compensates for the skipped first window
        true
    } else {
        if buf.len() > 2 {
            let keep_from = buf.len() - 2;
            buf.drain(..keep_from);
        }
        false
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

        // The reader is either pre-existing or was inserted just above, so the
        // lookup cannot miss.
        Ok(self
            .readers
            .get_mut(&monitor_id)
            .expect("reader present: inserted above when absent"))
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
    fn test_h265_nal_type() {
        // The HEVC NAL type is bits 1–6 of the first header byte. VPS = 32
        // (0x40), SPS = 33 (0x42), PPS = 34 (0x44).
        assert_eq!(
            h265_nal_type(&[0x00, 0x00, 0x00, 0x01, 0x40, 0x01]),
            Some(32)
        );
        assert_eq!(
            h265_nal_type(&[0x00, 0x00, 0x00, 0x01, 0x42, 0x01]),
            Some(33)
        );
        assert_eq!(
            h265_nal_type(&[0x00, 0x00, 0x00, 0x01, 0x44, 0x01]),
            Some(34)
        );
        // IDR_W_RADL = 19 (0x26), with a 3-byte start code.
        assert_eq!(h265_nal_type(&[0x00, 0x00, 0x01, 0x26, 0x01]), Some(19));
        // No start code, and too-short input, both yield None.
        assert_eq!(h265_nal_type(&[0x40, 0x01]), None);
        assert_eq!(h265_nal_type(&[0x00, 0x00, 0x00, 0x01]), None);
    }

    #[test]
    fn test_parse_zm_frame_complete() {
        let header = b"ZM 12345 96096000\n";
        let mut buf = header.to_vec();
        buf.extend_from_slice(&[0x00, 0x00, 0x00, 0x01, 0x67]); // start of payload
        match parse_zm_frame(&buf) {
            ZmFrame::Complete {
                header_len,
                payload_size,
                pts,
            } => {
                assert_eq!(header_len, header.len());
                assert_eq!(payload_size, 12345);
                assert_eq!(pts, 96_096_000);
            }
            other => panic!("expected Complete, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_zm_frame_negative_pts() {
        match parse_zm_frame(b"ZM 50 -42\npayload") {
            ZmFrame::Complete {
                payload_size, pts, ..
            } => {
                assert_eq!(payload_size, 50);
                assert_eq!(pts, -42);
            }
            other => panic!("expected Complete, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_zm_frame_incomplete() {
        // Empty, partial prefix, and full prefix are all still potential headers.
        assert_eq!(parse_zm_frame(b""), ZmFrame::Incomplete);
        assert_eq!(parse_zm_frame(b"Z"), ZmFrame::Incomplete);
        assert_eq!(parse_zm_frame(b"ZM "), ZmFrame::Incomplete);
        // Truncated mid size digits.
        assert_eq!(parse_zm_frame(b"ZM 123"), ZmFrame::Incomplete);
        // Truncated mid pts digits (no newline yet).
        assert_eq!(parse_zm_frame(b"ZM 100 960"), ZmFrame::Incomplete);
    }

    #[test]
    fn test_parse_zm_frame_invalid() {
        // No `ZM ` prefix at all.
        assert_eq!(parse_zm_frame(b"XY 100 200\n"), ZmFrame::Invalid);
        // Raw Annex B bytes (FIFO opened mid-stream).
        assert_eq!(
            parse_zm_frame(&[0x00, 0x00, 0x00, 0x01, 0x67]),
            ZmFrame::Invalid
        );
        // Prefix present but the size field is not digits.
        assert_eq!(parse_zm_frame(b"ZM x100 200\n"), ZmFrame::Invalid);
        // Wrong separator between size and pts.
        assert_eq!(parse_zm_frame(b"ZM 100x200\n"), ZmFrame::Invalid);
        // A zero-length payload is rejected.
        assert_eq!(parse_zm_frame(b"ZM 0 200\n"), ZmFrame::Invalid);
        // A payload larger than the accepted maximum is rejected.
        let huge = format!("ZM {} 200\n", MAX_NAL_BUF_SIZE + 1);
        assert_eq!(parse_zm_frame(huge.as_bytes()), ZmFrame::Invalid);
    }

    #[test]
    fn test_resync_zm_finds_next_header() {
        let mut buf = b"trailing payload bytesZM 100 200\nrest".to_vec();
        assert!(resync_zm(&mut buf));
        assert!(buf.starts_with(b"ZM 100 200\n"));
    }

    #[test]
    fn test_resync_zm_skips_header_at_index_zero() {
        // A header at index 0 that failed to parse must be skipped to the next.
        let mut buf = b"ZM bogusZM 7 9\nx".to_vec();
        assert!(resync_zm(&mut buf));
        assert!(buf.starts_with(b"ZM 7 9\n"));
    }

    #[test]
    fn test_resync_zm_no_header_trims_tail() {
        let mut buf = b"no marker present at all".to_vec();
        assert!(!resync_zm(&mut buf));
        // Tail is preserved in case a `ZM` prefix is split across reads.
        assert!(buf.len() <= 2);
    }

    #[test]
    fn test_split_annexb_nals_multi() {
        // Three NALs back to back; the last one has no trailing start code.
        let body = vec![
            0x00, 0x00, 0x00, 0x01, 0x67, 0xAA, // SPS
            0x00, 0x00, 0x00, 0x01, 0x68, 0xBB, // PPS
            0x00, 0x00, 0x00, 0x01, 0x65, 0xCC, 0xDD, // IDR slice
        ];
        let nals = split_annexb_nals(body);
        assert_eq!(nals.len(), 3);
        assert_eq!(nals[0], vec![0x00, 0x00, 0x00, 0x01, 0x67, 0xAA]);
        assert_eq!(nals[1], vec![0x00, 0x00, 0x00, 0x01, 0x68, 0xBB]);
        assert_eq!(nals[2], vec![0x00, 0x00, 0x00, 0x01, 0x65, 0xCC, 0xDD]);
    }

    #[test]
    fn test_split_annexb_nals_single_and_empty() {
        let single = split_annexb_nals(vec![0x00, 0x00, 0x00, 0x01, 0x65, 0x11]);
        assert_eq!(single, vec![vec![0x00, 0x00, 0x00, 0x01, 0x65, 0x11]]);

        assert!(split_annexb_nals(vec![0x11, 0x22, 0x33]).is_empty());
        assert!(split_annexb_nals(Vec::new()).is_empty());
    }

    #[test]
    fn test_zm_framing_recovers_multi_slice_picture_cleanly() {
        // One ZM packet carrying a 3-slice access unit, framed exactly as
        // ZoneMinder writes it, with a real `ZM` header for the next packet
        // appended so the framing boundary is exercised.
        let body = [
            0x00, 0x00, 0x00, 0x01, 0x65, 0x88, // slice 0
            0x00, 0x00, 0x00, 0x01, 0x65, 0x10, // slice 1
            0x00, 0x00, 0x00, 0x01, 0x65, 0x10, // slice 2
        ];
        let mut buf = format!("ZM {} 500000\n", body.len()).into_bytes();
        buf.extend_from_slice(&body);
        buf.extend_from_slice(b"ZM 4 600000\n"); // next packet's header

        let ZmFrame::Complete {
            header_len,
            payload_size,
            pts,
        } = parse_zm_frame(&buf)
        else {
            panic!("expected Complete");
        };
        assert_eq!(pts, 500_000);
        assert_eq!(buf[header_len + payload_size], b'Z'); // next header follows

        let nals = split_annexb_nals(buf[header_len..header_len + payload_size].to_vec());
        // All three slices are recovered, and none carries the `ZM` header
        // bytes that previously leaked into NAL payloads.
        assert_eq!(nals.len(), 3);
        for nal in &nals {
            assert!(
                !nal.windows(2).any(|w| w == b"ZM"),
                "NAL must not contain framing-header bytes"
            );
        }
    }

    /// Encode `first_mb_in_slice` as an H.264 `ue(v)` and return the byte that
    /// immediately follows the NAL header — the byte `slice_starts_picture`
    /// inspects.
    fn first_slice_header_byte(first_mb_in_slice: u32) -> u8 {
        let code = first_mb_in_slice as u64 + 1;
        let significant = 64 - code.leading_zeros();
        let leading_zeros = significant - 1;
        let mut byte = 0u8;
        let mut bit_pos = 0u32; // 0 = MSB
        for _ in 0..leading_zeros {
            bit_pos += 1; // a zero bit; nothing to set
            if bit_pos >= 8 {
                return byte;
            }
        }
        for i in (0..significant).rev() {
            if (code >> i) & 1 == 1 && bit_pos < 8 {
                byte |= 1 << (7 - bit_pos);
            }
            bit_pos += 1;
            if bit_pos >= 8 {
                break;
            }
        }
        byte
    }

    #[test]
    fn test_slice_starts_picture_h264_first_slice() {
        // first_mb_in_slice == 0 → ue(v) is the single bit `1` → MSB set.
        let first = first_slice_header_byte(0);
        assert_eq!(first, 0x80);
        let nal = vec![0x00, 0x00, 0x00, 0x01, 0x65, first, 0x40];
        assert!(slice_starts_picture(&nal, VideoCodec::H264));
        // Also the 3-byte start-code form.
        let nal3 = vec![0x00, 0x00, 0x01, 0x41, first, 0x40];
        assert!(slice_starts_picture(&nal3, VideoCodec::H264));
    }

    #[test]
    fn test_slice_starts_picture_h264_continuation_slices() {
        // Every continuation slice of a multi-slice picture has
        // first_mb_in_slice > 0, so its first slice-header byte has the MSB
        // clear — including the large macroblock indices a 4K picture emits.
        for first_mb in [1u32, 5, 99, 240, 8160, 16200, 32000] {
            let byte = first_slice_header_byte(first_mb);
            assert_eq!(
                byte & 0x80,
                0,
                "first_mb_in_slice={first_mb} must encode with the MSB clear"
            );
            let nal = vec![0x00, 0x00, 0x00, 0x01, 0x65, byte, 0xAA, 0xBB];
            assert!(
                !slice_starts_picture(&nal, VideoCodec::H264),
                "first_mb_in_slice={first_mb} must read as a continuation slice"
            );
        }
    }

    #[test]
    fn test_slice_starts_picture_h264_emulation_prevention_unaffected() {
        // A continuation slice whose header begins `00 00 03 …` (an inserted
        // emulation-prevention byte). The detector inspects only the first
        // slice-header byte (0x00), so the 0x03 cannot shift its decision.
        let nal = vec![0x00, 0x00, 0x00, 0x01, 0x65, 0x00, 0x00, 0x03, 0x12];
        assert!(!slice_starts_picture(&nal, VideoCodec::H264));
    }

    #[test]
    fn test_slice_starts_picture_h265() {
        // H.265 has a 2-byte NAL header; the slice segment header opens with
        // `first_slice_segment_in_pic_flag`.
        // NAL header bytes 0x26,0x01 = IDR_W_RADL.
        let starts = vec![0x00, 0x00, 0x00, 0x01, 0x26, 0x01, 0x80, 0x40];
        assert!(slice_starts_picture(&starts, VideoCodec::H265));
        let continuation = vec![0x00, 0x00, 0x00, 0x01, 0x26, 0x01, 0x40, 0x40];
        assert!(!slice_starts_picture(&continuation, VideoCodec::H265));
    }

    #[test]
    fn test_slice_starts_picture_conservative_defaults() {
        // No start code → cannot locate the slice header → treat as a new AU.
        assert!(slice_starts_picture(&[0x65, 0x80], VideoCodec::H264));
        // Start code present but nothing after the NAL header → new AU.
        assert!(slice_starts_picture(
            &[0x00, 0x00, 0x00, 0x01, 0x65],
            VideoCodec::H264
        ));
    }

    #[test]
    fn test_parse_zm_frame_accepts_4k_keyframe() {
        // A 5 MiB 4K IDR access unit: rejected by the old 4 MiB ceiling,
        // accepted now. `parse_zm_frame` only needs the header present.
        let payload = 5 * 1024 * 1024;
        assert!(payload <= MAX_NAL_BUF_SIZE);
        let header = format!("ZM {payload} 96096000\n");
        match parse_zm_frame(header.as_bytes()) {
            ZmFrame::Complete { payload_size, .. } => assert_eq!(payload_size, payload),
            other => panic!("expected Complete for a 4K-sized packet, got {other:?}"),
        }
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

    // --- Tests for public h264_nal_type ---

    #[test]
    fn test_h264_nal_type_4byte_start_code() {
        assert_eq!(h264_nal_type(&[0, 0, 0, 1, 0x67]), Some(7)); // SPS
        assert_eq!(h264_nal_type(&[0, 0, 0, 1, 0x68]), Some(8)); // PPS
        assert_eq!(h264_nal_type(&[0, 0, 0, 1, 0x65]), Some(5)); // IDR
        assert_eq!(h264_nal_type(&[0, 0, 0, 1, 0x41]), Some(1)); // non-IDR
    }

    #[test]
    fn test_h264_nal_type_3byte_start_code() {
        assert_eq!(h264_nal_type(&[0, 0, 1, 0x67]), Some(7));
    }

    #[test]
    fn test_h264_nal_type_no_start_code() {
        assert_eq!(h264_nal_type(&[0xFF, 0x67]), None);
        assert_eq!(h264_nal_type(&[]), None);
    }

    // --- Tests for public extract_profile_level_id ---

    #[test]
    fn test_extract_profile_level_id_valid_sps() {
        // Main Profile, Level 5.1
        let sps = vec![0x00, 0x00, 0x00, 0x01, 0x67, 0x4D, 0x00, 0x33, 0xFF];
        assert_eq!(extract_profile_level_id(&sps), Some("4d0033".to_string()));
    }

    #[test]
    fn test_extract_profile_level_id_not_sps() {
        // PPS (type 8) — should return None
        let pps = vec![0x00, 0x00, 0x00, 0x01, 0x68, 0xCE, 0x3C, 0x80];
        assert_eq!(extract_profile_level_id(&pps), None);
    }

    #[test]
    fn test_extract_profile_level_id_too_short() {
        let short = vec![0x00, 0x00, 0x00, 0x01, 0x67, 0x4D];
        assert_eq!(extract_profile_level_id(&short), None);
    }
}
