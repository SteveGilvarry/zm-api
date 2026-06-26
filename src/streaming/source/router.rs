//! Source Router for unified streaming source management
//!
//! Provides a unified abstraction over per-monitor stream-socket readers that
//! serves all output protocols (WebRTC, HLS). Manages lazy initialization of
//! monitor sources; one socket connection carries both video and audio.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

use dashmap::DashMap;
use tokio::io::AsyncWriteExt;
use tokio::sync::{broadcast, mpsc, watch, RwLock};
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};

use super::media::{
    extract_profile_level_id, h264_nal_type, h265_nal_type, AudioCodec, AudioPacket, VideoCodec,
    VideoPacket,
};
use super::protocol::{self, MonitorEvent};
use super::stream_socket::{stream_socket_path, SocketEvent, SourceError, StreamSocketReader};
use crate::configure::streaming::ZoneMinderConfig;

/// A monitor EVENT decoded off a stream socket, tagged with its monitor, ready
/// for DB ingest. The reader task forwards these to the router's event sink
/// (when one is registered); media flows unaffected.
///
/// `reply` lets the ingest side answer the worker on the *same* connection —
/// the id-assignment handshake replies to a `recording_opening` EVENT with the
/// allocated event id and target path.
#[derive(Debug, Clone)]
pub struct MonitorEventEnvelope {
    pub monitor_id: u32,
    pub event: MonitorEvent,
    pub reply: ControlReply,
}

/// Cloneable handle for sending client→server control messages on one monitor's
/// stream-socket connection. Messages are queued to the reader task, which owns
/// the connection's write half and flushes them. Best-effort by design: if the
/// connection has closed (queue full or receiver gone) the send is dropped, so
/// ingest never blocks or stalls on a dead worker.
#[derive(Debug, Clone)]
pub struct ControlReply {
    tx: mpsc::Sender<Vec<u8>>,
    seq: Arc<AtomicU32>,
}

impl ControlReply {
    fn new(tx: mpsc::Sender<Vec<u8>>) -> Self {
        Self {
            tx,
            seq: Arc::new(AtomicU32::new(0)),
        }
    }

    /// A reply handle with no live connection behind it: every send is dropped
    /// (returns `false`). Models a genuinely one-way EVENT (e.g. `review_assets`,
    /// which never replies) and lets ingest be driven in tests without a socket.
    pub fn detached() -> Self {
        // The receiver is dropped immediately, so `try_send` always errors and
        // `send_command_json` reports `false` — exactly the "connection gone"
        // best-effort semantics.
        let (tx, _rx) = mpsc::channel(1);
        Self::new(tx)
    }

    /// Queue a `0x11 Command` with a JSON payload for the worker. Returns
    /// whether it was queued (false ⇒ connection gone / queue full).
    pub fn send_command_json(&self, json: &str) -> bool {
        let seq = self.seq.fetch_add(1, Ordering::Relaxed);
        let msg = protocol::build_control_message(protocol::MSG_TYPE_COMMAND, seq, json.as_bytes());
        self.tx.try_send(msg).is_ok()
    }
}

/// Default broadcast channel capacity for source packets
const DEFAULT_SOURCE_CAPACITY: usize = 100;

/// Health state of the stream-socket reader task.
///
/// Subscribers (e.g. the coordinator's processing task) can watch this to
/// detect reader failures instead of hanging on a broadcast channel that
/// never closes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReaderHealth {
    /// Reader not started yet
    Idle,
    /// Attempting to connect to the stream socket
    Opening,
    /// Socket connected and producing packets
    Active,
    /// Socket closed or errored, attempting reconnect
    Reconnecting,
    /// Reader task exited (watch sender dropped)
    Stopped,
}

/// Stream topology learned from the socket's connect handshake.
///
/// zmc sends the per-stream HELLOs before any media, so once any media
/// message has arrived the topology is final for that connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StreamInfo {
    pub video_codec: VideoCodec,
    /// `None` when the monitor has no audio stream.
    pub audio_codec: Option<AudioCodec>,
}

/// Cached keyframe data for fast WebRTC startup.
///
/// When a reader is active, this is populated on each keyframe arrival so
/// that new WebRTC connections can skip codec detection and keyframe waits.
/// The injectable unit is `keyframe_au`: SPS+PPS+IDR for H.264,
/// VPS+SPS+PPS+IRAP for H.265 — everything a decoder needs to initialise.
#[derive(Debug, Clone)]
pub struct CachedKeyframe {
    /// Raw SPS NAL (with Annex B start code)
    pub sps: Vec<u8>,
    /// Raw PPS NAL (with Annex B start code)
    pub pps: Vec<u8>,
    /// Parameter sets + keyframe concatenated (Annex B), ready to inject as
    /// one AU. For H.265 this starts with the VPS.
    pub keyframe_au: Vec<u8>,
    /// profile-level-id extracted from the H.264 SPS (e.g. "4d0033").
    /// Empty for H.265 — the H.265 SDP offer does not use it.
    pub profile_level_id: String,
    /// Video codec
    pub codec: VideoCodec,
    /// Timestamp in microseconds (from reader's monotonic clock)
    pub timestamp_us: i64,
}

/// Tracks parameter sets across NALs so a complete [`CachedKeyframe`] can be
/// assembled the moment a keyframe slice arrives. H.264 needs SPS+PPS;
/// H.265 additionally needs the VPS. Re-created on each socket reconnect
/// (fresh parameter sets are expected after a stream restart).
#[derive(Default)]
struct KeyframeCacheBuilder {
    /// H.265 VPS (unused for H.264)
    vps: Option<Vec<u8>>,
    sps: Option<Vec<u8>>,
    pps: Option<Vec<u8>>,
    /// Extracted from the H.264 SPS (None for H.265)
    profile_level_id: Option<String>,
}

impl KeyframeCacheBuilder {
    /// Feed one NAL packet. Returns a complete cache entry when the packet
    /// is a keyframe slice and all required parameter sets have been seen.
    fn push(&mut self, packet: &VideoPacket) -> Option<CachedKeyframe> {
        match packet.codec {
            VideoCodec::H264 => match h264_nal_type(&packet.data)? {
                7 => {
                    self.sps = Some(packet.data.clone());
                    self.profile_level_id = extract_profile_level_id(&packet.data);
                    None
                }
                8 => {
                    self.pps = Some(packet.data.clone());
                    None
                }
                5 => {
                    let (sps, pps, plid) = (
                        self.sps.as_ref()?,
                        self.pps.as_ref()?,
                        self.profile_level_id.as_ref()?,
                    );
                    let mut keyframe_au =
                        Vec::with_capacity(sps.len() + pps.len() + packet.data.len());
                    keyframe_au.extend_from_slice(sps);
                    keyframe_au.extend_from_slice(pps);
                    keyframe_au.extend_from_slice(&packet.data);
                    Some(CachedKeyframe {
                        sps: sps.clone(),
                        pps: pps.clone(),
                        keyframe_au,
                        profile_level_id: plid.clone(),
                        codec: VideoCodec::H264,
                        timestamp_us: packet.timestamp_us,
                    })
                }
                _ => None,
            },
            VideoCodec::H265 => match h265_nal_type(&packet.data)? {
                32 => {
                    self.vps = Some(packet.data.clone());
                    None
                }
                33 => {
                    self.sps = Some(packet.data.clone());
                    None
                }
                34 => {
                    self.pps = Some(packet.data.clone());
                    None
                }
                // IRAP pictures (BLA 16-18, IDR 19-20, CRA 21) are valid
                // decoder entry points.
                t if (16..=21).contains(&t) => {
                    let (vps, sps, pps) =
                        (self.vps.as_ref()?, self.sps.as_ref()?, self.pps.as_ref()?);
                    let mut keyframe_au =
                        Vec::with_capacity(vps.len() + sps.len() + pps.len() + packet.data.len());
                    keyframe_au.extend_from_slice(vps);
                    keyframe_au.extend_from_slice(sps);
                    keyframe_au.extend_from_slice(pps);
                    keyframe_au.extend_from_slice(&packet.data);
                    Some(CachedKeyframe {
                        sps: sps.clone(),
                        pps: pps.clone(),
                        keyframe_au,
                        profile_level_id: String::new(),
                        codec: VideoCodec::H265,
                        timestamp_us: packet.timestamp_us,
                    })
                }
                _ => None,
            },
            VideoCodec::Unknown => None,
        }
    }
}

/// Represents an active monitor source with video and optional audio streams
pub struct MonitorSource {
    monitor_id: u32,
    /// Broadcast sender for video packets
    video_tx: broadcast::Sender<VideoPacket>,
    /// Broadcast sender for audio packets. Always present — whether the
    /// monitor actually has audio is only known after the socket handshake
    /// (see [`StreamInfo`]); a no-audio monitor's channel simply never
    /// carries packets.
    audio_tx: broadcast::Sender<AudioPacket>,
    /// Background reader task handle
    reader_handle: RwLock<Option<JoinHandle<()>>>,
    /// Audio packets read since the source was created
    audio_packets: Arc<std::sync::atomic::AtomicU64>,
    /// Detected video codec
    codec: RwLock<VideoCodec>,
    /// Whether the source is actively reading
    active: RwLock<bool>,
    /// Reader health watch — subscribers get notified on state transitions.
    /// When the reader task exits, the sender is dropped and receivers get Err.
    reader_health_tx: watch::Sender<ReaderHealth>,
    reader_health_rx: watch::Receiver<ReaderHealth>,
    /// Stream topology from the socket handshake; `None` until the reader
    /// has seen the connect HELLOs confirmed by a first media message.
    stream_info_tx: watch::Sender<Option<StreamInfo>>,
    stream_info_rx: watch::Receiver<Option<StreamInfo>>,
    /// Cached keyframe (SPS+PPS+IDR) for fast WebRTC startup.
    /// Updated each time an IDR is seen by the reader task.
    keyframe_cache_tx: watch::Sender<Option<CachedKeyframe>>,
    keyframe_cache_rx: watch::Receiver<Option<CachedKeyframe>>,
}

impl MonitorSource {
    /// Create a new monitor source
    fn new(monitor_id: u32) -> Self {
        let (video_tx, _) = broadcast::channel(DEFAULT_SOURCE_CAPACITY);
        let (audio_tx, _) = broadcast::channel(DEFAULT_SOURCE_CAPACITY);
        let (reader_health_tx, reader_health_rx) = watch::channel(ReaderHealth::Idle);
        let (stream_info_tx, stream_info_rx) = watch::channel(None);
        let (keyframe_cache_tx, keyframe_cache_rx) = watch::channel(None);

        Self {
            monitor_id,
            video_tx,
            audio_tx,
            reader_handle: RwLock::new(None),
            audio_packets: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            codec: RwLock::new(VideoCodec::Unknown),
            active: RwLock::new(false),
            reader_health_tx,
            reader_health_rx,
            stream_info_tx,
            stream_info_rx,
            keyframe_cache_tx,
            keyframe_cache_rx,
        }
    }

    /// Number of audio packets read since the source was created
    pub fn audio_packet_count(&self) -> u64 {
        self.audio_packets
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Subscribe to receive video packets
    pub fn subscribe_video(&self) -> broadcast::Receiver<VideoPacket> {
        self.video_tx.subscribe()
    }

    /// Subscribe to receive audio packets. For a monitor without audio the
    /// receiver simply never yields (check [`Self::has_audio`]).
    pub fn subscribe_audio(&self) -> broadcast::Receiver<AudioPacket> {
        self.audio_tx.subscribe()
    }

    /// Whether the socket handshake announced an audio stream.
    pub fn has_audio(&self) -> bool {
        self.audio_codec().is_some()
    }

    /// The audio codec, when the monitor has audio. `None` also while the
    /// handshake has not completed yet — see [`Self::wait_for_stream_info`].
    pub fn audio_codec(&self) -> Option<AudioCodec> {
        self.stream_info_rx.borrow().as_ref()?.audio_codec
    }

    /// Stream topology, once the socket handshake has completed.
    pub fn stream_info(&self) -> Option<StreamInfo> {
        *self.stream_info_rx.borrow()
    }

    /// Wait until the socket handshake has revealed the stream topology, or
    /// the timeout elapses (zmc not running / camera not delivering).
    pub async fn wait_for_stream_info(&self, timeout: Duration) -> Option<StreamInfo> {
        let mut rx = self.stream_info_rx.clone();
        let result = tokio::time::timeout(timeout, rx.wait_for(|info| info.is_some())).await;
        match result {
            Ok(Ok(info)) => *info,
            _ => None,
        }
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

    /// Get the current cached keyframe synchronously (non-blocking).
    ///
    /// Returns `None` if no keyframe has been seen yet by the reader task.
    pub fn cached_keyframe(&self) -> Option<CachedKeyframe> {
        self.keyframe_cache_rx.borrow().clone()
    }

    /// Subscribe to keyframe cache updates.
    ///
    /// Useful for waiting until the first keyframe is cached (cold start).
    pub fn subscribe_keyframe_cache(&self) -> watch::Receiver<Option<CachedKeyframe>> {
        self.keyframe_cache_rx.clone()
    }

    /// Get the number of video subscribers
    pub fn video_subscriber_count(&self) -> usize {
        self.video_tx.receiver_count()
    }

    /// Get the number of audio subscribers
    pub fn audio_subscriber_count(&self) -> usize {
        self.audio_tx.receiver_count()
    }
}

/// Router errors
#[derive(Debug, thiserror::Error)]
pub enum RouterError {
    #[error("Monitor {0} source not available")]
    SourceNotAvailable(u32),

    #[error("Monitor {0} stream socket not found")]
    SocketNotFound(u32),

    #[error("Failed to start reader for monitor {0}: {1}")]
    ReaderStartFailed(u32, String),

    #[error("Source error: {0}")]
    SourceError(#[from] SourceError),
}

/// Configuration for the source router
#[derive(Debug, Clone)]
pub struct RouterConfig {
    /// ZoneMinder stream-socket configuration
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

/// Unified source router that manages stream-socket readers and serves all
/// output protocols
pub struct SourceRouter {
    /// Active monitor sources
    active_sources: DashMap<u32, Arc<MonitorSource>>,
    /// Configuration
    config: RouterConfig,
    /// Optional sink for monitor EVENTs decoded off the stream sockets. When
    /// `Some`, each reader task forwards `MonitorEventEnvelope`s here (via
    /// `try_send`, so a slow/backed-up ingest never stalls the media reader).
    /// `None` means events are simply not ingested (e.g. tests, or DB absent).
    event_sink: Option<mpsc::Sender<MonitorEventEnvelope>>,
    /// Most-recent WebRTC startup timing per monitor, recorded by the signaling
    /// handler and surfaced on `/live/{id}/stats` to confirm cold-vs-warm.
    webrtc_startup: DashMap<u32, WebRtcStartupTiming>,
}

/// Server-side WebRTC startup profile for a monitor's most recent session.
/// All `*_ms` are milliseconds from WS connect, so the phases nest:
/// `get_source_ms ≤ offer_ms ≤ ice_connected_ms ≤ connected_ms ≤ first_rtp_ms`.
#[derive(Debug, Clone, Default)]
pub struct WebRtcStartupTiming {
    /// Reader was already hot at connect (no reader spin-up on the offer path).
    pub warm_start: bool,
    /// connect → `get_source` returned — the reader acquire/restart cost (the
    /// suspected cold-offer culprit).
    pub get_source_ms: Option<u64>,
    /// connect → SDP offer sent.
    pub offer_ms: Option<u64>,
    /// connect → ICE-connected (first usable candidate pair). With trickle ICE
    /// this is the lower bound of the offer→connected window; the remaining gap
    /// to `connected_ms` is the DTLS handshake, so the two split the ICE cost
    /// from the DTLS cost.
    pub ice_connected_ms: Option<u64>,
    /// connect → peer connection `Connected` (ICE + DTLS complete). The
    /// `connected_ms − ice_connected_ms` gap is the DTLS handshake.
    pub connected_ms: Option<u64>,
    /// connect → first video RTP written to the track.
    pub first_rtp_ms: Option<u64>,
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
            event_sink: None,
            webrtc_startup: DashMap::new(),
        }
    }

    /// Register a sink to receive monitor EVENTs decoded off the stream
    /// sockets. Call before any reader starts (i.e. before wrapping the router
    /// in an `Arc` / handing it to the coordinator). Readers started after this
    /// forward events to `sink`; without a sink, EVENTs are dropped after
    /// decoding.
    pub fn set_event_sink(&mut self, sink: mpsc::Sender<MonitorEventEnvelope>) {
        self.event_sink = Some(sink);
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

        if !self.is_available(monitor_id) {
            return Err(RouterError::SocketNotFound(monitor_id));
        }

        let source = Arc::new(MonitorSource::new(monitor_id));
        self.active_sources.insert(monitor_id, source.clone());

        info!("Created source for monitor {}", monitor_id);
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
        // Reuse an existing source, or create one. Either way we must still
        // ensure its reader task is running below: a source can outlive its
        // reader (`stop_reader` aborts the task but leaves the source in
        // `active_sources`), and a subscriber to a source whose reader is
        // dead hangs forever on the broadcast channel — it never closes
        // while the `MonitorSource` holds the `Sender`.
        let source = match self.active_sources.get(&monitor_id) {
            Some(source) => source.clone(),
            None => self.create_source(monitor_id).await?,
        };

        // Start the reader if auto_start is enabled. `start_reader` is
        // idempotent — a no-op when a reader task is already alive.
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

        // Skip only when a reader task is still alive. The `active` flag is
        // not a reliable "running" signal: it is false during normal
        // reconnect cycles, and it stays false on a source whose reader was
        // aborted by `stop_reader`. Keying off the `JoinHandle` instead means
        // a dead/aborted/panicked reader is correctly restarted, while a live
        // one (even mid-reconnect) is never duplicated.
        {
            let handle_guard = source.reader_handle.read().await;
            if let Some(handle) = handle_guard.as_ref() {
                if !handle.is_finished() {
                    debug!("Reader already running for monitor {}", monitor_id);
                    return Ok(());
                }
            }
        }

        // Create and start the reader task
        let config = self.config.zoneminder.clone();
        let video_tx = source.video_tx.clone();
        let audio_tx = source.audio_tx.clone();
        let audio_packets = source.audio_packets.clone();
        let source_for_task = source.clone();
        // Clone the health sender into the task — when the task exits (or is
        // aborted), this sender is dropped and subscribers see Err from changed().
        let health_tx = source.reader_health_tx.clone();
        let stream_info_tx = source.stream_info_tx.clone();
        let keyframe_cache_tx = source.keyframe_cache_tx.clone();
        let event_sink = self.event_sink.clone();

        let handle = tokio::spawn(async move {
            info!(
                "Starting stream socket reader task for monitor {}",
                monitor_id
            );

            // Outer loop: handles reconnection when the socket closes or errors
            loop {
                let mut reader = StreamSocketReader::new(monitor_id, config.clone());
                let _ = health_tx.send(ReaderHealth::Opening);

                // Keyframe cache state — tracks parameter sets across NALs so
                // we can assemble a complete CachedKeyframe when a keyframe
                // arrives. Re-initialized on each reconnect (new parameter
                // sets expected). The watch channel is NOT cleared — old
                // cache is still usable during brief reconnects.
                let mut keyframe_cache_builder = KeyframeCacheBuilder::default();

                match reader.connect().await {
                    Ok(()) => {
                        *source_for_task.active.write().await = true;
                        let _ = health_tx.send(ReaderHealth::Active);
                    }
                    Err(SourceError::NotFound { .. }) => {
                        debug!(
                            "Stream socket not found for monitor {}, waiting to retry...",
                            monitor_id
                        );
                        tokio::time::sleep(Duration::from_millis(config.reconnect_delay_ms * 5))
                            .await;
                        continue;
                    }
                    Err(e) => {
                        error!(
                            "Failed to connect stream socket for monitor {}: {}",
                            monitor_id, e
                        );
                        tokio::time::sleep(Duration::from_millis(config.reconnect_delay_ms)).await;
                        continue;
                    }
                }

                // Per-connection control channel for the id-assignment
                // handshake: ingest queues replies here and the reader task
                // writes them to this connection's write half. Recreated each
                // reconnect; the old channel + writer drop with the old socket.
                let mut writer = reader.take_writer();
                let (cmd_tx, mut cmd_rx) = mpsc::channel::<Vec<u8>>(8);
                let control_reply = ControlReply::new(cmd_tx.clone());
                let _cmd_keepalive = cmd_tx; // hold the channel open for this connection

                // Topology of this connection. zmc sends every stream's HELLO
                // before any media, so the first media event confirms the
                // handshake is complete and the audio answer is final.
                let mut video_codec = VideoCodec::Unknown;
                let mut audio_codec: Option<AudioCodec> = None;
                let mut announced = false;

                // Inner loop: read events until the socket closes or an
                // unrecoverable error occurs. `select!` also drains queued
                // control replies, writing them on this connection (biased so a
                // pending id-assignment is flushed before the next read).
                loop {
                    tokio::select! {
                        biased;
                        Some(bytes) = cmd_rx.recv() => {
                            if let Some(w) = writer.as_mut() {
                                if let Err(e) = w.write_all(&bytes).await {
                                    warn!("Monitor {}: control write failed: {}", monitor_id, e);
                                } else {
                                    let _ = w.flush().await;
                                }
                            }
                        }
                        result = reader.next_event() => {
                    match result {
                        Ok(SocketEvent::VideoParams { codec }) => {
                            video_codec = codec;
                            let mut codec_guard = source_for_task.codec.write().await;
                            if *codec_guard != codec {
                                *codec_guard = codec;
                                info!("Monitor {} video codec: {}", monitor_id, codec.as_str());
                            }
                        }
                        Ok(SocketEvent::AudioParams { codec }) => {
                            audio_codec = Some(codec);
                            if announced {
                                // Audio appeared after the initial handshake
                                // (e.g. zmc primed it later) — update.
                                let _ = stream_info_tx.send(Some(StreamInfo {
                                    video_codec,
                                    audio_codec,
                                }));
                            }
                        }
                        Ok(SocketEvent::Video(packet)) => {
                            if !announced {
                                announced = true;
                                let _ = stream_info_tx.send(Some(StreamInfo {
                                    video_codec,
                                    audio_codec,
                                }));
                            }

                            // --- Keyframe cache: assemble parameter sets +
                            // keyframe per the packet's codec ---
                            if let Some(cached) = keyframe_cache_builder.push(&packet) {
                                let _ = keyframe_cache_tx.send(Some(cached));
                                debug!("Updated keyframe cache for monitor {}", monitor_id);
                            }

                            // Broadcast the packet (ignore errors if no receivers)
                            if video_tx.send(packet).is_err() {
                                // No receivers - this is fine, just means no one is subscribed
                                debug!("No receivers for monitor {}", monitor_id);
                            }
                        }
                        Ok(SocketEvent::Audio(packet)) => {
                            if !announced {
                                announced = true;
                                let _ = stream_info_tx.send(Some(StreamInfo {
                                    video_codec,
                                    audio_codec,
                                }));
                            }
                            audio_packets.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            // No receivers is fine — nobody listening.
                            let _ = audio_tx.send(packet);
                        }
                        Ok(SocketEvent::MonitorEvent(event)) => {
                            // Forward to DB ingest. `try_send` keeps the media
                            // reader non-blocking: if ingest is backed up or
                            // absent we drop the event rather than stall video.
                            // `reply` lets ingest answer on this connection.
                            if let Some(sink) = &event_sink {
                                if let Err(e) = sink.try_send(MonitorEventEnvelope {
                                    monitor_id,
                                    event,
                                    reply: control_reply.clone(),
                                }) {
                                    warn!(
                                        "Monitor {}: dropping EVENT, ingest sink unavailable: {}",
                                        monitor_id, e
                                    );
                                }
                            }
                        }
                        Err(SourceError::Timeout { .. }) => {
                            // Expected when no media is flowing (idle camera);
                            // STATS messages keep a healthy connection chatty.
                            debug!("Read timeout for monitor {}, continuing...", monitor_id);
                            continue;
                        }
                        Err(SourceError::Closed) => {
                            warn!(
                                "Stream socket closed for monitor {}, will reconnect...",
                                monitor_id
                            );
                            break; // Break inner loop to reconnect
                        }
                        Err(e) => {
                            error!(
                                "Error reading stream socket for monitor {}: {}",
                                monitor_id, e
                            );
                            tokio::time::sleep(Duration::from_millis(config.reconnect_delay_ms))
                                .await;
                            break; // Break inner loop to reconnect with fresh state
                        }
                    }
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
    ) -> Result<broadcast::Receiver<VideoPacket>, RouterError> {
        let source = self.get_source(monitor_id).await?;
        Ok(source.subscribe_video())
    }

    /// Subscribe to audio packets for a monitor
    pub async fn subscribe_audio(
        &self,
        monitor_id: u32,
    ) -> Result<broadcast::Receiver<AudioPacket>, RouterError> {
        let source = self.get_source(monitor_id).await?;
        Ok(source.subscribe_audio())
    }

    /// Check if a monitor's stream socket exists (without creating a source).
    ///
    /// The socket appears when zmc starts and survives camera reconnects.
    pub fn is_available(&self, monitor_id: u32) -> bool {
        stream_socket_path(&self.config.zoneminder, monitor_id).exists()
    }

    /// Start a fresh WebRTC startup profile for a monitor at offer time,
    /// recording whether the reader was hot and how long `get_source` took.
    pub fn record_webrtc_offer(
        &self,
        monitor_id: u32,
        warm_start: bool,
        get_source_ms: u64,
        offer_ms: u64,
    ) {
        self.webrtc_startup.insert(
            monitor_id,
            WebRtcStartupTiming {
                warm_start,
                get_source_ms: Some(get_source_ms),
                offer_ms: Some(offer_ms),
                ice_connected_ms: None,
                connected_ms: None,
                first_rtp_ms: None,
            },
        );
    }

    /// Record connect → ICE-connected (first usable candidate pair), only the
    /// first per session.
    pub fn record_webrtc_ice_connected(&self, monitor_id: u32, ice_connected_ms: u64) {
        if let Some(mut t) = self.webrtc_startup.get_mut(&monitor_id) {
            if t.ice_connected_ms.is_none() {
                t.ice_connected_ms = Some(ice_connected_ms);
            }
        }
    }

    /// Record connect → peer `Connected` (ICE+DTLS done), only the first per
    /// session.
    pub fn record_webrtc_connected(&self, monitor_id: u32, connected_ms: u64) {
        if let Some(mut t) = self.webrtc_startup.get_mut(&monitor_id) {
            if t.connected_ms.is_none() {
                t.connected_ms = Some(connected_ms);
            }
        }
    }

    /// Record the WS-connect → first-video-RTP time (only the first per session).
    pub fn record_webrtc_first_rtp(&self, monitor_id: u32, first_rtp_ms: u64) {
        if let Some(mut t) = self.webrtc_startup.get_mut(&monitor_id) {
            if t.first_rtp_ms.is_none() {
                t.first_rtp_ms = Some(first_rtp_ms);
            }
        }
    }

    /// The most-recent WebRTC startup profile for a monitor, if any.
    pub fn webrtc_startup(&self, monitor_id: u32) -> Option<WebRtcStartupTiming> {
        self.webrtc_startup.get(&monitor_id).map(|t| t.clone())
    }

    /// Whether a monitor's reader is already hot: the source exists and its
    /// reader task is alive. Checked *before* `get_source` so the offer path can
    /// report whether it paid a reader spin-up.
    pub async fn is_reader_hot(&self, monitor_id: u32) -> bool {
        match self.get_existing_source(monitor_id) {
            Some(source) => {
                let guard = source.reader_handle.read().await;
                guard.as_ref().map(|h| !h.is_finished()).unwrap_or(false)
            }
            None => false,
        }
    }

    /// Ensure a monitor's reader is running, creating the source if needed.
    /// Idempotent; skips silently when the socket isn't present yet. Returns
    /// whether the reader is now (or already was) being kept warm.
    pub async fn ensure_warm(&self, monitor_id: u32) -> bool {
        if !self.is_available(monitor_id) {
            debug!("prewarm: monitor {monitor_id} socket not present yet; will retry");
            return false;
        }
        if let Err(e) = self.create_source(monitor_id).await {
            warn!("prewarm: create_source({monitor_id}) failed: {e}");
            return false;
        }
        if let Err(e) = self.start_reader(monitor_id).await {
            warn!("prewarm: start_reader({monitor_id}) failed: {e}");
            return false;
        }
        true
    }

    /// Spawn the warm-keeper: every `interval` it re-ensures each monitor in
    /// `monitors` has a live reader (restarting any stopped by the HLS reaper or
    /// a crash, and starting them once their socket appears). This keeps the
    /// keyframe cache hot so the first viewer skips cold spin-up. A `Weak` ref
    /// lets the task exit when the router is dropped. No-op when the list is
    /// empty or `interval` is zero.
    pub fn spawn_prewarm_task(self: &Arc<Self>, monitors: Vec<u32>, interval: Duration) {
        if monitors.is_empty() || interval.is_zero() {
            info!("source pre-warming disabled");
            return;
        }
        info!(
            "source pre-warming enabled for monitors {:?} (every {:?})",
            monitors, interval
        );
        let weak = Arc::downgrade(self);
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
            loop {
                ticker.tick().await;
                let Some(router) = weak.upgrade() else { break };
                for &monitor_id in &monitors {
                    router.ensure_warm(monitor_id).await;
                }
                // Release the strong ref before sleeping so it never pins the
                // router across the interval.
                drop(router);
            }
        });
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
            stats.push(Self::source_stats(entry.value()).await);
        }

        stats
    }

    /// Get statistics for a specific monitor
    pub async fn get_source_stats(&self, monitor_id: u32) -> Option<SourceStats> {
        let source = self.active_sources.get(&monitor_id)?.clone();
        Some(Self::source_stats(&source).await)
    }

    async fn source_stats(source: &MonitorSource) -> SourceStats {
        SourceStats {
            monitor_id: source.monitor_id,
            codec: source.codec().await,
            active: source.is_active().await,
            video_subscribers: source.video_subscriber_count(),
            audio_subscribers: source.audio_subscriber_count(),
            has_audio: source.has_audio(),
            audio_packets: source.audio_packet_count(),
        }
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
    pub audio_subscribers: usize,
    pub has_audio: bool,
    /// Audio packets read since the source was created
    pub audio_packets: u64,
}

#[cfg(test)]
mod tests {
    use super::super::protocol::test_encode::*;
    use super::super::protocol::FLAG_KEYFRAME;
    use super::super::stream_socket::test_support::*;
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
    fn test_monitor_source_creation() {
        let source = MonitorSource::new(1);
        assert_eq!(source.monitor_id(), 1);
        // Audio presence is unknown until the socket handshake completes.
        assert!(!source.has_audio());
        assert!(source.stream_info().is_none());
        assert_eq!(source.video_subscriber_count(), 0);
    }

    #[tokio::test]
    async fn test_monitor_source_initial_state() {
        let source = MonitorSource::new(1);
        assert!(!source.is_active().await);
        assert_eq!(source.codec().await, VideoCodec::Unknown);
    }

    #[test]
    fn test_is_available_nonexistent() {
        let router = SourceRouter::new();
        // This will return false for a non-existent monitor
        // since the stream socket won't exist
        assert!(!router.is_available(99999));
    }

    #[test]
    fn webrtc_startup_timing_records_phases() {
        let router = SourceRouter::new();
        assert!(router.webrtc_startup(3).is_none(), "no session yet");

        // A new session records warm flag + get_source/offer, clearing later phases.
        router.record_webrtc_offer(3, true, 5, 120);
        let t = router.webrtc_startup(3).unwrap();
        assert!(t.warm_start);
        assert_eq!(t.get_source_ms, Some(5));
        assert_eq!(t.offer_ms, Some(120));
        assert_eq!(t.ice_connected_ms, None);
        assert_eq!(t.connected_ms, None);
        assert_eq!(t.first_rtp_ms, None);

        // ICE-connected, Connected + first RTP are each recorded once; later
        // calls don't clobber.
        router.record_webrtc_ice_connected(3, 140);
        router.record_webrtc_ice_connected(3, 9999);
        router.record_webrtc_connected(3, 1240);
        router.record_webrtc_connected(3, 9999);
        router.record_webrtc_first_rtp(3, 1300);
        router.record_webrtc_first_rtp(3, 9999);
        let t = router.webrtc_startup(3).unwrap();
        assert_eq!(t.ice_connected_ms, Some(140));
        assert_eq!(t.connected_ms, Some(1240));
        assert_eq!(t.first_rtp_ms, Some(1300));

        // A fresh session resets the whole profile.
        router.record_webrtc_offer(3, false, 2100, 2130);
        let t = router.webrtc_startup(3).unwrap();
        assert!(!t.warm_start);
        assert_eq!(t.get_source_ms, Some(2100));
        assert_eq!(t.ice_connected_ms, None);
        assert_eq!(t.connected_ms, None);
        assert_eq!(t.first_rtp_ms, None);
    }

    #[test]
    fn test_monitor_source_cached_keyframe_initially_none() {
        let source = MonitorSource::new(1);
        assert!(source.cached_keyframe().is_none());
    }

    fn h264_packet(nal_byte: u8, payload: &[u8]) -> VideoPacket {
        let mut data = vec![0x00, 0x00, 0x00, 0x01, nal_byte];
        data.extend_from_slice(payload);
        VideoPacket {
            monitor_id: 1,
            timestamp_us: 100,
            data,
            is_keyframe: (nal_byte & 0x1F) == 5,
            codec: VideoCodec::H264,
        }
    }

    fn h265_packet(nal_type: u8, payload: &[u8]) -> VideoPacket {
        let mut data = vec![0x00, 0x00, 0x00, 0x01, nal_type << 1, 0x01];
        data.extend_from_slice(payload);
        VideoPacket {
            monitor_id: 1,
            timestamp_us: 100,
            data,
            is_keyframe: (16..=21).contains(&nal_type),
            codec: VideoCodec::H265,
        }
    }

    #[test]
    fn test_keyframe_cache_builder_h264_assembles_on_idr() {
        let mut builder = KeyframeCacheBuilder::default();

        // SPS (Main 5.1), PPS — no cache yet
        assert!(builder
            .push(&h264_packet(0x67, &[0x4D, 0x00, 0x33]))
            .is_none());
        assert!(builder.push(&h264_packet(0x68, &[0xCE, 0x3C])).is_none());
        // P-slice does not assemble
        assert!(builder.push(&h264_packet(0x41, &[0x9A])).is_none());

        // IDR assembles SPS+PPS+IDR
        let ck = builder
            .push(&h264_packet(0x65, &[0x88, 0x84]))
            .expect("cache entry on IDR");
        assert_eq!(ck.codec, VideoCodec::H264);
        assert_eq!(ck.profile_level_id, "4d0033");
        assert_eq!(ck.timestamp_us, 100);
        let starts = ck
            .keyframe_au
            .windows(4)
            .filter(|w| w == &[0, 0, 0, 1])
            .count();
        assert_eq!(starts, 3, "SPS+PPS+IDR expected in keyframe AU");
    }

    #[test]
    fn test_keyframe_cache_builder_h264_requires_sps_and_pps() {
        let mut builder = KeyframeCacheBuilder::default();
        // IDR with no parameter sets seen — nothing to cache
        assert!(builder.push(&h264_packet(0x65, &[0x88])).is_none());
        // Only SPS — still nothing
        assert!(builder
            .push(&h264_packet(0x67, &[0x4D, 0x00, 0x33]))
            .is_none());
        assert!(builder.push(&h264_packet(0x65, &[0x88])).is_none());
    }

    #[test]
    fn test_keyframe_cache_builder_h265_assembles_on_irap() {
        let mut builder = KeyframeCacheBuilder::default();

        // VPS (32), SPS (33), PPS (34)
        assert!(builder.push(&h265_packet(32, &[0x0C, 0x01])).is_none());
        assert!(builder.push(&h265_packet(33, &[0x01, 0x01])).is_none());
        assert!(builder.push(&h265_packet(34, &[0xC1, 0x62])).is_none());
        // TRAIL_R does not assemble
        assert!(builder.push(&h265_packet(1, &[0x9A])).is_none());

        // IDR_W_RADL (19) assembles VPS+SPS+PPS+IDR
        let ck = builder
            .push(&h265_packet(19, &[0x88]))
            .expect("cache entry on IRAP");
        assert_eq!(ck.codec, VideoCodec::H265);
        // profile_level_id is an H.264 SDP concept; empty for H.265
        assert!(ck.profile_level_id.is_empty());
        let starts = ck
            .keyframe_au
            .windows(4)
            .filter(|w| w == &[0, 0, 0, 1])
            .count();
        assert_eq!(starts, 4, "VPS+SPS+PPS+IDR expected in keyframe AU");
        // VPS must come first — the decoder cannot init without it
        assert_eq!((ck.keyframe_au[4] >> 1) & 0x3F, 32);
    }

    #[test]
    fn test_keyframe_cache_builder_h265_requires_vps() {
        let mut builder = KeyframeCacheBuilder::default();
        // SPS + PPS but no VPS — IRAP must not assemble a cache entry
        assert!(builder.push(&h265_packet(33, &[0x01])).is_none());
        assert!(builder.push(&h265_packet(34, &[0xC1])).is_none());
        assert!(builder.push(&h265_packet(19, &[0x88])).is_none());

        // VPS arrives; next IRAP assembles
        assert!(builder.push(&h265_packet(32, &[0x0C])).is_none());
        assert!(builder.push(&h265_packet(19, &[0x88])).is_some());
    }

    #[test]
    fn test_keyframe_cache_builder_h265_cra_assembles() {
        // CRA (21) is an IRAP — a valid stream entry point worth caching.
        let mut builder = KeyframeCacheBuilder::default();
        assert!(builder.push(&h265_packet(32, &[0x0C])).is_none());
        assert!(builder.push(&h265_packet(33, &[0x01])).is_none());
        assert!(builder.push(&h265_packet(34, &[0xC1])).is_none());
        assert!(builder.push(&h265_packet(21, &[0x88])).is_some());
    }

    #[test]
    fn test_cached_keyframe_populated_via_watch() {
        let source = MonitorSource::new(1);

        // Initially empty
        assert!(source.cached_keyframe().is_none());

        // Simulate the reader task populating the cache
        let cached = CachedKeyframe {
            sps: vec![0x00, 0x00, 0x00, 0x01, 0x67, 0x4D, 0x00, 0x33],
            pps: vec![0x00, 0x00, 0x00, 0x01, 0x68, 0xCE, 0x3C, 0x80],
            keyframe_au: vec![
                0x00, 0x00, 0x00, 0x01, 0x67, 0x4D, 0x00, 0x33, // SPS
                0x00, 0x00, 0x00, 0x01, 0x68, 0xCE, 0x3C, 0x80, // PPS
                0x00, 0x00, 0x00, 0x01, 0x65, 0x88, 0x84, 0x00, // IDR
            ],
            profile_level_id: "4d0033".to_string(),
            codec: VideoCodec::H264,
            timestamp_us: 12345,
        };
        let _ = source.keyframe_cache_tx.send(Some(cached));

        // Now cached_keyframe() should return the data
        let result = source.cached_keyframe();
        assert!(result.is_some());
        let ck = result.unwrap();
        assert_eq!(ck.profile_level_id, "4d0033");
        assert_eq!(ck.codec, VideoCodec::H264);
        assert_eq!(ck.timestamp_us, 12345);
        assert_eq!(ck.sps[4] & 0x1F, 7); // SPS NAL type
        assert_eq!(ck.pps[4] & 0x1F, 8); // PPS NAL type
    }

    #[test]
    fn test_subscribe_keyframe_cache_receives_updates() {
        let source = MonitorSource::new(1);
        let rx = source.subscribe_keyframe_cache();

        // Initially None
        assert!(rx.borrow().is_none());

        // Send a cached keyframe
        let cached = CachedKeyframe {
            sps: vec![0x00, 0x00, 0x00, 0x01, 0x67, 0x4D, 0x00, 0x33],
            pps: vec![0x00, 0x00, 0x00, 0x01, 0x68, 0xCE, 0x3C, 0x80],
            keyframe_au: vec![0x00, 0x00, 0x00, 0x01, 0x65, 0x88],
            profile_level_id: "4d0033".to_string(),
            codec: VideoCodec::H264,
            timestamp_us: 0,
        };
        let _ = source.keyframe_cache_tx.send(Some(cached));

        // Subscriber should see the update
        assert!(rx.borrow().is_some());
        assert_eq!(rx.borrow().as_ref().unwrap().profile_level_id, "4d0033");
    }

    /// Poll `is_active()` until it reaches `want`, or panic after ~2s.
    async fn await_active(router: &SourceRouter, monitor_id: u32, want: bool) {
        for _ in 0..200 {
            if let Some(source) = router.get_existing_source(monitor_id) {
                if source.is_active().await == want {
                    return;
                }
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        panic!("monitor {monitor_id} reader never reached active={want}");
    }

    fn test_zm_config(dir: &std::path::Path) -> ZoneMinderConfig {
        ZoneMinderConfig {
            socks_path: dir.to_string_lossy().into_owned(),
            ..ZoneMinderConfig::default()
        }
    }

    const SPS: &[u8] = &[0x00, 0x00, 0x00, 0x01, 0x67, 0x4D, 0x00, 0x33];
    const PPS: &[u8] = &[0x00, 0x00, 0x00, 0x01, 0x68, 0xCE, 0x3C, 0x80];

    fn h264_extradata() -> Vec<u8> {
        let mut e = SPS.to_vec();
        e.extend_from_slice(PPS);
        e
    }

    /// A canned zmc connect script: video HELLO (+ optional audio HELLO),
    /// then a keyframe AU.
    fn connect_script(with_audio: bool) -> Vec<u8> {
        let mut script = Vec::new();
        script.extend_from_slice(&encode_message(
            0x01,
            0,
            0,
            0,
            0,
            0,
            &hello_payload(h264_codec_id(), &h264_extradata()),
        ));
        if with_audio {
            script.extend_from_slice(&encode_message(
                0x01,
                1,
                0,
                0,
                0,
                0,
                &hello_payload(aac_codec_id(), &[0x14, 0x08]),
            ));
        }
        script.extend_from_slice(&encode_message(
            0x03,
            0,
            FLAG_KEYFRAME,
            0,
            0,
            1_000_000,
            &[0x00, 0x00, 0x00, 0x01, 0x65, 0x88, 0x84, 0x00],
        ));
        if with_audio {
            script.extend_from_slice(&encode_message(
                0x02,
                1,
                0,
                0,
                0,
                1_020_000,
                &[0x21, 0x1B, 0x55],
            ));
        }
        script
    }

    /// Regression test: a source can outlive its reader task. `stop_reader`
    /// aborts the reader but leaves the `MonitorSource` registered, so a
    /// later `get_source` must restart the reader instead of handing back a
    /// source whose broadcast channel has no producer (subscribers of such a
    /// source hang forever — the channel never closes while the `Sender`
    /// lives). This is the bug that froze WebRTC after the first keyframe.
    #[tokio::test]
    async fn test_get_source_restarts_dead_reader() {
        let dir = test_sock_dir("router_restart");
        let server = spawn_fake_zmc(dir.join("stream_7.sock"), connect_script(false), false);

        let router = SourceRouter::from_zoneminder_config(test_zm_config(&dir));

        // First acquisition creates the source and starts the reader.
        router
            .get_source(7)
            .await
            .expect("first get_source should succeed");
        await_active(&router, 7, true).await;

        // Simulate a session ending: the reader is aborted but the source
        // stays registered in `active_sources`.
        router
            .stop_reader(7)
            .await
            .expect("stop_reader should succeed");
        await_active(&router, 7, false).await;

        // Re-acquiring the existing source must bring the reader back to
        // life — not hand back a source with a dead broadcast channel.
        router
            .get_source(7)
            .await
            .expect("second get_source should succeed");
        await_active(&router, 7, true).await;

        let _ = router.stop_reader(7).await;
        server.abort();
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn ensure_warm_starts_reader_only_when_socket_present() {
        let dir = test_sock_dir("router_ensure_warm");
        let router = SourceRouter::from_zoneminder_config(test_zm_config(&dir));

        // No socket yet → warming is a no-op (the camera/zmc isn't up).
        assert!(
            !router.ensure_warm(7).await,
            "no socket present → not warmed"
        );

        // Socket appears → warming starts the reader, and is idempotent.
        let server = spawn_fake_zmc(dir.join("stream_7.sock"), connect_script(false), false);
        assert!(router.ensure_warm(7).await, "socket present → warmed");
        await_active(&router, 7, true).await;
        assert!(router.ensure_warm(7).await, "second call keeps it warm");

        let _ = router.stop_reader(7).await;
        server.abort();
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn prewarm_task_brings_reader_up() {
        let dir = test_sock_dir("router_prewarm_task");
        let server = spawn_fake_zmc(dir.join("stream_5.sock"), connect_script(false), false);
        let router = Arc::new(SourceRouter::from_zoneminder_config(test_zm_config(&dir)));

        // The warm-keeper's first tick fires immediately and starts the reader.
        router.spawn_prewarm_task(vec![5], Duration::from_millis(50));
        await_active(&router, 5, true).await;

        let _ = router.stop_reader(5).await;
        server.abort();
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// End-to-end: a fake zmc with video + audio streams. The reader task
    /// publishes stream info from the handshake, broadcasts ADTS-framed
    /// audio, and populates the keyframe cache from the keyframe message.
    #[tokio::test]
    async fn test_reader_broadcasts_av_and_announces_topology() {
        let dir = test_sock_dir("router_av");
        let server = spawn_fake_zmc(dir.join("stream_8.sock"), connect_script(true), false);

        let router = SourceRouter::from_zoneminder_config(test_zm_config(&dir));
        let source = router.create_source(8).await.expect("create_source");

        // Subscribe BEFORE starting the reader so nothing is missed.
        let mut video_rx = source.subscribe_video();
        let mut audio_rx = source.subscribe_audio();
        router.start_reader(8).await.expect("start_reader");

        // The handshake reveals the topology.
        let info = source
            .wait_for_stream_info(Duration::from_secs(5))
            .await
            .expect("stream info within 5s");
        assert_eq!(info.video_codec, VideoCodec::H264);
        assert_eq!(info.audio_codec, Some(AudioCodec::Aac));
        assert!(source.has_audio());

        // Video packets flow (extradata SPS/PPS prepended to the keyframe).
        let first = tokio::time::timeout(Duration::from_secs(5), video_rx.recv())
            .await
            .expect("video packet within 5s")
            .expect("video channel alive");
        assert_eq!(first.monitor_id, 8);
        assert_eq!(first.codec, VideoCodec::H264);

        // Audio is ADTS-framed from the HELLO's ASC.
        let audio = tokio::time::timeout(Duration::from_secs(5), audio_rx.recv())
            .await
            .expect("audio packet within 5s")
            .expect("audio channel alive");
        assert_eq!(audio.codec, AudioCodec::Aac);
        assert!(super::super::media::AdtsHeader::parse(&audio.data).is_some());
        assert_eq!(audio.timestamp_us, 20_000); // 20ms after the video keyframe
        assert!(source.audio_packet_count() >= 1);

        // The keyframe message populated the cache (SPS+PPS+IDR).
        for _ in 0..200 {
            if source.cached_keyframe().is_some() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        let ck = source.cached_keyframe().expect("keyframe cache populated");
        assert_eq!(ck.codec, VideoCodec::H264);
        assert_eq!(ck.profile_level_id, "4d0033");

        let _ = router.stop_reader(8).await;
        server.abort();
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The id-assignment handshake round-trips over a real socket: the worker
    /// emits a recording_opening EVENT, the consumer replies via the EVENT
    /// envelope, and the reply lands back on the same connection as a 0x11
    /// Command — exercising the reader's write half and ControlReply encoding.
    #[tokio::test]
    async fn test_control_reply_round_trips_to_worker() {
        use super::super::protocol::{
            parse_header, EVENT_RECORDING_OPENING, HEADER_SIZE, MSG_TYPE_COMMAND,
        };
        use super::super::stream_socket::test_support::spawn_fake_zmc_capturing;

        let dir = test_sock_dir("router_control");
        // Script: video HELLO, then a recording_opening EVENT on the monitor
        // stream carrying a clip_token.
        let mut script = encode_message(
            0x01,
            0,
            0,
            0,
            0,
            0,
            &hello_payload(h264_codec_id(), &h264_extradata()),
        );
        let opening = event_payload(
            EVENT_RECORDING_OPENING,
            &tlv(0x10, br#"{"clip_token":"tok-1"}"#),
        );
        script.extend_from_slice(&encode_message(0x06, 2, 0, 0, 0, 0, &opening));
        let (server, mut captured) = spawn_fake_zmc_capturing(dir.join("stream_20.sock"), script);

        let mut router = SourceRouter::from_zoneminder_config(test_zm_config(&dir));
        let (ev_tx, mut ev_rx) = mpsc::channel(8);
        router.set_event_sink(ev_tx);
        router.get_source(20).await.expect("get_source");

        // Receive the EVENT envelope and reply on its connection.
        let env = tokio::time::timeout(Duration::from_secs(5), ev_rx.recv())
            .await
            .expect("event within 5s")
            .expect("envelope");
        assert_eq!(env.monitor_id, 20);
        assert_eq!(env.event.code, EVENT_RECORDING_OPENING);
        assert!(env
            .reply
            .send_command_json(r#"{"cmd":"assign_recording","event_id":42}"#));

        // The worker reads the reply: a 0x11 Command framing our JSON.
        let mut buf = Vec::new();
        for _ in 0..50 {
            if let Ok(Some(chunk)) =
                tokio::time::timeout(Duration::from_millis(200), captured.recv()).await
            {
                buf.extend_from_slice(&chunk);
                if buf.len() >= HEADER_SIZE {
                    break;
                }
            }
        }
        let header = parse_header(buf[..HEADER_SIZE].try_into().unwrap()).expect("valid header");
        assert_eq!(header.msg_type, MSG_TYPE_COMMAND);
        assert_eq!(header.stream, 2); // Monitor
        let payload = &buf[HEADER_SIZE..HEADER_SIZE + header.payload_len];
        let json: serde_json::Value = serde_json::from_slice(payload).expect("json payload");
        assert_eq!(json["cmd"], "assign_recording");
        assert_eq!(json["event_id"], 42);

        let _ = router.stop_reader(20).await;
        server.abort();
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A monitor without audio: the handshake completes with no audio codec.
    #[tokio::test]
    async fn test_reader_announces_video_only_topology() {
        let dir = test_sock_dir("router_vo");
        let server = spawn_fake_zmc(dir.join("stream_9.sock"), connect_script(false), false);

        let router = SourceRouter::from_zoneminder_config(test_zm_config(&dir));
        let source = router.get_source(9).await.expect("get_source");

        let info = source
            .wait_for_stream_info(Duration::from_secs(5))
            .await
            .expect("stream info within 5s");
        assert_eq!(info.video_codec, VideoCodec::H264);
        assert_eq!(info.audio_codec, None);
        assert!(!source.has_audio());
        assert!(source.audio_codec().is_none());

        let _ = router.stop_reader(9).await;
        server.abort();
        let _ = std::fs::remove_dir_all(&dir);
    }
}
