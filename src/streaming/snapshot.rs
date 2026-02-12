//! Monitor snapshot service
//!
//! Captures H.264 keyframes from the FIFO broadcast pipeline and converts
//! them to JPEG images using libavcodec/libswscale (via `ffmpeg-next`).
//! Per-monitor caching minimizes overhead for repeated requests.

use std::sync::Arc;
use std::time::{Duration, Instant};

use dashmap::DashMap;
use tracing::{debug, warn};

use crate::streaming::source::router::RouterError;
use crate::streaming::source::SourceRouter;

/// Default cache TTL for snapshots
const DEFAULT_CACHE_TTL: Duration = Duration::from_secs(2);

/// Timeout for waiting for a keyframe from the broadcast channel
const KEYFRAME_TIMEOUT: Duration = Duration::from_secs(5);

/// JPEG quality (2 = high quality, 31 = lowest). Maps to ffmpeg's qscale.
const JPEG_QUALITY: u32 = 2;

/// Errors from the snapshot service
#[derive(Debug, thiserror::Error)]
pub enum SnapshotError {
    #[error("Source not available for monitor {0}")]
    SourceNotAvailable(u32),

    #[error("Keyframe capture timed out for monitor {0}")]
    KeyframeTimeout(u32),

    #[error("Decode failed: {0}")]
    DecodeFailed(String),

    #[error("Encode failed: {0}")]
    EncodeFailed(String),

    #[error("Router error: {0}")]
    RouterError(#[from] RouterError),
}

/// A cached JPEG snapshot
struct CachedSnapshot {
    jpeg: Vec<u8>,
    captured_at: Instant,
}

/// Service for capturing monitor snapshots as JPEG images
pub struct SnapshotService {
    source_router: Arc<SourceRouter>,
    cache: DashMap<u32, CachedSnapshot>,
    cache_ttl: Duration,
}

impl SnapshotService {
    /// Create a new snapshot service
    pub fn new(source_router: Arc<SourceRouter>, cache_ttl: Duration) -> Self {
        Self {
            source_router,
            cache: DashMap::new(),
            cache_ttl,
        }
    }

    /// Create with default TTL
    pub fn with_defaults(source_router: Arc<SourceRouter>) -> Self {
        Self::new(source_router, DEFAULT_CACHE_TTL)
    }

    /// Get a JPEG snapshot for a monitor, using cache when fresh
    pub async fn get_snapshot(&self, monitor_id: u32) -> Result<Vec<u8>, SnapshotError> {
        // Check cache
        if let Some(cached) = self.cache.get(&monitor_id) {
            if cached.captured_at.elapsed() < self.cache_ttl {
                debug!("Serving cached snapshot for monitor {}", monitor_id);
                return Ok(cached.jpeg.clone());
            }
        }

        // Capture fresh snapshot
        let h264_data = self.capture_keyframe(monitor_id).await?;
        let jpeg = Self::decode_to_jpeg(&h264_data).await?;

        // Update cache
        self.cache.insert(
            monitor_id,
            CachedSnapshot {
                jpeg: jpeg.clone(),
                captured_at: Instant::now(),
            },
        );

        Ok(jpeg)
    }

    /// Capture a keyframe (IDR) from the broadcast channel
    async fn capture_keyframe(&self, monitor_id: u32) -> Result<Vec<u8>, SnapshotError> {
        // Try to piggyback on an existing source first
        let (source, created_temp) = if let Some(source) =
            self.source_router.get_existing_source(monitor_id)
        {
            (source, false)
        } else {
            // Create a temporary source+reader
            let source = self
                .source_router
                .get_source(monitor_id)
                .await
                .map_err(|e| match e {
                    RouterError::FifoNotFound(_) => SnapshotError::SourceNotAvailable(monitor_id),
                    other => SnapshotError::RouterError(other),
                })?;
            (source, true)
        };

        let mut rx = source.subscribe_video();

        // Wait for a keyframe with timeout
        let result = tokio::time::timeout(KEYFRAME_TIMEOUT, async {
            let mut keyframe_data: Option<Vec<u8>> = None;
            let mut keyframe_ts: Option<i64> = None;

            loop {
                match rx.recv().await {
                    Ok(packet) => {
                        if let Some(ref mut data) = keyframe_data {
                            // We already have the keyframe; collect remaining NALs
                            // from the same access unit (same timestamp)
                            if let Some(ts) = keyframe_ts {
                                if packet.timestamp_us != ts {
                                    // Timestamp changed — access unit complete
                                    break;
                                }
                                data.extend_from_slice(&packet.data);
                            }
                        } else if packet.is_keyframe {
                            // Found the keyframe — start collecting
                            keyframe_data = Some(packet.data.clone());
                            keyframe_ts = Some(packet.timestamp_us);
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        warn!(
                            "Snapshot receiver lagged {} packets for monitor {}",
                            n, monitor_id
                        );
                        continue;
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        return Err(SnapshotError::SourceNotAvailable(monitor_id));
                    }
                }
            }

            keyframe_data.ok_or(SnapshotError::KeyframeTimeout(monitor_id))
        })
        .await;

        // Cleanup temporary source if we created one
        if created_temp {
            let _ = self.source_router.remove_source(monitor_id).await;
        }

        match result {
            Ok(Ok(data)) => Ok(data),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(SnapshotError::KeyframeTimeout(monitor_id)),
        }
    }

    /// Convert raw H.264 Annex B NAL data to JPEG using libavcodec/libswscale.
    ///
    /// Runs on a blocking thread since ffmpeg-next is synchronous.
    async fn decode_to_jpeg(h264_data: &[u8]) -> Result<Vec<u8>, SnapshotError> {
        let data = h264_data.to_vec();
        tokio::task::spawn_blocking(move || Self::decode_to_jpeg_blocking(&data))
            .await
            .map_err(|e| SnapshotError::DecodeFailed(format!("Task join error: {}", e)))?
    }

    /// Synchronous H.264 → JPEG conversion via libavcodec.
    fn decode_to_jpeg_blocking(h264_data: &[u8]) -> Result<Vec<u8>, SnapshotError> {
        use ffmpeg_next as ffmpeg;

        // Find H.264 decoder
        let decoder_codec = ffmpeg::codec::decoder::find(ffmpeg::codec::Id::H264)
            .ok_or_else(|| SnapshotError::DecodeFailed("H.264 decoder not found".into()))?;

        let mut decoder_ctx = ffmpeg::codec::Context::new_with_codec(decoder_codec)
            .decoder()
            .video()
            .map_err(|e| SnapshotError::DecodeFailed(format!("Failed to open decoder: {}", e)))?;

        // Feed the raw Annex B data as a single packet
        let mut av_packet = ffmpeg::Packet::copy(h264_data);
        av_packet.set_pts(Some(0));
        av_packet.set_dts(Some(0));

        decoder_ctx
            .send_packet(&av_packet)
            .map_err(|e| SnapshotError::DecodeFailed(format!("send_packet failed: {}", e)))?;
        decoder_ctx
            .send_eof()
            .map_err(|e| SnapshotError::DecodeFailed(format!("send_eof failed: {}", e)))?;

        // Receive decoded frame
        let mut decoded_frame = ffmpeg::frame::Video::empty();
        decoder_ctx
            .receive_frame(&mut decoded_frame)
            .map_err(|e| SnapshotError::DecodeFailed(format!("No frame decoded: {}", e)))?;

        if decoded_frame.width() == 0 || decoded_frame.height() == 0 {
            return Err(SnapshotError::DecodeFailed(
                "Decoded frame has zero dimensions".into(),
            ));
        }

        // Convert pixel format to YUVJ420P (MJPEG's native format)
        let target_format = ffmpeg::format::Pixel::YUVJ420P;
        let mut scaler = ffmpeg::software::scaling::Context::get(
            decoded_frame.format(),
            decoded_frame.width(),
            decoded_frame.height(),
            target_format,
            decoded_frame.width(),
            decoded_frame.height(),
            ffmpeg::software::scaling::Flags::BILINEAR,
        )
        .map_err(|e| SnapshotError::EncodeFailed(format!("Scaler init failed: {}", e)))?;

        let mut yuv_frame = ffmpeg::frame::Video::empty();
        scaler
            .run(&decoded_frame, &mut yuv_frame)
            .map_err(|e| SnapshotError::EncodeFailed(format!("Scaler run failed: {}", e)))?;

        // Encode as MJPEG
        let encoder_codec = ffmpeg::codec::encoder::find(ffmpeg::codec::Id::MJPEG)
            .ok_or_else(|| SnapshotError::EncodeFailed("MJPEG encoder not found".into()))?;

        let mut encoder_ctx = ffmpeg::codec::Context::new_with_codec(encoder_codec)
            .encoder()
            .video()
            .map_err(|e| SnapshotError::EncodeFailed(format!("Failed to init encoder: {}", e)))?;

        encoder_ctx.set_width(decoded_frame.width());
        encoder_ctx.set_height(decoded_frame.height());
        encoder_ctx.set_format(target_format);
        encoder_ctx.set_time_base(ffmpeg::Rational(1, 25));
        encoder_ctx.set_quality(JPEG_QUALITY as usize);

        // Set global quality via priv_data for MJPEG (qscale maps to quality)
        let encoder_ctx = encoder_ctx.open().map_err(|e| {
            SnapshotError::EncodeFailed(format!("Failed to open MJPEG encoder: {}", e))
        })?;

        // Wrap in a new scope to encode
        let mut encoder = encoder_ctx;
        yuv_frame.set_pts(Some(0));

        encoder
            .send_frame(&yuv_frame)
            .map_err(|e| SnapshotError::EncodeFailed(format!("send_frame failed: {}", e)))?;
        encoder
            .send_eof()
            .map_err(|e| SnapshotError::EncodeFailed(format!("encoder send_eof failed: {}", e)))?;

        let mut encoded_packet = ffmpeg::Packet::empty();
        encoder
            .receive_packet(&mut encoded_packet)
            .map_err(|e| SnapshotError::EncodeFailed(format!("No JPEG packet received: {}", e)))?;

        let jpeg_data = encoded_packet
            .data()
            .ok_or_else(|| SnapshotError::EncodeFailed("Encoded packet has no data".into()))?
            .to_vec();

        if jpeg_data.is_empty() {
            return Err(SnapshotError::EncodeFailed(
                "MJPEG encoder produced empty output".into(),
            ));
        }

        Ok(jpeg_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Generate a minimal H.264 Annex B bitstream (SPS + PPS + IDR) using libavcodec.
    ///
    /// Returns `None` if the libx264 encoder is not available.
    fn generate_test_h264() -> Option<Vec<u8>> {
        use ffmpeg_next as ffmpeg;

        let encoder_codec = ffmpeg::codec::encoder::find_by_name("libx264")?;
        let mut encoder_ctx = ffmpeg::codec::Context::new_with_codec(encoder_codec)
            .encoder()
            .video()
            .ok()?;

        encoder_ctx.set_width(64);
        encoder_ctx.set_height(64);
        encoder_ctx.set_format(ffmpeg::format::Pixel::YUV420P);
        encoder_ctx.set_time_base(ffmpeg::Rational(1, 25));
        encoder_ctx.set_gop(10);

        let mut opts = ffmpeg::Dictionary::new();
        opts.set("preset", "ultrafast");
        opts.set("tune", "zerolatency");

        let mut encoder = encoder_ctx.open_with(opts).ok()?;

        // Create a blank frame
        let mut frame = ffmpeg::frame::Video::new(ffmpeg::format::Pixel::YUV420P, 64, 64);
        // Fill Y plane with black (0), U/V with 128 (neutral)
        for plane in 0..3u32 {
            let data = frame.data_mut(plane as usize);
            let fill_val = if plane == 0 { 0u8 } else { 128u8 };
            data.fill(fill_val);
        }
        frame.set_pts(Some(0));

        encoder.send_frame(&frame).ok()?;
        encoder.send_eof().ok()?;

        // Collect all encoded packets — without GLOBAL_HEADER flag,
        // SPS/PPS are inline in the first packet's Annex B stream.
        let mut h264_bytes = Vec::new();

        let mut packet = ffmpeg::Packet::empty();
        while encoder.receive_packet(&mut packet).is_ok() {
            if let Some(data) = packet.data() {
                h264_bytes.extend_from_slice(data);
            }
        }

        if h264_bytes.is_empty() {
            None
        } else {
            Some(h264_bytes)
        }
    }

    #[tokio::test]
    async fn test_decode_to_jpeg_with_valid_h264() {
        ffmpeg_next::init().ok();

        let h264_data = match generate_test_h264() {
            Some(data) => data,
            None => {
                eprintln!("Skipping test: libx264 encoder not available");
                return;
            }
        };

        let jpeg = SnapshotService::decode_to_jpeg(&h264_data).await;
        match jpeg {
            Ok(data) => {
                assert!(data.len() > 2, "JPEG should have content");
                assert_eq!(data[0], 0xFF, "JPEG should start with SOI marker byte 1");
                assert_eq!(data[1], 0xD8, "JPEG should start with SOI marker byte 2");
            }
            Err(e) => {
                panic!("decode_to_jpeg failed: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_decode_to_jpeg_with_invalid_data() {
        ffmpeg_next::init().ok();

        let garbage = vec![0xDE, 0xAD, 0xBE, 0xEF];
        let result = SnapshotService::decode_to_jpeg(&garbage).await;
        assert!(result.is_err(), "Should fail on invalid H.264 data");
    }

    #[tokio::test]
    async fn test_cache_ttl() {
        use crate::configure::streaming::ZoneMinderConfig;
        use crate::streaming::source::router::RouterConfig;

        let config = RouterConfig {
            zoneminder: ZoneMinderConfig {
                fifo_base_path: "/tmp/zm_snapshot_test_nonexistent".to_string(),
                ..ZoneMinderConfig::default()
            },
            ..RouterConfig::default()
        };
        let router = Arc::new(SourceRouter::with_config(config));
        let service = SnapshotService::new(Arc::clone(&router), Duration::from_millis(100));

        // Manually insert a cached snapshot
        service.cache.insert(
            1,
            CachedSnapshot {
                jpeg: vec![0xFF, 0xD8, 0xFF, 0xE0], // fake JPEG SOI
                captured_at: Instant::now(),
            },
        );

        // Should return cached value
        let result = service.get_snapshot(1).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![0xFF, 0xD8, 0xFF, 0xE0]);

        // Wait for TTL to expire
        tokio::time::sleep(Duration::from_millis(150)).await;

        // After TTL, cache miss triggers capture which will fail (no FIFO)
        let result = service.get_snapshot(1).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_cache_miss_triggers_capture() {
        use crate::configure::streaming::ZoneMinderConfig;
        use crate::streaming::source::router::RouterConfig;

        let config = RouterConfig {
            zoneminder: ZoneMinderConfig {
                fifo_base_path: "/tmp/zm_snapshot_test_nonexistent2".to_string(),
                ..ZoneMinderConfig::default()
            },
            ..RouterConfig::default()
        };
        let router = Arc::new(SourceRouter::with_config(config));
        let service = SnapshotService::with_defaults(Arc::clone(&router));

        // Fresh monitor with no cache — should fail because FIFO doesn't exist
        let result = service.get_snapshot(999).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            SnapshotError::SourceNotAvailable(id) => assert_eq!(id, 999),
            SnapshotError::RouterError(RouterError::FifoNotFound(id)) => assert_eq!(id, 999),
            other => panic!("Unexpected error: {:?}", other),
        }
    }
}
