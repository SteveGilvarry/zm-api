//! fMP4 (Fragmented MP4) segment generation for HLS
//!
//! Generates ISO BMFF compliant fMP4 segments from H.264/H.265 NAL units.
//! Supports both initialization segments (ftyp + moov) and media segments (moof + mdat).

use bytes::{BufMut, BytesMut};
use std::time::Duration;
use tracing::warn;

use super::h264;
use crate::streaming::source::AdtsHeader;
use crate::streaming::source::{slice_starts_picture, VideoCodec};

/// fMP4 initialization segment containing ftyp and moov boxes
#[derive(Debug, Clone)]
pub struct InitSegment {
    pub data: Vec<u8>,
    pub codec: VideoCodec,
    pub width: u32,
    pub height: u32,
    pub timescale: u32,
    /// MPEG-4 Audio Object Type of the muxed audio track (2 = AAC-LC), or
    /// `None` when the init segment is video-only. Drives the playlist
    /// CODECS attribute (`mp4a.40.{aot}`).
    pub audio_object_type: Option<u8>,
}

/// fMP4 media segment containing moof and mdat boxes
#[derive(Debug, Clone)]
pub struct FMP4Segment {
    pub sequence: u64,
    pub data: Vec<u8>,
    pub duration: Duration,
    pub timestamp: u64,
    pub is_keyframe: bool,
}

/// Builder for fMP4 boxes
struct BoxBuilder {
    buffer: BytesMut,
}

impl BoxBuilder {
    fn new() -> Self {
        Self {
            buffer: BytesMut::with_capacity(4096),
        }
    }

    /// Write a box with the given type and data
    fn write_box(&mut self, box_type: &[u8; 4], data: &[u8]) {
        let size = 8 + data.len() as u32;
        self.buffer.put_u32(size);
        self.buffer.put_slice(box_type);
        self.buffer.put_slice(data);
    }

    /// Write a full box (with version and flags)
    fn write_full_box(&mut self, box_type: &[u8; 4], version: u8, flags: u32, data: &[u8]) {
        let size = 12 + data.len() as u32;
        self.buffer.put_u32(size);
        self.buffer.put_slice(box_type);
        self.buffer.put_u8(version);
        self.buffer.put_u8(((flags >> 16) & 0xFF) as u8);
        self.buffer.put_u8(((flags >> 8) & 0xFF) as u8);
        self.buffer.put_u8((flags & 0xFF) as u8);
        self.buffer.put_slice(data);
    }

    fn into_bytes(self) -> Vec<u8> {
        self.buffer.to_vec()
    }

    #[allow(dead_code)]
    fn len(&self) -> usize {
        self.buffer.len()
    }
}

/// A completed access unit (frame): one or more NALs that share a timestamp.
struct SegmentSample {
    nals: Vec<Vec<u8>>,
    timestamp: u64,
    is_keyframe: bool,
}

/// Frame currently being assembled from incoming NAL units.
struct PendingFrame {
    nals: Vec<Vec<u8>>,
    timestamp: u64,
    is_keyframe: bool,
}

/// Audio track parameters, derived from the first ADTS header seen.
#[derive(Debug, Clone, Copy)]
struct AudioParams {
    /// MPEG-4 Audio Object Type (2 = AAC-LC)
    object_type: u8,
    /// Sample rate in Hz; doubles as the audio track timescale, so one AAC
    /// frame is exactly 1024 ticks — no rounding drift.
    sample_rate: u32,
    channels: u8,
    /// 2-byte AudioSpecificConfig for the esds box
    asc: [u8; 2],
}

/// One AAC frame, ADTS header stripped, timestamp in audio-timescale ticks.
struct AudioSample {
    data: Vec<u8>,
    timestamp: u64,
}

/// Video samples to accumulate before giving up the wait for audio
/// parameters and generating a video-only init segment (~4 s at 25 fps).
/// The init segment is immutable once served, so when a source is expected
/// to carry audio, init generation waits for the first ADTS frame to
/// describe the track — but never indefinitely.
const AUDIO_PARAMS_WAIT_SAMPLES: u64 = 100;

/// HLS Segmenter for generating fMP4 segments
pub struct HlsSegmenter {
    monitor_id: u32,
    timescale: u32,
    sequence: u64,
    codec: VideoCodec,
    width: u32,
    height: u32,
    sps: Option<Vec<u8>>,
    pps: Option<Vec<u8>>,
    vps: Option<Vec<u8>>, // H.265 only
    init_segment: Option<InitSegment>,
    current_segment_samples: Vec<SegmentSample>,
    pending_frame: Option<PendingFrame>,
    segment_start_time: Option<u64>,
    target_duration: Duration,
    /// Whether we've received the first keyframe. Data before first keyframe is
    /// dropped because decoders cannot decode P-frames without a reference IDR.
    received_keyframe: bool,
    /// First VCL NAL timestamp (microseconds). All subsequent timestamps are
    /// normalized relative to this so that baseMediaDecodeTime in tfdt starts
    /// near zero. This avoids problems with hls.js PTS normalization when
    /// cameras have been running for many hours (large absolute timestamps).
    base_timestamp_us: Option<u64>,
    /// Whether the source is expected to deliver audio (the stream-socket
    /// handshake announced an audio stream). Gates init-segment generation on
    /// audio parameters so the audio track is described in the init that
    /// clients cache.
    expect_audio: bool,
    /// Audio track parameters from the first ADTS frame
    audio_params: Option<AudioParams>,
    /// Whether the generated init segment includes the audio track. Audio
    /// arriving after a video-only init was served must be dropped — the
    /// init cannot be changed retroactively.
    init_has_audio: bool,
    /// Audio samples accumulated for the current segment
    current_segment_audio: Vec<AudioSample>,
    /// Count of video samples processed, for the audio-params wait deadline
    video_samples_seen: u64,
    /// One-shot warning latches for non-muxable audio
    warned_non_adts: bool,
    warned_late_audio: bool,
}

impl HlsSegmenter {
    /// Create a new HLS segmenter
    pub fn new(monitor_id: u32, target_duration: Duration) -> Self {
        Self {
            monitor_id,
            timescale: 90000, // 90kHz for video
            sequence: 0,
            codec: VideoCodec::Unknown,
            width: 1920,
            height: 1080,
            sps: None,
            pps: None,
            vps: None,
            init_segment: None,
            current_segment_samples: Vec::new(),
            pending_frame: None,
            segment_start_time: None,
            target_duration,
            received_keyframe: false,
            base_timestamp_us: None,
            expect_audio: false,
            audio_params: None,
            init_has_audio: false,
            current_segment_audio: Vec::new(),
            video_samples_seen: 0,
            warned_non_adts: false,
            warned_late_audio: false,
        }
    }

    /// Set video dimensions
    pub fn set_dimensions(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    /// Set the codec
    pub fn set_codec(&mut self, codec: VideoCodec) {
        self.codec = codec;
    }

    /// Declare that the source delivers audio. Must be called before the
    /// init segment is generated; it makes init generation wait (bounded)
    /// for the first ADTS frame so the audio track is included.
    pub fn set_expect_audio(&mut self, expect_audio: bool) {
        self.expect_audio = expect_audio;
    }

    /// Process a NAL unit and potentially produce a segment
    pub fn process_nal(
        &mut self,
        nal_data: &[u8],
        timestamp_us: u64,
        is_keyframe: bool,
    ) -> Option<FMP4Segment> {
        // Extract parameter sets from NAL units
        self.extract_parameter_sets(nal_data);

        // Only VCL NALs (actual video slice data) go into media segments.
        // Non-VCL NALs (SPS, PPS, VPS, AUD, SEI, filler, etc.) are dropped:
        // - Parameter sets live in the init segment's avcC/hvcC record
        // - AUD/SEI/filler carry no decodable video data and cause decoder
        //   errors when packaged as standalone fMP4 samples
        if !self.is_vcl_nal(nal_data) {
            return None;
        }

        // Drop all frames before the first keyframe — decoders cannot decode
        // P/B-frames without a preceding IDR reference.
        if !self.received_keyframe {
            if is_keyframe {
                self.received_keyframe = true;
            } else {
                return None;
            }
        }

        // Normalize timestamps so the session starts near zero. Cameras that have
        // been running for hours produce huge absolute timestamps (e.g. 12+ hours at
        // 90kHz) which cause hls.js PTS normalization issues with v1 (64-bit) tfdt.
        let timestamp_us = {
            if self.base_timestamp_us.is_none() {
                self.base_timestamp_us = Some(timestamp_us);
            }
            timestamp_us.saturating_sub(self.base_timestamp_us.unwrap())
        };

        // Convert timestamp to timescale units
        let timestamp = (timestamp_us * self.timescale as u64) / 1_000_000;

        // Initialize segment start time
        if self.segment_start_time.is_none() {
            self.segment_start_time = Some(timestamp);
        }

        // Access unit aggregation: flush the pending frame when this NAL is
        // the first slice of a new coded picture. Grouping keys off the slice
        // header (`first_mb_in_slice == 0`), NOT the timestamp: a 4K picture
        // has many slices, and they can reach the segmenter spread across
        // ZoneMinder framed packets that each carry their own `pts`. Keying
        // off the timestamp would split one picture into several torn samples
        // (or, when every packet shares one `pts`, never flush at all and
        // produce zero segments). Continuation slices (`first_mb_in_slice > 0`)
        // belong to the picture already being assembled.
        let mut result = None;
        if let Some(pending) = self.pending_frame.take() {
            let is_new_frame = slice_starts_picture(nal_data, self.codec);
            if is_new_frame {
                // Flush the pending frame to completed samples
                self.current_segment_samples.push(SegmentSample {
                    nals: pending.nals,
                    timestamp: pending.timestamp,
                    is_keyframe: pending.is_keyframe,
                });
                self.video_samples_seen += 1;

                // Check if we should finalize segment (on keyframe after target duration)
                if is_keyframe && self.current_segment_samples.len() > 1 {
                    // `segment_start_time` is set to `Some` above whenever it was
                    // `None`, so it is always populated here; fall back to the
                    // current timestamp (zero-length segment) rather than panic.
                    let seg_start = self.segment_start_time.unwrap_or(timestamp);
                    let seg_start_us = seg_start * 1_000_000 / self.timescale as u64;
                    let segment_duration = Duration::from_micros(timestamp_us - seg_start_us);

                    if segment_duration >= self.target_duration {
                        result = self.finalize_segment(timestamp);
                        self.segment_start_time = Some(timestamp);
                    }
                }
            } else {
                // Same access unit (multi-slice frame) — keep accumulating.
                self.pending_frame = Some(pending);
            }
        }

        // Add current NAL to pending frame
        match self.pending_frame {
            Some(ref mut pending) => {
                pending.nals.push(nal_data.to_vec());
                pending.is_keyframe |= is_keyframe;
            }
            None => {
                self.pending_frame = Some(PendingFrame {
                    nals: vec![nal_data.to_vec()],
                    timestamp,
                    is_keyframe,
                });
            }
        }

        result
    }

    /// Flush the buffered access unit and emit the final (trailing) segment.
    ///
    /// Live streaming finalizes a segment only when the *next* keyframe arrives,
    /// so a finite/VOD source must call this at end-of-file to emit the last
    /// segment (and the only segment, for clips shorter than one target
    /// duration). Returns `None` if nothing is buffered.
    pub fn flush(&mut self) -> Option<FMP4Segment> {
        // Move the last in-progress access unit into the current segment.
        if let Some(pending) = self.pending_frame.take() {
            self.current_segment_samples.push(SegmentSample {
                nals: pending.nals,
                timestamp: pending.timestamp,
                is_keyframe: pending.is_keyframe,
            });
        }
        if self.current_segment_samples.is_empty() {
            return None;
        }
        // Estimate the final sample's duration from the trailing inter-sample
        // gap (fall back to ~1/30s) so its trun duration isn't degenerate.
        let n = self.current_segment_samples.len();
        let last_ts = self.current_segment_samples[n - 1].timestamp;
        let frame_dur = if n >= 2 {
            last_ts
                .saturating_sub(self.current_segment_samples[n - 2].timestamp)
                .max(1)
        } else {
            u64::from(self.timescale) / 30
        };
        self.finalize_segment(last_ts + frame_dur)
    }

    /// Process one audio packet (an ADTS-framed AAC frame from the source).
    ///
    /// Audio never produces a segment — video keyframes drive segment
    /// boundaries — it accumulates and is muxed into the next segment's
    /// audio `traf`. Frames are dropped when:
    /// * the data is not ADTS (raw RTSP AAC carries no codec parameters and
    ///   cannot be described in an fMP4 `esds`),
    /// * the first video keyframe has not arrived yet (segments start at a
    ///   keyframe; earlier audio belongs to no segment),
    /// * the init segment was already generated without an audio track.
    pub fn process_audio(&mut self, data: &[u8], timestamp_us: u64) {
        let Some(header) = AdtsHeader::parse(data) else {
            if !self.warned_non_adts {
                self.warned_non_adts = true;
                warn!(
                    "Monitor {}: audio is not ADTS-framed (raw AAC?); cannot derive \
                     AudioSpecificConfig — skipping HLS audio for this session",
                    self.monitor_id
                );
            }
            return;
        };

        if self.audio_params.is_none() {
            self.audio_params = Some(AudioParams {
                object_type: header.audio_object_type,
                sample_rate: header.sample_rate,
                channels: header.channel_configuration,
                asc: header.audio_specific_config(),
            });
        }

        // Init already served without audio — too late to add the track.
        if self.init_segment.is_some() && !self.init_has_audio {
            if !self.warned_late_audio {
                self.warned_late_audio = true;
                warn!(
                    "Monitor {}: first audio frame arrived after a video-only init \
                     segment was generated; HLS audio disabled for this session",
                    self.monitor_id
                );
            }
            return;
        }

        // Segments start at the first video keyframe; drop earlier audio.
        if !self.received_keyframe {
            return;
        }
        let Some(base_us) = self.base_timestamp_us else {
            return;
        };

        let sample_rate = self
            .audio_params
            .expect("audio_params set above")
            .sample_rate;
        let normalized_us = timestamp_us.saturating_sub(base_us);
        let timestamp = normalized_us * u64::from(sample_rate) / 1_000_000;

        // Strip the ADTS header — fMP4 carries raw AAC frames.
        let end = header.frame_len.min(data.len());
        self.current_segment_audio.push(AudioSample {
            data: data[header.header_len..end].to_vec(),
            timestamp,
        });
    }

    /// Extract SPS/PPS/VPS from NAL units
    fn extract_parameter_sets(&mut self, nal_data: &[u8]) {
        if nal_data.len() < 5 {
            return;
        }

        // Find start code offset
        let offset = if nal_data.starts_with(&[0x00, 0x00, 0x00, 0x01]) {
            4
        } else if nal_data.starts_with(&[0x00, 0x00, 0x01]) {
            3
        } else {
            return;
        };

        let nal_type_byte = nal_data[offset];

        match self.codec {
            VideoCodec::H264 => {
                let nal_type = nal_type_byte & 0x1F;
                match nal_type {
                    7 => {
                        // SPS
                        self.sps = Some(nal_data[offset..].to_vec());
                        self.parse_h264_sps(&nal_data[offset..]);
                    }
                    8 => {
                        // PPS
                        self.pps = Some(nal_data[offset..].to_vec());
                    }
                    _ => {}
                }
            }
            VideoCodec::H265 => {
                let nal_type = (nal_type_byte >> 1) & 0x3F;
                match nal_type {
                    32 => {
                        // VPS
                        self.vps = Some(nal_data[offset..].to_vec());
                    }
                    33 => {
                        // SPS
                        self.sps = Some(nal_data[offset..].to_vec());
                    }
                    34 => {
                        // PPS
                        self.pps = Some(nal_data[offset..].to_vec());
                    }
                    _ => {}
                }
            }
            VideoCodec::Unknown => {
                // Try to detect codec
                let h264_type = nal_type_byte & 0x1F;
                if h264_type == 7 || h264_type == 8 || h264_type == 5 {
                    self.codec = VideoCodec::H264;
                    self.extract_parameter_sets(nal_data);
                } else {
                    let h265_type = (nal_type_byte >> 1) & 0x3F;
                    if (32..=34).contains(&h265_type) {
                        self.codec = VideoCodec::H265;
                        self.extract_parameter_sets(nal_data);
                    }
                }
            }
        }
    }

    /// Parse H.264 SPS to extract actual video dimensions.
    fn parse_h264_sps(&mut self, sps: &[u8]) {
        if let Some((w, h)) = h264::parse_sps_dimensions(sps) {
            if w != self.width || h != self.height {
                tracing::info!(
                    monitor_id = self.monitor_id,
                    width = w,
                    height = h,
                    "SPS parsed: updating dimensions"
                );
                self.width = w;
                self.height = h;
                // Invalidate cached init segment so it's regenerated with new dimensions
                self.init_segment = None;
            }
        }
    }

    /// Generate initialization segment
    pub fn generate_init_segment(&mut self) -> Option<InitSegment> {
        if self.sps.is_none() || self.pps.is_none() {
            return None;
        }

        // The init segment is cached by clients and cannot change once
        // served. When audio is expected, hold init generation until the
        // first ADTS frame supplies the track parameters — but only for a
        // bounded number of video samples, so a silent/broken audio
        // stream cannot stall the video.
        if self.expect_audio
            && self.audio_params.is_none()
            && self.video_samples_seen < AUDIO_PARAMS_WAIT_SAMPLES
        {
            return None;
        }
        if self.expect_audio && self.audio_params.is_none() {
            warn!(
                "Monitor {}: no usable audio after {} video samples; \
                 generating video-only init segment",
                self.monitor_id, AUDIO_PARAMS_WAIT_SAMPLES
            );
        }

        self.init_has_audio = self.audio_params.is_some();

        let data = match self.codec {
            VideoCodec::H264 => self.generate_h264_init(),
            VideoCodec::H265 => self.generate_h265_init(),
            VideoCodec::Unknown => return None,
        };

        let init = InitSegment {
            data,
            codec: self.codec,
            width: self.width,
            height: self.height,
            timescale: self.timescale,
            audio_object_type: self.audio_params.map(|p| p.object_type),
        };

        self.init_segment = Some(init.clone());
        Some(init)
    }

    /// Get cached init segment
    pub fn get_init_segment(&self) -> Option<&InitSegment> {
        self.init_segment.as_ref()
    }

    /// Generate H.264 initialization segment
    fn generate_h264_init(&self) -> Vec<u8> {
        let mut builder = BoxBuilder::new();

        // ftyp box
        let ftyp_data = self.build_ftyp();
        builder.write_box(b"ftyp", &ftyp_data);

        // moov box
        let moov_data = self.build_moov_h264();
        builder.write_box(b"moov", &moov_data);

        builder.into_bytes()
    }

    /// Generate H.265 initialization segment
    fn generate_h265_init(&self) -> Vec<u8> {
        let mut builder = BoxBuilder::new();

        // ftyp box
        let ftyp_data = self.build_ftyp();
        builder.write_box(b"ftyp", &ftyp_data);

        // moov box
        let moov_data = self.build_moov_h265();
        builder.write_box(b"moov", &moov_data);

        builder.into_bytes()
    }

    /// Build ftyp box data
    fn build_ftyp(&self) -> Vec<u8> {
        let mut data = BytesMut::with_capacity(20);
        data.put_slice(b"isom"); // major brand
        data.put_u32(0x200); // minor version
        data.put_slice(b"isom"); // compatible brands
        data.put_slice(b"iso6");
        data.put_slice(b"mp41");
        data.to_vec()
    }

    /// Build moov box for H.264
    fn build_moov_h264(&self) -> Vec<u8> {
        let mut moov = BoxBuilder::new();

        // mvhd (movie header)
        let mvhd = self.build_mvhd();
        moov.write_full_box(b"mvhd", 0, 0, &mvhd);

        // trak (track)
        let trak = self.build_trak_h264();
        moov.write_box(b"trak", &trak);

        // audio trak (track 2), when the init includes audio
        if self.init_has_audio {
            if let Some(params) = self.audio_params {
                let trak = self.build_trak_audio(&params);
                moov.write_box(b"trak", &trak);
            }
        }

        // mvex (movie extends - for fragmented MP4)
        let mvex = self.build_mvex();
        moov.write_box(b"mvex", &mvex);

        moov.into_bytes()
    }

    /// Build moov box for H.265
    fn build_moov_h265(&self) -> Vec<u8> {
        let mut moov = BoxBuilder::new();

        // mvhd
        let mvhd = self.build_mvhd();
        moov.write_full_box(b"mvhd", 0, 0, &mvhd);

        // trak
        let trak = self.build_trak_h265();
        moov.write_box(b"trak", &trak);

        // audio trak (track 2), when the init includes audio
        if self.init_has_audio {
            if let Some(params) = self.audio_params {
                let trak = self.build_trak_audio(&params);
                moov.write_box(b"trak", &trak);
            }
        }

        // mvex
        let mvex = self.build_mvex();
        moov.write_box(b"mvex", &mvex);

        moov.into_bytes()
    }

    /// Build mvhd box data
    fn build_mvhd(&self) -> Vec<u8> {
        let mut data = BytesMut::with_capacity(96);
        data.put_u32(0); // creation_time
        data.put_u32(0); // modification_time
        data.put_u32(self.timescale); // timescale
        data.put_u32(0); // duration (unknown for live)
        data.put_u32(0x00010000); // rate = 1.0
        data.put_u16(0x0100); // volume = 1.0
        data.put_u16(0); // reserved
        data.put_u64(0); // reserved
                         // Matrix (identity)
        data.put_u32(0x00010000);
        data.put_u32(0);
        data.put_u32(0);
        data.put_u32(0);
        data.put_u32(0x00010000);
        data.put_u32(0);
        data.put_u32(0);
        data.put_u32(0);
        data.put_u32(0x40000000);
        // Pre-defined
        for _ in 0..6 {
            data.put_u32(0);
        }
        data.put_u32(2); // next_track_ID
        data.to_vec()
    }

    /// Build trak box for H.264
    fn build_trak_h264(&self) -> Vec<u8> {
        let mut trak = BoxBuilder::new();

        // tkhd
        let tkhd = self.build_tkhd();
        trak.write_full_box(b"tkhd", 0, 0x000007, &tkhd);

        // mdia
        let mdia = self.build_mdia_h264();
        trak.write_box(b"mdia", &mdia);

        trak.into_bytes()
    }

    /// Build trak box for H.265
    fn build_trak_h265(&self) -> Vec<u8> {
        let mut trak = BoxBuilder::new();

        // tkhd
        let tkhd = self.build_tkhd();
        trak.write_full_box(b"tkhd", 0, 0x000007, &tkhd);

        // mdia
        let mdia = self.build_mdia_h265();
        trak.write_box(b"mdia", &mdia);

        trak.into_bytes()
    }

    /// Build tkhd box data
    fn build_tkhd(&self) -> Vec<u8> {
        let mut data = BytesMut::with_capacity(80);
        data.put_u32(0); // creation_time
        data.put_u32(0); // modification_time
        data.put_u32(1); // track_ID
        data.put_u32(0); // reserved
        data.put_u32(0); // duration
        data.put_u64(0); // reserved
        data.put_u16(0); // layer
        data.put_u16(0); // alternate_group
        data.put_u16(0); // volume (0 for video)
        data.put_u16(0); // reserved
                         // Matrix
        data.put_u32(0x00010000);
        data.put_u32(0);
        data.put_u32(0);
        data.put_u32(0);
        data.put_u32(0x00010000);
        data.put_u32(0);
        data.put_u32(0);
        data.put_u32(0);
        data.put_u32(0x40000000);
        data.put_u32(self.width << 16); // width (16.16 fixed point)
        data.put_u32(self.height << 16); // height (16.16 fixed point)
        data.to_vec()
    }

    /// Build mdia box for H.264
    fn build_mdia_h264(&self) -> Vec<u8> {
        let mut mdia = BoxBuilder::new();

        // mdhd
        let mdhd = self.build_mdhd();
        mdia.write_full_box(b"mdhd", 0, 0, &mdhd);

        // hdlr
        let hdlr = self.build_hdlr();
        mdia.write_full_box(b"hdlr", 0, 0, &hdlr);

        // minf
        let minf = self.build_minf_h264();
        mdia.write_box(b"minf", &minf);

        mdia.into_bytes()
    }

    /// Build mdia box for H.265
    fn build_mdia_h265(&self) -> Vec<u8> {
        let mut mdia = BoxBuilder::new();

        // mdhd
        let mdhd = self.build_mdhd();
        mdia.write_full_box(b"mdhd", 0, 0, &mdhd);

        // hdlr
        let hdlr = self.build_hdlr();
        mdia.write_full_box(b"hdlr", 0, 0, &hdlr);

        // minf
        let minf = self.build_minf_h265();
        mdia.write_box(b"minf", &minf);

        mdia.into_bytes()
    }

    /// Build mdhd box data
    fn build_mdhd(&self) -> Vec<u8> {
        let mut data = BytesMut::with_capacity(20);
        data.put_u32(0); // creation_time
        data.put_u32(0); // modification_time
        data.put_u32(self.timescale); // timescale
        data.put_u32(0); // duration
        data.put_u16(0x55C4); // language = 'und'
        data.put_u16(0); // pre_defined
        data.to_vec()
    }

    /// Build hdlr box data
    fn build_hdlr(&self) -> Vec<u8> {
        let mut data = BytesMut::with_capacity(32);
        data.put_u32(0); // pre_defined
        data.put_slice(b"vide"); // handler_type
        for _ in 0..3 {
            data.put_u32(0); // reserved
        }
        data.put_slice(b"VideoHandler\0"); // name
        data.to_vec()
    }

    /// Build the audio trak (track 2): AAC in fMP4.
    fn build_trak_audio(&self, params: &AudioParams) -> Vec<u8> {
        let mut trak = BoxBuilder::new();

        // tkhd — flags 7 (enabled | in_movie | in_preview)
        let tkhd = Self::build_tkhd_audio();
        trak.write_full_box(b"tkhd", 0, 0x000007, &tkhd);

        // mdia
        let mdia = self.build_mdia_audio(params);
        trak.write_box(b"mdia", &mdia);

        trak.into_bytes()
    }

    /// Build tkhd box data for the audio track
    fn build_tkhd_audio() -> Vec<u8> {
        let mut data = BytesMut::with_capacity(80);
        data.put_u32(0); // creation_time
        data.put_u32(0); // modification_time
        data.put_u32(2); // track_ID
        data.put_u32(0); // reserved
        data.put_u32(0); // duration
        data.put_u64(0); // reserved
        data.put_u16(0); // layer
        data.put_u16(0); // alternate_group
        data.put_u16(0x0100); // volume = 1.0 (audio)
        data.put_u16(0); // reserved
                         // Matrix (identity)
        data.put_u32(0x00010000);
        data.put_u32(0);
        data.put_u32(0);
        data.put_u32(0);
        data.put_u32(0x00010000);
        data.put_u32(0);
        data.put_u32(0);
        data.put_u32(0);
        data.put_u32(0x40000000);
        data.put_u32(0); // width (0 for audio)
        data.put_u32(0); // height (0 for audio)
        data.to_vec()
    }

    /// Build mdia box for the audio track
    fn build_mdia_audio(&self, params: &AudioParams) -> Vec<u8> {
        let mut mdia = BoxBuilder::new();

        // mdhd — the audio track's timescale is its sample rate, making one
        // AAC frame exactly 1024 ticks.
        let mut mdhd = BytesMut::with_capacity(20);
        mdhd.put_u32(0); // creation_time
        mdhd.put_u32(0); // modification_time
        mdhd.put_u32(params.sample_rate); // timescale
        mdhd.put_u32(0); // duration
        mdhd.put_u16(0x55C4); // language = 'und'
        mdhd.put_u16(0); // pre_defined
        mdia.write_full_box(b"mdhd", 0, 0, &mdhd);

        // hdlr
        let mut hdlr = BytesMut::with_capacity(32);
        hdlr.put_u32(0); // pre_defined
        hdlr.put_slice(b"soun"); // handler_type
        for _ in 0..3 {
            hdlr.put_u32(0); // reserved
        }
        hdlr.put_slice(b"SoundHandler\0"); // name
        mdia.write_full_box(b"hdlr", 0, 0, &hdlr);

        // minf
        let mut minf = BoxBuilder::new();

        // smhd (sound media header)
        let mut smhd = BytesMut::with_capacity(4);
        smhd.put_u16(0); // balance
        smhd.put_u16(0); // reserved
        minf.write_full_box(b"smhd", 0, 0, &smhd);

        // dinf (same self-contained data reference as video)
        let dinf = self.build_dinf();
        minf.write_box(b"dinf", &dinf);

        // stbl
        let stbl = self.build_stbl_audio(params);
        minf.write_box(b"stbl", &stbl);

        mdia.write_box(b"minf", &minf.into_bytes());

        mdia.into_bytes()
    }

    /// Build stbl box for the audio track (empty sample tables — samples
    /// live in fragments)
    fn build_stbl_audio(&self, params: &AudioParams) -> Vec<u8> {
        let mut stbl = BoxBuilder::new();

        // stsd with the mp4a sample entry
        let mut stsd = BytesMut::with_capacity(128);
        stsd.put_u32(1); // entry_count
        let mp4a = Self::build_mp4a(params);
        stsd.put_slice(&mp4a);
        stbl.write_full_box(b"stsd", 0, 0, &stsd);

        // Empty stts, stsc, stsz, stco
        stbl.write_full_box(b"stts", 0, 0, &[0u8; 4]);
        stbl.write_full_box(b"stsc", 0, 0, &[0u8; 4]);
        stbl.write_full_box(b"stsz", 0, 0, &[0u8; 8]);
        stbl.write_full_box(b"stco", 0, 0, &[0u8; 4]);

        stbl.into_bytes()
    }

    /// Build the complete mp4a sample entry box (including esds)
    fn build_mp4a(params: &AudioParams) -> Vec<u8> {
        let mut entry = BytesMut::with_capacity(96);

        // SampleEntry header
        entry.put_slice(&[0u8; 6]); // reserved
        entry.put_u16(1); // data_reference_index

        // AudioSampleEntry
        entry.put_u64(0); // reserved
        entry.put_u16(u16::from(params.channels)); // channelcount
        entry.put_u16(16); // samplesize
        entry.put_u16(0); // pre_defined
        entry.put_u16(0); // reserved
        entry.put_u32(params.sample_rate << 16); // samplerate, 16.16 fixed

        // esds (MPEG-4 Elementary Stream Descriptor)
        let esds = Self::build_esds(params);
        entry.put_u32(12 + esds.len() as u32); // full box size
        entry.put_slice(b"esds");
        entry.put_u32(0); // version + flags
        entry.put_slice(&esds);

        let mut boxed = BoxBuilder::new();
        boxed.write_box(b"mp4a", &entry);
        boxed.into_bytes()
    }

    /// Build the esds descriptor chain (ISO 14496-1 §7.2.6).
    ///
    /// All descriptor payloads are tiny, so single-byte lengths suffice.
    fn build_esds(params: &AudioParams) -> Vec<u8> {
        let asc = params.asc;

        // DecoderSpecificInfo (tag 0x05): the AudioSpecificConfig
        let dsi_len = asc.len() as u8; // 2

        // DecoderConfigDescriptor (tag 0x04)
        let dcd_len = 13 + 2 + dsi_len; // fixed fields + nested DSI

        // ES_Descriptor (tag 0x03)
        let esd_len = 3 + 2 + dcd_len + 2 + 1; // ES fields + DCD + SLConfig

        let mut d = BytesMut::with_capacity(64);
        d.put_u8(0x03); // ES_Descriptor tag
        d.put_u8(esd_len);
        d.put_u16(2); // ES_ID (track 2)
        d.put_u8(0); // flags

        d.put_u8(0x04); // DecoderConfigDescriptor tag
        d.put_u8(dcd_len);
        d.put_u8(0x40); // objectTypeIndication: MPEG-4 Audio
        d.put_u8(0x15); // streamType audio (0x05 << 2) | reserved 1
        d.put_slice(&[0, 0, 0]); // bufferSizeDB
        d.put_u32(0); // maxBitrate (unknown)
        d.put_u32(0); // avgBitrate (unknown)

        d.put_u8(0x05); // DecoderSpecificInfo tag
        d.put_u8(dsi_len);
        d.put_slice(&asc);

        d.put_u8(0x06); // SLConfigDescriptor tag
        d.put_u8(1);
        d.put_u8(0x02); // predefined: MP4

        d.to_vec()
    }

    /// Build minf box for H.264
    fn build_minf_h264(&self) -> Vec<u8> {
        let mut minf = BoxBuilder::new();

        // vmhd
        let vmhd = self.build_vmhd();
        minf.write_full_box(b"vmhd", 0, 1, &vmhd);

        // dinf
        let dinf = self.build_dinf();
        minf.write_box(b"dinf", &dinf);

        // stbl
        let stbl = self.build_stbl_h264();
        minf.write_box(b"stbl", &stbl);

        minf.into_bytes()
    }

    /// Build minf box for H.265
    fn build_minf_h265(&self) -> Vec<u8> {
        let mut minf = BoxBuilder::new();

        // vmhd
        let vmhd = self.build_vmhd();
        minf.write_full_box(b"vmhd", 0, 1, &vmhd);

        // dinf
        let dinf = self.build_dinf();
        minf.write_box(b"dinf", &dinf);

        // stbl
        let stbl = self.build_stbl_h265();
        minf.write_box(b"stbl", &stbl);

        minf.into_bytes()
    }

    /// Build vmhd box data
    fn build_vmhd(&self) -> Vec<u8> {
        let mut data = BytesMut::with_capacity(8);
        data.put_u16(0); // graphicsmode
        data.put_u16(0); // opcolor[0]
        data.put_u16(0); // opcolor[1]
        data.put_u16(0); // opcolor[2]
        data.to_vec()
    }

    /// Build dinf box
    fn build_dinf(&self) -> Vec<u8> {
        let mut dinf = BoxBuilder::new();

        // dref
        let mut dref_data = BytesMut::with_capacity(16);
        dref_data.put_u32(1); // entry_count
                              // url entry (self-contained)
        dref_data.put_u32(12); // size
        dref_data.put_slice(b"url ");
        dref_data.put_u32(1); // version=0, flags=1 (self-contained)

        dinf.write_full_box(b"dref", 0, 0, &dref_data);

        dinf.into_bytes()
    }

    /// Build stbl box for H.264
    fn build_stbl_h264(&self) -> Vec<u8> {
        let mut stbl = BoxBuilder::new();

        // stsd
        let stsd = self.build_stsd_h264();
        stbl.write_full_box(b"stsd", 0, 0, &stsd);

        // stts (empty for fragmented)
        stbl.write_full_box(b"stts", 0, 0, &[0, 0, 0, 0]);

        // stsc (empty for fragmented)
        stbl.write_full_box(b"stsc", 0, 0, &[0, 0, 0, 0]);

        // stsz (empty for fragmented)
        let mut stsz = BytesMut::with_capacity(8);
        stsz.put_u32(0); // sample_size
        stsz.put_u32(0); // sample_count
        stbl.write_full_box(b"stsz", 0, 0, &stsz);

        // stco (empty for fragmented)
        stbl.write_full_box(b"stco", 0, 0, &[0, 0, 0, 0]);

        stbl.into_bytes()
    }

    /// Build stbl box for H.265
    fn build_stbl_h265(&self) -> Vec<u8> {
        let mut stbl = BoxBuilder::new();

        // stsd
        let stsd = self.build_stsd_h265();
        stbl.write_full_box(b"stsd", 0, 0, &stsd);

        // stts
        stbl.write_full_box(b"stts", 0, 0, &[0, 0, 0, 0]);

        // stsc
        stbl.write_full_box(b"stsc", 0, 0, &[0, 0, 0, 0]);

        // stsz
        let mut stsz = BytesMut::with_capacity(8);
        stsz.put_u32(0);
        stsz.put_u32(0);
        stbl.write_full_box(b"stsz", 0, 0, &stsz);

        // stco
        stbl.write_full_box(b"stco", 0, 0, &[0, 0, 0, 0]);

        stbl.into_bytes()
    }

    /// Build stsd box for H.264
    fn build_stsd_h264(&self) -> Vec<u8> {
        let mut data = BytesMut::with_capacity(256);
        data.put_u32(1); // entry_count

        // avc1 sample entry
        let avc1 = self.build_avc1();
        data.put_slice(&avc1);

        data.to_vec()
    }

    /// Build stsd box for H.265
    fn build_stsd_h265(&self) -> Vec<u8> {
        let mut data = BytesMut::with_capacity(256);
        data.put_u32(1); // entry_count

        // hvc1 sample entry
        let hvc1 = self.build_hvc1();
        data.put_slice(&hvc1);

        data.to_vec()
    }

    /// Build avc1 sample entry
    fn build_avc1(&self) -> Vec<u8> {
        // Reached only via `generate_init_segment`, which returns early when
        // either SPS or PPS is missing — so both are guaranteed present here.
        let sps = self
            .sps
            .as_ref()
            .expect("SPS present: generate_init_segment guards against None");
        let pps = self
            .pps
            .as_ref()
            .expect("PPS present: generate_init_segment guards against None");

        // Build avcC
        let mut avcc = BytesMut::with_capacity(128);
        avcc.put_u8(1); // configurationVersion
        avcc.put_u8(sps.get(1).copied().unwrap_or(0x42)); // AVCProfileIndication
        avcc.put_u8(sps.get(2).copied().unwrap_or(0x00)); // profile_compatibility
        avcc.put_u8(sps.get(3).copied().unwrap_or(0x1E)); // AVCLevelIndication
        avcc.put_u8(0xFF); // lengthSizeMinusOne = 3 (4 bytes)
        avcc.put_u8(0xE1); // numOfSequenceParameterSets = 1
        avcc.put_u16(sps.len() as u16);
        avcc.put_slice(sps);
        avcc.put_u8(1); // numOfPictureParameterSets
        avcc.put_u16(pps.len() as u16);
        avcc.put_slice(pps);

        let avcc_size = 8 + avcc.len();

        // Build avc1 box
        let mut avc1 = BytesMut::with_capacity(256);
        let avc1_size = 8 + 78 + avcc_size;
        avc1.put_u32(avc1_size as u32);
        avc1.put_slice(b"avc1");

        // Reserved
        for _ in 0..6 {
            avc1.put_u8(0);
        }
        avc1.put_u16(1); // data_reference_index

        // Pre-defined and reserved
        avc1.put_u16(0);
        avc1.put_u16(0);
        for _ in 0..3 {
            avc1.put_u32(0);
        }

        avc1.put_u16(self.width as u16);
        avc1.put_u16(self.height as u16);
        avc1.put_u32(0x00480000); // horizresolution = 72 dpi
        avc1.put_u32(0x00480000); // vertresolution = 72 dpi
        avc1.put_u32(0); // reserved
        avc1.put_u16(1); // frame_count
        for _ in 0..32 {
            avc1.put_u8(0); // compressorname
        }
        avc1.put_u16(0x0018); // depth = 24
        avc1.put_i16(-1); // pre_defined

        // avcC box
        avc1.put_u32(avcc_size as u32);
        avc1.put_slice(b"avcC");
        avc1.put_slice(&avcc);

        avc1.to_vec()
    }

    /// Build hvc1 sample entry
    fn build_hvc1(&self) -> Vec<u8> {
        // Reached only via `generate_init_segment`, which returns early when
        // either SPS or PPS is missing — so both are guaranteed present here.
        let sps = self
            .sps
            .as_ref()
            .expect("SPS present: generate_init_segment guards against None");
        let pps = self
            .pps
            .as_ref()
            .expect("PPS present: generate_init_segment guards against None");
        let vps = self.vps.as_ref();

        // Build hvcC (simplified)
        let mut hvcc = BytesMut::with_capacity(256);
        hvcc.put_u8(1); // configurationVersion
        hvcc.put_u8(0); // general_profile_space, general_tier_flag, general_profile_idc
        hvcc.put_u32(0); // general_profile_compatibility_flags
        hvcc.put_slice(&[0u8; 6]); // general_constraint_indicator_flags
        hvcc.put_u8(0); // general_level_idc
        hvcc.put_u16(0xF000); // min_spatial_segmentation_idc
        hvcc.put_u8(0xFC); // parallelismType
        hvcc.put_u8(0xFD); // chromaFormat
        hvcc.put_u8(0xF8); // bitDepthLumaMinus8
        hvcc.put_u8(0xF8); // bitDepthChromaMinus8
        hvcc.put_u16(0); // avgFrameRate
        hvcc.put_u8(0x0F); // constantFrameRate, numTemporalLayers, etc.

        // Number of arrays
        let num_arrays = if vps.is_some() { 3 } else { 2 };
        hvcc.put_u8(num_arrays);

        // VPS array
        if let Some(vps_data) = vps {
            hvcc.put_u8(0x20); // array_completeness=0, NAL_unit_type=32 (VPS)
            hvcc.put_u16(1); // numNalus
            hvcc.put_u16(vps_data.len() as u16);
            hvcc.put_slice(vps_data);
        }

        // SPS array
        hvcc.put_u8(0x21); // NAL_unit_type=33 (SPS)
        hvcc.put_u16(1);
        hvcc.put_u16(sps.len() as u16);
        hvcc.put_slice(sps);

        // PPS array
        hvcc.put_u8(0x22); // NAL_unit_type=34 (PPS)
        hvcc.put_u16(1);
        hvcc.put_u16(pps.len() as u16);
        hvcc.put_slice(pps);

        let hvcc_size = 8 + hvcc.len();

        // Build hvc1 box
        let mut hvc1 = BytesMut::with_capacity(256);
        let hvc1_size = 8 + 78 + hvcc_size;
        hvc1.put_u32(hvc1_size as u32);
        hvc1.put_slice(b"hvc1");

        // Same visual sample entry structure as avc1
        for _ in 0..6 {
            hvc1.put_u8(0);
        }
        hvc1.put_u16(1);
        hvc1.put_u16(0);
        hvc1.put_u16(0);
        for _ in 0..3 {
            hvc1.put_u32(0);
        }
        hvc1.put_u16(self.width as u16);
        hvc1.put_u16(self.height as u16);
        hvc1.put_u32(0x00480000);
        hvc1.put_u32(0x00480000);
        hvc1.put_u32(0);
        hvc1.put_u16(1);
        for _ in 0..32 {
            hvc1.put_u8(0);
        }
        hvc1.put_u16(0x0018);
        hvc1.put_i16(-1);

        // hvcC box
        hvc1.put_u32(hvcc_size as u32);
        hvc1.put_slice(b"hvcC");
        hvc1.put_slice(&hvcc);

        hvc1.to_vec()
    }

    /// Build mvex box
    fn build_mvex(&self) -> Vec<u8> {
        let mut mvex = BoxBuilder::new();

        // trex (track extends)
        let mut trex = BytesMut::with_capacity(20);
        trex.put_u32(1); // track_ID
        trex.put_u32(1); // default_sample_description_index
        trex.put_u32(0); // default_sample_duration
        trex.put_u32(0); // default_sample_size
        trex.put_u32(0); // default_sample_flags

        mvex.write_full_box(b"trex", 0, 0, &trex);

        // trex for the audio track
        if self.init_has_audio {
            let mut trex = BytesMut::with_capacity(20);
            trex.put_u32(2); // track_ID
            trex.put_u32(1); // default_sample_description_index
            trex.put_u32(0); // default_sample_duration
            trex.put_u32(0); // default_sample_size
            trex.put_u32(0); // default_sample_flags
            mvex.write_full_box(b"trex", 0, 0, &trex);
        }

        mvex.into_bytes()
    }

    /// Finalize current segment and return it.
    /// `next_start` is the decode timestamp of the next segment's first sample
    /// (the keyframe that triggered this finalization). It's used to compute the
    /// last sample's duration exactly, preventing timestamp overlap between segments.
    fn finalize_segment(&mut self, next_start: u64) -> Option<FMP4Segment> {
        if self.current_segment_samples.is_empty() {
            return None;
        }

        let start_time = self.segment_start_time?;
        let has_keyframe = self.current_segment_samples.iter().any(|s| s.is_keyframe);

        // Segment duration spans from this segment's start to the next segment's
        // first sample (`next_start`) — the same boundary the trun uses to set the
        // last sample's duration. Using the last sample's *own* timestamp instead
        // would drop one frame interval per segment, accumulating drift (a 600s
        // clip reports ~585s) and skewing EXTINF/seek timelines.
        let duration_ticks = next_start.saturating_sub(start_time);
        let duration = Duration::from_micros(duration_ticks * 1_000_000 / self.timescale as u64);

        // Audio rides along only when the init segment described track 2.
        let n_audio = if self.init_has_audio {
            self.current_segment_audio.len()
        } else {
            0
        };

        // Calculate data offsets (relative to moof start, per
        // default-base-is-moof):
        //   video traf = 8 (traf) + 16 (tfhd) + 20 (tfdt)
        //              + 12 (trun hdr) + 8 + Nv*12   = 64 + Nv*12
        //   audio traf = 8 (traf) + 16 (tfhd) + 20 (tfdt)
        //              + 12 (trun hdr) + 8 + Na*8    = 64 + Na*8
        //   moof = 8 (header) + 16 (mfhd) + trafs
        // Video samples are written to mdat first, then audio samples.
        let n = self.current_segment_samples.len();
        let video_traf_size = 64 + n * 12;
        let audio_traf_size = if n_audio > 0 { 64 + n_audio * 8 } else { 0 };
        let moof_size = 24 + video_traf_size + audio_traf_size;

        let video_mdat_len: usize = self
            .current_segment_samples
            .iter()
            .flat_map(|s| s.nals.iter())
            .map(|nal| 4 + nal.len() - self.nal_start_code_len(nal))
            .sum();

        let video_data_offset = (moof_size + 8) as u32;
        let audio_data_offset = (moof_size + 8 + video_mdat_len) as u32;

        // Build moof + mdat
        let mut builder = BoxBuilder::new();

        let moof = self.build_moof(
            start_time,
            video_data_offset,
            next_start,
            (n_audio > 0).then_some(audio_data_offset),
        );
        builder.write_box(b"moof", &moof);

        let mdat = self.build_mdat_data(n_audio > 0);
        builder.write_box(b"mdat", &mdat);

        let segment = FMP4Segment {
            sequence: self.sequence,
            data: builder.into_bytes(),
            duration,
            timestamp: start_time,
            is_keyframe: has_keyframe,
        };

        self.sequence += 1;
        self.current_segment_samples.clear();
        self.current_segment_audio.clear();
        self.segment_start_time = None;

        Some(segment)
    }

    /// Build moof box
    fn build_moof(
        &self,
        base_time: u64,
        data_offset: u32,
        next_start: u64,
        audio_data_offset: Option<u32>,
    ) -> Vec<u8> {
        let mut moof = BoxBuilder::new();

        // mfhd
        let mut mfhd = BytesMut::with_capacity(4);
        mfhd.put_u32(self.sequence as u32 + 1);
        moof.write_full_box(b"mfhd", 0, 0, &mfhd);

        // traf
        let traf = self.build_traf(base_time, data_offset, next_start);
        moof.write_box(b"traf", &traf);

        // audio traf
        if let Some(audio_offset) = audio_data_offset {
            let traf = self.build_traf_audio(audio_offset);
            moof.write_box(b"traf", &traf);
        }

        moof.into_bytes()
    }

    /// Build traf box for the audio track.
    ///
    /// Audio sample durations are a constant 1024 ticks: the track timescale
    /// is the sample rate, and an AAC frame is 1024 samples. Each fragment's
    /// tfdt re-anchors the track, so rounding never accumulates.
    fn build_traf_audio(&self, data_offset: u32) -> Vec<u8> {
        let mut traf = BoxBuilder::new();

        // tfhd — default-base-is-moof, track 2
        let mut tfhd = BytesMut::with_capacity(4);
        tfhd.put_u32(2); // track_ID
        traf.write_full_box(b"tfhd", 0, 0x020000, &tfhd);

        // tfdt — first audio sample's decode time
        let base_time = self
            .current_segment_audio
            .first()
            .map(|s| s.timestamp)
            .unwrap_or(0);
        let mut tfdt = BytesMut::with_capacity(8);
        tfdt.put_u64(base_time);
        traf.write_full_box(b"tfdt", 1, 0, &tfdt);

        // trun — flags 0x000301: data-offset + sample-duration + sample-size
        let sample_count = self.current_segment_audio.len();
        let mut trun = BytesMut::with_capacity(16 + sample_count * 8);
        trun.put_u32(sample_count as u32);
        trun.put_u32(data_offset);
        for sample in &self.current_segment_audio {
            trun.put_u32(1024); // sample_duration (AAC frame, timescale = rate)
            trun.put_u32(sample.data.len() as u32); // sample_size
        }
        traf.write_full_box(b"trun", 0, 0x000301, &trun);

        traf.into_bytes()
    }

    /// Build traf box
    fn build_traf(&self, base_time: u64, data_offset: u32, next_start: u64) -> Vec<u8> {
        let mut traf = BoxBuilder::new();

        // tfhd
        let mut tfhd = BytesMut::with_capacity(4);
        tfhd.put_u32(1); // track_ID
        traf.write_full_box(b"tfhd", 0, 0x020000, &tfhd); // default-base-is-moof

        // tfdt
        let mut tfdt = BytesMut::with_capacity(8);
        tfdt.put_u64(base_time);
        traf.write_full_box(b"tfdt", 1, 0, &tfdt);

        // trun — flags 0x000701: data-offset + sample-duration + sample-size + sample-flags
        let trun = self.build_trun(data_offset, next_start);
        traf.write_full_box(b"trun", 0, 0x000701, &trun);

        traf.into_bytes()
    }

    /// Build trun box data.
    /// `next_start` is the decode timestamp of the next segment's first sample,
    /// used to compute the last sample's duration for perfect segment contiguity.
    fn build_trun(&self, data_offset: u32, next_start: u64) -> Vec<u8> {
        let sample_count = self.current_segment_samples.len();
        let mut trun = BytesMut::with_capacity(256);
        trun.put_u32(sample_count as u32); // sample_count
        trun.put_u32(data_offset); // data_offset (relative to moof start)

        // Compute per-sample durations using forward-looking gaps: each sample's
        // duration = gap to the next sample's DTS. The last sample's duration is
        // computed from the next segment's start timestamp for perfect contiguity.
        let mut durations: Vec<u64> = Vec::with_capacity(sample_count);
        for i in 0..sample_count {
            let next_ts = if i + 1 < sample_count {
                self.current_segment_samples[i + 1].timestamp
            } else {
                // Last sample: use next segment's start time for exact contiguity
                next_start
            };
            let dur = next_ts
                .saturating_sub(self.current_segment_samples[i].timestamp)
                .max(1);
            durations.push(dur);
        }

        // Per-sample entries: duration + size + flags
        for (i, sample) in self.current_segment_samples.iter().enumerate() {
            // Sample size = sum of all length-prefixed NALs in the access unit
            let sample_size: usize = sample
                .nals
                .iter()
                .map(|nal| 4 + nal.len() - self.nal_start_code_len(nal))
                .sum();

            trun.put_u32(durations[i] as u32); // sample_duration
            trun.put_u32(sample_size as u32); // sample_size
            if sample.is_keyframe {
                trun.put_u32(0x02000000); // sample_flags (sync)
            } else {
                trun.put_u32(0x01010000); // sample_flags (non-sync)
            }
        }

        trun.to_vec()
    }

    /// Get NAL start code length
    fn nal_start_code_len(&self, nal_data: &[u8]) -> usize {
        if nal_data.starts_with(&[0x00, 0x00, 0x00, 0x01]) {
            4
        } else if nal_data.starts_with(&[0x00, 0x00, 0x01]) {
            3
        } else {
            0
        }
    }

    /// Check if a NAL unit contains video coding layer (slice) data.
    ///
    /// Only VCL NALs should be placed into fMP4 media segment mdat boxes.
    /// Non-VCL NALs (SPS, PPS, AUD, SEI, filler, etc.) must be excluded.
    fn is_vcl_nal(&self, nal_data: &[u8]) -> bool {
        let offset = if nal_data.starts_with(&[0x00, 0x00, 0x00, 0x01]) {
            4
        } else if nal_data.starts_with(&[0x00, 0x00, 0x01]) {
            3
        } else {
            return false;
        };

        if offset >= nal_data.len() {
            return false;
        }

        let nal_type_byte = nal_data[offset];

        match self.codec {
            VideoCodec::H264 => {
                // H.264 VCL NAL types 1-5:
                //   1 = non-IDR slice, 2-4 = data partitions, 5 = IDR slice
                let nal_type = nal_type_byte & 0x1F;
                (1..=5).contains(&nal_type)
            }
            VideoCodec::H265 => {
                // H.265 VCL NAL types 0-31 (all slice segment types)
                let nal_type = (nal_type_byte >> 1) & 0x3F;
                nal_type <= 31
            }
            VideoCodec::Unknown => false,
        }
    }

    /// Build mdat box inner data: length-prefixed video NAL units, followed
    /// by raw AAC frames when `include_audio` is set (matching the audio
    /// traf's data offset).
    fn build_mdat_data(&self, include_audio: bool) -> Vec<u8> {
        let mut mdat = BytesMut::with_capacity(65536);

        for sample in &self.current_segment_samples {
            for nal_data in &sample.nals {
                let start_code_len = self.nal_start_code_len(nal_data);
                let nal_content = &nal_data[start_code_len..];

                // Write length-prefixed NAL unit (Annex B → AVCC)
                mdat.put_u32(nal_content.len() as u32);
                mdat.put_slice(nal_content);
            }
        }

        if include_audio {
            for sample in &self.current_segment_audio {
                mdat.put_slice(&sample.data);
            }
        }

        mdat.to_vec()
    }

    /// Get the stored SPS data (without start code)
    pub fn sps(&self) -> Option<&[u8]> {
        self.sps.as_deref()
    }

    /// Get current sequence number
    pub fn sequence(&self) -> u64 {
        self.sequence
    }

    /// Get monitor ID
    pub fn monitor_id(&self) -> u32 {
        self.monitor_id
    }

    /// Total pending samples: completed samples plus the pending frame (if any).
    /// Used in tests to verify how many access units have been accumulated.
    #[cfg(test)]
    fn total_pending_samples(&self) -> usize {
        self.current_segment_samples.len() + if self.pending_frame.is_some() { 1 } else { 0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_segmenter_creation() {
        let segmenter = HlsSegmenter::new(1, Duration::from_secs(4));
        assert_eq!(segmenter.monitor_id(), 1);
        assert_eq!(segmenter.sequence(), 0);
    }

    #[test]
    fn test_box_builder() {
        let mut builder = BoxBuilder::new();
        builder.write_box(b"test", b"data");
        let bytes = builder.into_bytes();

        assert_eq!(bytes.len(), 12); // 4 (size) + 4 (type) + 4 (data)
        assert_eq!(&bytes[0..4], &[0, 0, 0, 12]); // size = 12
        assert_eq!(&bytes[4..8], b"test");
        assert_eq!(&bytes[8..12], b"data");
    }

    #[test]
    fn test_ftyp_generation() {
        let segmenter = HlsSegmenter::new(1, Duration::from_secs(4));
        let ftyp = segmenter.build_ftyp();

        assert!(ftyp.starts_with(b"isom"));
    }

    #[test]
    fn test_extract_h264_sps() {
        let mut segmenter = HlsSegmenter::new(1, Duration::from_secs(4));
        segmenter.set_codec(VideoCodec::H264);

        // H.264 SPS NAL unit
        let sps = vec![0x00, 0x00, 0x00, 0x01, 0x67, 0x42, 0x00, 0x1E, 0xAB, 0x40];
        segmenter.extract_parameter_sets(&sps);

        assert!(segmenter.sps.is_some());
        assert_eq!(segmenter.sps.as_ref().unwrap()[0], 0x67);
    }

    #[test]
    fn test_extract_h264_pps() {
        let mut segmenter = HlsSegmenter::new(1, Duration::from_secs(4));
        segmenter.set_codec(VideoCodec::H264);

        // H.264 PPS NAL unit
        let pps = vec![0x00, 0x00, 0x00, 0x01, 0x68, 0xCE, 0x3C, 0x80];
        segmenter.extract_parameter_sets(&pps);

        assert!(segmenter.pps.is_some());
    }

    #[test]
    fn test_segment_data_offset_and_trun_flags() {
        let mut segmenter = HlsSegmenter::new(1, Duration::from_secs(2));
        segmenter.set_codec(VideoCodec::H264);

        // Feed SPS + PPS
        let sps = vec![0x00, 0x00, 0x00, 0x01, 0x67, 0x42, 0x00, 0x1E, 0xAB, 0x40];
        let pps = vec![0x00, 0x00, 0x00, 0x01, 0x68, 0xCE, 0x3C, 0x80];
        segmenter.process_nal(&sps, 0, false);
        segmenter.process_nal(&pps, 1000, false);

        // Feed a keyframe to start the segment
        let idr = vec![0x00, 0x00, 0x00, 0x01, 0x65, 0x88, 0x80, 0x40];
        segmenter.process_nal(&idr, 100_000, true);

        // Feed some P-frames over target duration
        for i in 1..=10 {
            let p_frame = vec![0x00, 0x00, 0x00, 0x01, 0x41, 0x9A, i];
            segmenter.process_nal(&p_frame, 100_000 + i as u64 * 300_000, false);
        }

        // Feed another keyframe to finalize the segment
        let idr2 = vec![0x00, 0x00, 0x00, 0x01, 0x65, 0x88, 0x80, 0x41];
        let segment = segmenter.process_nal(&idr2, 100_000 + 11 * 300_000, true);
        assert!(segment.is_some(), "Segment should be produced");

        let seg = segment.unwrap();
        let data = &seg.data;

        // Parse moof box: first 4 bytes = size, next 4 = "moof"
        let moof_size = u32::from_be_bytes([data[0], data[1], data[2], data[3]]) as usize;
        assert_eq!(&data[4..8], b"moof");

        // mdat box header follows moof
        let mdat_offset = moof_size;
        assert_eq!(&data[mdat_offset + 4..mdat_offset + 8], b"mdat");

        // Verify data_offset in trun points to mdat data (moof_size + 8)
        // Position of data_offset in segment: 8 (moof header) + 16 (mfhd) + 8 (traf header)
        //   + 16 (tfhd) + 20 (tfdt) + 12 (trun header) + 4 (sample_count) = 84
        let data_offset_pos = 84;
        let data_offset = u32::from_be_bytes([
            data[data_offset_pos],
            data[data_offset_pos + 1],
            data[data_offset_pos + 2],
            data[data_offset_pos + 3],
        ]);
        assert_eq!(
            data_offset as usize,
            moof_size + 8,
            "data_offset should point past moof and mdat header"
        );

        // Verify trun flags are 0x000701
        // trun fullbox is at: 8 + 16 + 8 + 16 + 20 = 68 (start of trun box)
        // version+flags at offset 76 (68 + 8 for box header)
        let trun_vf_pos = 76;
        let trun_flags = u32::from_be_bytes([
            data[trun_vf_pos],
            data[trun_vf_pos + 1],
            data[trun_vf_pos + 2],
            data[trun_vf_pos + 3],
        ]);
        assert_eq!(
            trun_flags & 0x00FFFFFF,
            0x000701,
            "trun flags should be 0x000701"
        );
    }

    #[test]
    fn test_segment_duration_spans_to_next_segment_start() {
        // Regression guard: a finalized segment's reported duration must span
        // from its start to the *next* segment's first sample (`next_start`),
        // not stop at the last sample's timestamp. Stopping at the last sample
        // drops one inter-frame interval per segment and accumulates drift
        // (a 600s clip mis-reports as ~585s in EXTINF / seek timelines).
        let mut segmenter = HlsSegmenter::new(1, Duration::from_secs(2));
        segmenter.set_codec(VideoCodec::H264);

        let sps = vec![0x00, 0x00, 0x00, 0x01, 0x67, 0x42, 0x00, 0x1E, 0xAB, 0x40];
        let pps = vec![0x00, 0x00, 0x00, 0x01, 0x68, 0xCE, 0x3C, 0x80];
        segmenter.process_nal(&sps, 0, false);
        segmenter.process_nal(&pps, 1000, false);

        // IDR is the first VCL keyframe -> it becomes the timestamp origin
        // (normalized to 0). Frames are a uniform 300ms apart.
        let idr = vec![0x00, 0x00, 0x00, 0x01, 0x65, 0x88, 0x80, 0x40];
        segmenter.process_nal(&idr, 100_000, true);
        for i in 1..=10 {
            let p_frame = vec![0x00, 0x00, 0x00, 0x01, 0x41, 0x9A, i];
            segmenter.process_nal(&p_frame, 100_000 + i as u64 * 300_000, false);
        }

        // Next keyframe finalizes the segment. Last sample sits at normalized
        // 3_000_000us; the boundary keyframe at 3_300_000us. Duration must be
        // 3.3s (start..next_start), NOT 3.0s (start..last_sample).
        let idr2 = vec![0x00, 0x00, 0x00, 0x01, 0x65, 0x88, 0x80, 0x41];
        let seg = segmenter
            .process_nal(&idr2, 100_000 + 11 * 300_000, true)
            .expect("segment should finalize on the second keyframe");

        assert_eq!(
            seg.duration,
            Duration::from_micros(3_300_000),
            "duration must span start..next_start (3.3s)"
        );
        assert_ne!(
            seg.duration,
            Duration::from_micros(3_000_000),
            "duration must NOT stop at the last sample's timestamp (the old bug)"
        );
    }

    #[test]
    fn test_flush_emits_trailing_segment_for_short_clip() {
        // A clip shorter than one target duration produces no segment until EOF;
        // flush() must emit the single trailing segment (VOD finalizes the tail,
        // unlike live which waits for the next keyframe that never comes).
        let mut segmenter = HlsSegmenter::new(1, Duration::from_secs(4));
        segmenter.set_codec(VideoCodec::H264);

        let sps = vec![0x00, 0x00, 0x00, 0x01, 0x67, 0x42, 0x00, 0x1E, 0xAB, 0x40];
        let pps = vec![0x00, 0x00, 0x00, 0x01, 0x68, 0xCE, 0x3C, 0x80];
        segmenter.process_nal(&sps, 0, false);
        segmenter.process_nal(&pps, 1000, false);

        let idr = vec![0x00, 0x00, 0x00, 0x01, 0x65, 0x88, 0x80, 0x40];
        assert!(
            segmenter.process_nal(&idr, 100_000, true).is_none(),
            "first keyframe alone must not finalize a segment"
        );
        for i in 1..=5 {
            let p_frame = vec![0x00, 0x00, 0x00, 0x01, 0x41, 0x9A, i];
            assert!(
                segmenter
                    .process_nal(&p_frame, 100_000 + i as u64 * 200_000, false)
                    .is_none(),
                "short clip under target duration must not finalize mid-stream"
            );
        }

        let tail = segmenter
            .flush()
            .expect("flush must emit the trailing segment");
        assert!(tail.is_keyframe, "the trailing segment opens on the IDR");
        assert!(
            tail.duration > Duration::ZERO,
            "flushed segment must have a non-degenerate duration"
        );
        assert!(
            segmenter.flush().is_none(),
            "a second flush has nothing left to emit"
        );
    }

    #[test]
    fn test_non_vcl_nals_excluded_from_media_segments() {
        let mut segmenter = HlsSegmenter::new(1, Duration::from_secs(2));
        segmenter.set_codec(VideoCodec::H264);

        // Feed SPS + PPS (should be extracted but NOT added to segment data)
        let sps = vec![0x00, 0x00, 0x00, 0x01, 0x67, 0x42, 0x00, 0x1E, 0xAB, 0x40];
        let pps = vec![0x00, 0x00, 0x00, 0x01, 0x68, 0xCE, 0x3C, 0x80];
        assert!(segmenter.process_nal(&sps, 0, false).is_none());
        assert!(segmenter.process_nal(&pps, 1000, false).is_none());

        // Parameter sets should be stored for init segment
        assert!(segmenter.sps.is_some());
        assert!(segmenter.pps.is_some());

        // But segment data should be empty (no samples accumulated)
        assert_eq!(
            segmenter.total_pending_samples(),
            0,
            "SPS/PPS should not be in segment data"
        );

        // Feed AUD (type 9) — should be filtered as non-VCL
        let aud = vec![0x00, 0x00, 0x00, 0x01, 0x09, 0xF0];
        assert!(segmenter.process_nal(&aud, 90_000, false).is_none());
        assert_eq!(
            segmenter.total_pending_samples(),
            0,
            "AUD should not be in segment data"
        );

        // Feed SEI (type 6) — should be filtered as non-VCL
        let sei = vec![0x00, 0x00, 0x00, 0x01, 0x06, 0x05, 0x04, 0x00];
        assert!(segmenter.process_nal(&sei, 95_000, false).is_none());
        assert_eq!(
            segmenter.total_pending_samples(),
            0,
            "SEI should not be in segment data"
        );

        // Feed IDR + P-frames to build a segment (each at different timestamp = 1 sample each)
        let idr = vec![0x00, 0x00, 0x00, 0x01, 0x65, 0x88, 0x80, 0x40];
        segmenter.process_nal(&idr, 100_000, true);
        for i in 1..=10 {
            let p_frame = vec![0x00, 0x00, 0x00, 0x01, 0x41, 0x9A, i];
            segmenter.process_nal(&p_frame, 100_000 + i as u64 * 300_000, false);
        }

        // Inline SPS/PPS again (cameras resend them periodically)
        segmenter.process_nal(&sps, 100_000 + 5 * 300_000, false);
        segmenter.process_nal(&pps, 100_000 + 5 * 300_000, false);

        // Should have 11 samples (IDR + 10 P-frames): 10 completed + 1 pending
        assert_eq!(
            segmenter.total_pending_samples(),
            11,
            "Only VCL NALs should be in segment data"
        );
    }

    #[test]
    fn test_data_dropped_before_first_keyframe() {
        let mut segmenter = HlsSegmenter::new(1, Duration::from_secs(2));
        segmenter.set_codec(VideoCodec::H264);

        // Feed SPS + PPS
        let sps = vec![0x00, 0x00, 0x00, 0x01, 0x67, 0x42, 0x00, 0x1E, 0xAB, 0x40];
        let pps = vec![0x00, 0x00, 0x00, 0x01, 0x68, 0xCE, 0x3C, 0x80];
        segmenter.process_nal(&sps, 0, false);
        segmenter.process_nal(&pps, 1000, false);

        // Feed P-frames before any keyframe — should be dropped
        for i in 0..5 {
            let p_frame = vec![0x00, 0x00, 0x00, 0x01, 0x41, 0x9A, i];
            segmenter.process_nal(&p_frame, 10_000 + i as u64 * 33_000, false);
        }
        assert_eq!(
            segmenter.total_pending_samples(),
            0,
            "P-frames before first keyframe should be dropped"
        );
        assert!(!segmenter.received_keyframe);

        // Feed first keyframe — should be accepted
        let idr = vec![0x00, 0x00, 0x00, 0x01, 0x65, 0x88, 0x80, 0x40];
        segmenter.process_nal(&idr, 200_000, true);
        assert_eq!(segmenter.total_pending_samples(), 1);
        assert!(segmenter.received_keyframe);

        // Subsequent P-frames should now be accepted
        let p_frame = vec![0x00, 0x00, 0x00, 0x01, 0x41, 0x9A, 0x01];
        segmenter.process_nal(&p_frame, 233_000, false);
        assert_eq!(segmenter.total_pending_samples(), 2);
    }

    #[test]
    fn test_init_segment_requires_sps_pps() {
        let mut segmenter = HlsSegmenter::new(1, Duration::from_secs(4));
        segmenter.set_codec(VideoCodec::H264);

        // No SPS/PPS yet
        assert!(segmenter.generate_init_segment().is_none());

        // Add SPS
        let sps = vec![0x00, 0x00, 0x00, 0x01, 0x67, 0x42, 0x00, 0x1E];
        segmenter.extract_parameter_sets(&sps);
        assert!(segmenter.generate_init_segment().is_none());

        // Add PPS
        let pps = vec![0x00, 0x00, 0x00, 0x01, 0x68, 0xCE, 0x3C, 0x80];
        segmenter.extract_parameter_sets(&pps);

        // Now should work
        let init = segmenter.generate_init_segment();
        assert!(init.is_some());
        assert!(!init.unwrap().data.is_empty());
    }

    /// Real ffmpeg-generated SPS for 3840x2160 (High profile)
    fn real_sps_4k() -> Vec<u8> {
        vec![
            0x67, 0x64, 0x00, 0x33, 0xAC, 0xD9, 0x40, 0x3C, 0x00, 0x43, 0xEC, 0x04, 0x40, 0x00,
            0x00, 0x03, 0x00, 0x40, 0x00, 0x00, 0x0C, 0x83, 0xC6, 0x0C, 0x65, 0x80,
        ]
    }

    #[test]
    fn test_init_segment_dimensions_from_sps() {
        let mut segmenter = HlsSegmenter::new(1, Duration::from_secs(4));
        segmenter.set_codec(VideoCodec::H264);

        // Feed a real 4K SPS with start code
        let mut sps_with_sc = vec![0x00, 0x00, 0x00, 0x01];
        sps_with_sc.extend_from_slice(&real_sps_4k());
        segmenter.extract_parameter_sets(&sps_with_sc);

        // Verify SPS parsing updated dimensions
        assert_eq!(segmenter.width, 3840);
        assert_eq!(segmenter.height, 2160);

        // Feed PPS
        let pps = vec![0x00, 0x00, 0x00, 0x01, 0x68, 0xEB, 0xE3, 0xCB, 0x22];
        segmenter.extract_parameter_sets(&pps);

        let init = segmenter.generate_init_segment().unwrap();
        assert_eq!(init.width, 3840);
        assert_eq!(init.height, 2160);

        // Verify tkhd box has correct dimensions in binary (16.16 fixed-point)
        let data = &init.data;
        let tkhd_dims = find_tkhd_dimensions(data);
        assert_eq!(tkhd_dims, Some((3840, 2160)));

        // Verify avc1 box has correct dimensions
        let avc1_dims = find_avc1_dimensions(data);
        assert_eq!(avc1_dims, Some((3840, 2160)));
    }

    #[test]
    fn test_sps_dimension_change_invalidates_init() {
        let mut segmenter = HlsSegmenter::new(1, Duration::from_secs(4));
        segmenter.set_codec(VideoCodec::H264);

        // Feed 1080p SPS (real ffmpeg-generated)
        let sps_1080p = vec![
            0x00, 0x00, 0x00, 0x01, 0x67, 0x64, 0x00, 0x28, 0xAC, 0xD9, 0x40, 0x78, 0x02, 0x27,
            0xE5, 0xC0, 0x44, 0x00, 0x00, 0x03, 0x00, 0x04, 0x00, 0x00, 0x03, 0x00, 0xC8, 0x3C,
            0x60, 0xC6, 0x58,
        ];
        let pps = vec![0x00, 0x00, 0x00, 0x01, 0x68, 0xEB, 0xE3, 0xCB, 0x22];
        segmenter.extract_parameter_sets(&sps_1080p);
        segmenter.extract_parameter_sets(&pps);
        assert_eq!(segmenter.width, 1920);
        assert_eq!(segmenter.height, 1080);

        // Generate init segment
        let init1 = segmenter.generate_init_segment();
        assert!(init1.is_some());
        assert!(segmenter.get_init_segment().is_some());

        // Feed 4K SPS — should invalidate cached init
        let mut sps_4k_with_sc = vec![0x00, 0x00, 0x00, 0x01];
        sps_4k_with_sc.extend_from_slice(&real_sps_4k());
        segmenter.extract_parameter_sets(&sps_4k_with_sc);
        assert_eq!(segmenter.width, 3840);
        assert_eq!(segmenter.height, 2160);
        assert!(
            segmenter.get_init_segment().is_none(),
            "init_segment should be invalidated on dimension change"
        );
    }

    #[test]
    fn test_multi_slice_frame_single_sample() {
        let mut segmenter = HlsSegmenter::new(1, Duration::from_secs(4));
        segmenter.set_codec(VideoCodec::H264);

        // Feed SPS + PPS
        let sps = vec![0x00, 0x00, 0x00, 0x01, 0x67, 0x42, 0x00, 0x1E, 0xAB, 0x40];
        let pps = vec![0x00, 0x00, 0x00, 0x01, 0x68, 0xCE, 0x3C, 0x80];
        segmenter.process_nal(&sps, 0, false);
        segmenter.process_nal(&pps, 1000, false);

        // Feed two IDR slices of one multi-slice frame. The first slice starts
        // the picture (`first_mb_in_slice == 0` → slice-header byte MSB set);
        // the second is a continuation slice (`first_mb_in_slice > 0` → MSB
        // clear). They share a timestamp here, but grouping no longer depends
        // on that — see `test_4k_multi_slice_grouped_regardless_of_timestamp`.
        let idr_slice1 = vec![0x00, 0x00, 0x00, 0x01, 0x65, 0x88, 0x80, 0x40];
        let idr_slice2 = vec![0x00, 0x00, 0x00, 0x01, 0x65, 0x08, 0x80, 0x41];
        segmenter.process_nal(&idr_slice1, 100_000, true);
        segmenter.process_nal(&idr_slice2, 100_000, true);

        // Both slices should be in the same pending frame (1 sample total)
        assert_eq!(
            segmenter.total_pending_samples(),
            1,
            "Two slices with same timestamp should be one sample"
        );
        assert_eq!(
            segmenter.pending_frame.as_ref().unwrap().nals.len(),
            2,
            "Pending frame should contain 2 NALs"
        );

        // Feed a P-frame at a new timestamp — flushes the pending IDR frame
        let p_frame = vec![0x00, 0x00, 0x00, 0x01, 0x41, 0x9A, 0x01];
        segmenter.process_nal(&p_frame, 133_333, false);

        // Now we should have 2 samples: 1 completed IDR (2 NALs) + 1 pending P-frame
        assert_eq!(segmenter.total_pending_samples(), 2);
        assert_eq!(
            segmenter.current_segment_samples[0].nals.len(),
            2,
            "Completed IDR sample should contain 2 NALs"
        );
        assert!(segmenter.current_segment_samples[0].is_keyframe);
    }

    #[test]
    fn test_multi_slice_mdat_format() {
        let mut segmenter = HlsSegmenter::new(1, Duration::from_secs(2));
        segmenter.set_codec(VideoCodec::H264);

        // Feed SPS + PPS
        let sps = vec![0x00, 0x00, 0x00, 0x01, 0x67, 0x42, 0x00, 0x1E, 0xAB, 0x40];
        let pps = vec![0x00, 0x00, 0x00, 0x01, 0x68, 0xCE, 0x3C, 0x80];
        segmenter.process_nal(&sps, 0, false);
        segmenter.process_nal(&pps, 1000, false);

        // Feed a multi-slice keyframe: a picture-starting slice (0xAA → MSB
        // set) followed by a continuation slice (0x0C → MSB clear).
        let idr1 = vec![0x00, 0x00, 0x00, 0x01, 0x65, 0xAA, 0xBB];
        let idr2 = vec![0x00, 0x00, 0x00, 0x01, 0x65, 0x0C, 0xDD];
        segmenter.process_nal(&idr1, 100_000, true);
        segmenter.process_nal(&idr2, 100_000, true);

        // Feed P-frames to go over target duration
        for i in 1..=10 {
            let p = vec![0x00, 0x00, 0x00, 0x01, 0x41, 0x9A, i];
            segmenter.process_nal(&p, 100_000 + i as u64 * 300_000, false);
        }

        // Feed another keyframe to finalize
        let idr3 = vec![0x00, 0x00, 0x00, 0x01, 0x65, 0xEE, 0xFF];
        let segment = segmenter.process_nal(&idr3, 100_000 + 11 * 300_000, true);
        assert!(segment.is_some(), "Segment should be produced");

        let seg = segment.unwrap();
        let data = &seg.data;

        // Find mdat box
        let moof_size = u32::from_be_bytes([data[0], data[1], data[2], data[3]]) as usize;
        let mdat_start = moof_size;
        let mdat_size = u32::from_be_bytes([
            data[mdat_start],
            data[mdat_start + 1],
            data[mdat_start + 2],
            data[mdat_start + 3],
        ]) as usize;
        assert_eq!(&data[mdat_start + 4..mdat_start + 8], b"mdat");

        // First sample should be the multi-slice IDR: two length-prefixed NALs
        let mdat_data_start = mdat_start + 8;
        // NAL 1: 3 bytes (0x65, 0xAA, 0xBB) → length prefix = 0x00000003
        let nal1_len = u32::from_be_bytes([
            data[mdat_data_start],
            data[mdat_data_start + 1],
            data[mdat_data_start + 2],
            data[mdat_data_start + 3],
        ]);
        assert_eq!(nal1_len, 3, "First NAL should be 3 bytes");
        assert_eq!(data[mdat_data_start + 4], 0x65); // IDR NAL type byte
        assert_eq!(data[mdat_data_start + 5], 0xAA);
        assert_eq!(data[mdat_data_start + 6], 0xBB);

        // NAL 2: 3 bytes (0x65, 0xCC, 0xDD)
        let nal2_start = mdat_data_start + 4 + 3;
        let nal2_len = u32::from_be_bytes([
            data[nal2_start],
            data[nal2_start + 1],
            data[nal2_start + 2],
            data[nal2_start + 3],
        ]);
        assert_eq!(nal2_len, 3, "Second NAL should be 3 bytes");
        assert_eq!(data[nal2_start + 4], 0x65);
        assert_eq!(data[nal2_start + 5], 0x0C);
        assert_eq!(data[nal2_start + 6], 0xDD);

        // Verify trun sample_count: should be 11 (1 IDR + 10 P-frames), not 12
        // sample_count is at offset 80 (8+16+8+16+20+12 = trun header, then 4 bytes)
        let sample_count_pos = 80;
        let sample_count = u32::from_be_bytes([
            data[sample_count_pos],
            data[sample_count_pos + 1],
            data[sample_count_pos + 2],
            data[sample_count_pos + 3],
        ]);
        assert_eq!(sample_count, 11, "Should have 11 samples (1 IDR + 10 P)");

        // Verify first sample size includes both NALs: (4+3) + (4+3) = 14
        // First sample entry starts at offset 88 (80 + 4 sample_count + 4 data_offset)
        let first_sample_pos = 88;
        let first_sample_size = u32::from_be_bytes([
            data[first_sample_pos + 4], // skip duration (4 bytes)
            data[first_sample_pos + 5],
            data[first_sample_pos + 6],
            data[first_sample_pos + 7],
        ]);
        assert_eq!(
            first_sample_size, 14,
            "First sample should include both NALs: (4+3)+(4+3)=14"
        );

        // Verify mdat total size is consistent
        let expected_mdat_data: usize = 14 + 10 * 7; // IDR(14) + 10 P-frames(4+3 each)
        assert_eq!(mdat_size, 8 + expected_mdat_data);
    }

    /// Build an H.264 slice NAL whose slice header opens with the given
    /// `first_mb_in_slice`, Exp-Golomb (`ue(v)`) encoded exactly as a real
    /// encoder would. Used to exercise multi-slice 4K access units, whose
    /// later slices carry large macroblock indices.
    fn slice_nal(nal_type: u8, first_mb_in_slice: u32) -> Vec<u8> {
        // ue(v): `leading_zeros` zero bits, then the significant bits of
        // `first_mb_in_slice + 1` (which is at least 1, so always non-empty).
        let code = first_mb_in_slice as u64 + 1;
        let significant = 64 - code.leading_zeros();
        let leading_zeros = significant - 1;
        let mut bits: Vec<bool> = Vec::new();
        bits.extend(std::iter::repeat_n(false, leading_zeros as usize));
        for i in (0..significant).rev() {
            bits.push((code >> i) & 1 == 1);
        }
        // Pad to a byte boundary; padding never touches the first byte the
        // picture-start detector inspects.
        while !bits.len().is_multiple_of(8) {
            bits.push(false);
        }
        let mut nal = vec![0x00, 0x00, 0x00, 0x01, nal_type];
        for chunk in bits.chunks(8) {
            let mut byte = 0u8;
            for (i, &bit) in chunk.iter().enumerate() {
                if bit {
                    byte |= 1 << (7 - i);
                }
            }
            nal.push(byte);
        }
        nal
    }

    /// Regression test for the 4K HLS failure: a 4K picture has many slices,
    /// and they can reach the segmenter spread across ZoneMinder framed
    /// packets carrying *different* `pts` values. The segmenter must group
    /// them into one access unit by slice-start detection, never by timestamp.
    #[test]
    fn test_4k_multi_slice_grouped_regardless_of_timestamp() {
        let mut segmenter = HlsSegmenter::new(1, Duration::from_secs(2));
        segmenter.set_codec(VideoCodec::H264);

        // Real 4K SPS + PPS.
        let mut sps = vec![0x00, 0x00, 0x00, 0x01];
        sps.extend_from_slice(&real_sps_4k());
        let pps = vec![0x00, 0x00, 0x00, 0x01, 0x68, 0xEB, 0xE3, 0xCB, 0x22];
        segmenter.process_nal(&sps, 0, false);
        segmenter.process_nal(&pps, 0, false);

        // A 3840×2160 frame is 240×135 = 32400 macroblocks. Model a 24-slice
        // keyframe: slice 0 starts at macroblock 0, the rest at 1350-MB steps.
        const SLICES: u32 = 24;
        let first_slice = slice_nal(0x65, 0);
        segmenter.process_nal(&first_slice, 100_000, true);
        for s in 1..SLICES {
            let continuation = slice_nal(0x65, s * 1350);
            // Deliberately give each continuation slice a DIFFERENT timestamp:
            // the old timestamp-based grouping would tear the picture apart.
            segmenter.process_nal(&continuation, 100_000 + s as u64, true);
        }

        // All 24 slices belong to one in-progress access unit.
        assert_eq!(
            segmenter.total_pending_samples(),
            1,
            "a 24-slice 4K keyframe must assemble as ONE sample"
        );
        assert_eq!(
            segmenter.pending_frame.as_ref().unwrap().nals.len(),
            SLICES as usize,
            "the pending frame must hold every slice of the picture"
        );

        // The next picture's first slice flushes the keyframe as one sample.
        segmenter.process_nal(&slice_nal(0x41, 0), 400_000, false);
        assert_eq!(
            segmenter.current_segment_samples[0].nals.len(),
            SLICES as usize,
            "the completed keyframe sample must keep all 24 slices"
        );
        assert!(segmenter.current_segment_samples[0].is_keyframe);

        // Drive multi-slice P-frame pictures past the 2s target, then a second
        // keyframe — the segmenter must actually finalize a media segment.
        for p in 1..=10u64 {
            let ts = 400_000 + p * 300_000;
            segmenter.process_nal(&slice_nal(0x41, 0), ts, false);
            segmenter.process_nal(&slice_nal(0x41, 1350), ts + 1, false);
            segmenter.process_nal(&slice_nal(0x41, 2700), ts + 2, false);
        }
        // The second keyframe's *first* slice closes the segment (it begins a
        // new picture after the target duration has elapsed).
        let kf2_ts = 400_000 + 11 * 300_000;
        let segment = segmenter.process_nal(&slice_nal(0x65, 0), kf2_ts, true);
        segmenter.process_nal(&slice_nal(0x65, 1350), kf2_ts + 1, true);

        let seg = segment.expect("a segment must be produced for the 4K stream");
        assert!(seg.is_keyframe);
    }

    /// Helper: find tkhd box in init segment and extract width/height (16.16 fixed-point).
    fn find_tkhd_dimensions(data: &[u8]) -> Option<(u32, u32)> {
        find_box_data(data, b"tkhd").map(|tkhd| {
            // tkhd v0: width at byte 76, height at byte 80 (relative to box data start)
            // But since we have the data after version+flags (which is already stripped by
            // write_full_box), the layout is:
            // 0..4: creation_time, 4..8: modification_time, 8..12: track_ID,
            // 12..16: reserved, 16..20: duration, 20..28: reserved,
            // 28..30: layer, 30..32: alternate_group, 32..34: volume, 34..36: reserved,
            // 36..72: matrix (36 bytes), 72..76: width, 76..80: height
            let w = u32::from_be_bytes([tkhd[72], tkhd[73], tkhd[74], tkhd[75]]) >> 16;
            let h = u32::from_be_bytes([tkhd[76], tkhd[77], tkhd[78], tkhd[79]]) >> 16;
            (w, h)
        })
    }

    /// Helper: find avc1 box in init segment and extract width/height.
    fn find_avc1_dimensions(data: &[u8]) -> Option<(u32, u32)> {
        // Find avc1 box type in binary data
        for i in 0..data.len().saturating_sub(4) {
            if &data[i..i + 4] == b"avc1" {
                // avc1 box: after 8 bytes box header, 6 reserved, 2 data_ref_index,
                // 2+2 pre-defined, 3*4 pre-defined/reserved = 24 bytes,
                // then 2 bytes width, 2 bytes height
                // From "avc1" marker: +4 (type already consumed) but we're at the 'a' byte
                // Layout from box start: size(4) + type(4) + reserved(6) + data_ref_idx(2) +
                // pre-defined(16) = 28, then width(2) + height(2)
                let base = i - 4; // box start (size field)
                let w_offset = base + 8 + 6 + 2 + 16;
                let h_offset = w_offset + 2;
                if h_offset + 2 <= data.len() {
                    let w = u16::from_be_bytes([data[w_offset], data[w_offset + 1]]) as u32;
                    let h = u16::from_be_bytes([data[h_offset], data[h_offset + 1]]) as u32;
                    return Some((w, h));
                }
            }
        }
        None
    }

    /// Verify that two consecutive segments have non-overlapping timestamps.
    /// This is critical for fMP4 playback: if segment N's declared end time
    /// exceeds segment N+1's baseMediaDecodeTime, browsers reject the append
    /// with bufferAppendError.
    #[test]
    fn test_consecutive_segments_no_timestamp_overlap() {
        let mut segmenter = HlsSegmenter::new(1, Duration::from_secs(2));
        segmenter.set_codec(VideoCodec::H264);

        // Feed SPS + PPS
        let sps = vec![0x00, 0x00, 0x00, 0x01, 0x67, 0x42, 0x00, 0x1E, 0xAB, 0x40];
        let pps = vec![0x00, 0x00, 0x00, 0x01, 0x68, 0xCE, 0x3C, 0x80];
        segmenter.process_nal(&sps, 0, false);
        segmenter.process_nal(&pps, 1000, false);

        // Generate THREE keyframes separated by 2+ seconds of P-frames (target_dur=2s).
        // Variable frame intervals simulate real cameras (~30fps with jitter).
        let mut segments = Vec::new();
        let frame_intervals_us = [33333, 33334, 33333, 33400, 33200, 33333];
        let mut ts = 100_000u64;

        for seg_idx in 0..3 {
            // Keyframe starts each segment
            let idr = vec![0x00, 0x00, 0x00, 0x01, 0x65, 0x88, 0x80, seg_idx + 1];
            if let Some(seg) = segmenter.process_nal(&idr, ts, true) {
                segments.push(seg);
            }
            ts += frame_intervals_us[0];

            // ~70 P-frames = ~2.3 seconds at 30fps (exceeds 2s target)
            for i in 0..70 {
                let p = vec![0x00, 0x00, 0x00, 0x01, 0x41, 0x9A, (i & 0xFF) as u8];
                let interval = frame_intervals_us[(i + 1) % frame_intervals_us.len()];
                if let Some(seg) = segmenter.process_nal(&p, ts, false) {
                    segments.push(seg);
                }
                ts += interval;
            }
        }

        assert!(
            segments.len() >= 2,
            "Should produce at least 2 segments, got {}",
            segments.len()
        );

        // Parse each segment's baseMediaDecodeTime and total duration from the binary
        for i in 0..segments.len() - 1 {
            let seg_a = &segments[i].data;
            let seg_b = &segments[i + 1].data;

            let (bdt_a, total_dur_a) = parse_segment_timing(seg_a);
            let (bdt_b, _) = parse_segment_timing(seg_b);

            let end_a = bdt_a + total_dur_a;

            assert!(
                end_a <= bdt_b,
                "Segment {} end ({}) must not exceed segment {} start ({}): overlap of {} ticks",
                i,
                end_a,
                i + 1,
                bdt_b,
                end_a.saturating_sub(bdt_b)
            );
        }
    }

    /// Verify that large absolute timestamps (e.g. 12+ hours of camera uptime)
    /// are normalized to start near zero in the fMP4 output. This prevents
    /// hls.js PTS normalization issues with v1 (64-bit) tfdt.
    #[test]
    fn test_large_timestamps_normalized() {
        let mut segmenter = HlsSegmenter::new(1, Duration::from_secs(2));
        segmenter.set_codec(VideoCodec::H264);

        // Feed SPS + PPS
        let sps = vec![0x00, 0x00, 0x00, 0x01, 0x67, 0x42, 0x00, 0x1E, 0xAB, 0x40];
        let pps = vec![0x00, 0x00, 0x00, 0x01, 0x68, 0xCE, 0x3C, 0x80];
        segmenter.process_nal(&sps, 0, false);
        segmenter.process_nal(&pps, 1000, false);

        // Simulate 12 hours of camera uptime: ~43200 seconds = 43_200_000_000 microseconds
        let base_us = 43_200_000_000u64;

        let mut segments = Vec::new();
        let mut ts = base_us;

        for seg_idx in 0..3 {
            let idr = vec![0x00, 0x00, 0x00, 0x01, 0x65, 0x88, 0x80, seg_idx + 1];
            if let Some(seg) = segmenter.process_nal(&idr, ts, true) {
                segments.push(seg);
            }
            ts += 33_333;

            for i in 0..70 {
                let p = vec![0x00, 0x00, 0x00, 0x01, 0x41, 0x9A, (i & 0xFF) as u8];
                if let Some(seg) = segmenter.process_nal(&p, ts, false) {
                    segments.push(seg);
                }
                ts += 33_333;
            }
        }

        assert!(segments.len() >= 2, "Should produce segments");

        // First segment's baseMediaDecodeTime should be near 0, NOT 43200 seconds
        let (bdt_0, _) = parse_segment_timing(&segments[0].data);
        let bdt_0_secs = bdt_0 as f64 / 90000.0;
        assert!(
            bdt_0_secs < 1.0,
            "First segment baseMediaDecodeTime should be near 0, got {:.3}s (raw: {})",
            bdt_0_secs,
            bdt_0
        );

        // Segments should still be contiguous
        for i in 0..segments.len() - 1 {
            let (bdt_a, dur_a) = parse_segment_timing(&segments[i].data);
            let (bdt_b, _) = parse_segment_timing(&segments[i + 1].data);
            assert!(
                bdt_a + dur_a <= bdt_b,
                "Segments {} and {} should not overlap",
                i,
                i + 1
            );
        }
    }

    /// Parse baseMediaDecodeTime and total sample duration from a raw fMP4 segment.
    fn parse_segment_timing(data: &[u8]) -> (u64, u64) {
        // tfdt is at absolute offset 48 in the segment (8+16+8+16)
        // tfdt v1: baseMediaDecodeTime is u64 at offset 48+12 = 60
        let bdt = u64::from_be_bytes([
            data[60], data[61], data[62], data[63], data[64], data[65], data[66], data[67],
        ]);

        // trun starts at absolute offset 68, sample_count at 68+12=80
        let sample_count = u32::from_be_bytes([data[80], data[81], data[82], data[83]]) as usize;

        // Sample entries at 68+20=88, each 12 bytes: dur(4)+size(4)+flags(4)
        let mut total_dur = 0u64;
        for i in 0..sample_count {
            let off = 88 + i * 12;
            let dur = u32::from_be_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]]);
            total_dur += dur as u64;
        }

        (bdt, total_dur)
    }

    /// Helper: find box data by type in nested ISO BMFF structure.
    /// Returns the data portion (after version+flags for full boxes, after header for plain boxes).
    fn find_box_data<'a>(data: &'a [u8], box_type: &[u8; 4]) -> Option<&'a [u8]> {
        let mut offset = 0;
        while offset + 8 <= data.len() {
            let size = u32::from_be_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]) as usize;
            if size < 8 || offset + size > data.len() {
                break;
            }
            let btype = &data[offset + 4..offset + 8];
            if btype == box_type {
                // For tkhd (full box), skip version(1) + flags(3)
                return Some(&data[offset + 12..offset + size]);
            }
            // Recurse into container boxes
            if matches!(btype, b"moov" | b"trak" | b"mdia" | b"minf" | b"stbl") {
                if let Some(result) = find_box_data(&data[offset + 8..offset + size], box_type) {
                    return Some(result);
                }
            }
            offset += size;
        }
        None
    }
}

#[cfg(test)]
mod audio_tests {
    use super::*;

    /// Minimal ADTS frame: AAC-LC, 16 kHz (index 8) mono, `payload_len` body bytes.
    fn adts_frame(payload_len: usize) -> Vec<u8> {
        let frame_len = 7 + payload_len;
        let mut f = vec![0u8; frame_len];
        f[0] = 0xFF;
        f[1] = 0xF1;
        f[2] = (1 << 6) | (8 << 2);
        f[3] = (1 << 6) | ((frame_len >> 11) as u8 & 0x03);
        f[4] = (frame_len >> 3) as u8;
        f[5] = ((frame_len as u8 & 0x07) << 5) | 0x1F;
        f[6] = 0xFC;
        for (i, b) in f.iter_mut().enumerate().skip(7) {
            *b = 0xA0 | (i as u8 & 0x0F);
        }
        f
    }

    fn count_pattern(haystack: &[u8], needle: &[u8]) -> usize {
        haystack
            .windows(needle.len())
            .filter(|w| w == &needle)
            .count()
    }

    fn feed_video_preamble(segmenter: &mut HlsSegmenter) {
        let sps = vec![0x00, 0x00, 0x00, 0x01, 0x67, 0x42, 0x00, 0x1E, 0xAB, 0x40];
        let pps = vec![0x00, 0x00, 0x00, 0x01, 0x68, 0xCE, 0x3C, 0x80];
        segmenter.process_nal(&sps, 0, false);
        segmenter.process_nal(&pps, 1000, false);
    }

    #[test]
    fn test_init_waits_for_audio_params_then_includes_track() {
        let mut segmenter = HlsSegmenter::new(1, Duration::from_secs(2));
        segmenter.set_codec(VideoCodec::H264);
        segmenter.set_expect_audio(true);
        feed_video_preamble(&mut segmenter);

        // SPS+PPS present, but audio params are pending — init must wait.
        assert!(segmenter.generate_init_segment().is_none());

        // First ADTS frame supplies the params (even before any keyframe).
        segmenter.process_audio(&adts_frame(16), 0);

        let init = segmenter.generate_init_segment().expect("init with audio");
        assert_eq!(init.audio_object_type, Some(2)); // AAC-LC
        assert_eq!(count_pattern(&init.data, b"mp4a"), 1);
        assert_eq!(count_pattern(&init.data, b"smhd"), 1);
        assert_eq!(count_pattern(&init.data, b"soun"), 1);
        assert_eq!(count_pattern(&init.data, b"esds"), 1);
        assert_eq!(count_pattern(&init.data, b"trex"), 2, "one trex per track");
        assert_eq!(count_pattern(&init.data, b"trak"), 2, "video + audio traks");
    }

    #[test]
    fn test_video_only_init_when_audio_never_arrives() {
        let mut segmenter = HlsSegmenter::new(1, Duration::from_secs(2));
        segmenter.set_codec(VideoCodec::H264);
        segmenter.set_expect_audio(true);
        feed_video_preamble(&mut segmenter);

        // Run past the bounded wait without any audio.
        let idr = vec![0x00, 0x00, 0x00, 0x01, 0x65, 0x88, 0x80, 0x40];
        segmenter.process_nal(&idr, 100_000, true);
        for i in 1..=(AUDIO_PARAMS_WAIT_SAMPLES + 2) {
            let p = vec![0x00, 0x00, 0x00, 0x01, 0x41, 0x9A, (i & 0xFF) as u8];
            segmenter.process_nal(&p, 100_000 + i * 40_000, false);
        }

        let init = segmenter
            .generate_init_segment()
            .expect("video-only init after deadline");
        assert_eq!(init.audio_object_type, None);
        assert_eq!(count_pattern(&init.data, b"mp4a"), 0);
        assert_eq!(count_pattern(&init.data, b"trex"), 1);
    }

    #[test]
    fn test_segment_muxes_audio_traf_and_mdat() {
        let mut segmenter = HlsSegmenter::new(1, Duration::from_secs(2));
        segmenter.set_codec(VideoCodec::H264);
        segmenter.set_expect_audio(true);
        feed_video_preamble(&mut segmenter);

        // Audio frame before the keyframe sets the params; its sample is
        // dropped (no segment exists yet).
        segmenter.process_audio(&adts_frame(16), 0);
        segmenter.generate_init_segment().expect("init with audio");

        // Keyframe starts the segment.
        let idr = vec![0x00, 0x00, 0x00, 0x01, 0x65, 0x88, 0x80, 0x40];
        segmenter.process_nal(&idr, 100_000, true);

        // Audio frames during the segment: 16 kHz → one frame per 64 ms.
        let f1 = adts_frame(20);
        let f2 = adts_frame(24);
        segmenter.process_audio(&f1, 100_000);
        segmenter.process_audio(&f2, 164_000);

        // Video P-frames past target duration, then a keyframe finalizes.
        for i in 1..=10u64 {
            let p = vec![0x00, 0x00, 0x00, 0x01, 0x41, 0x9A, i as u8];
            segmenter.process_nal(&p, 100_000 + i * 300_000, false);
        }
        let idr2 = vec![0x00, 0x00, 0x00, 0x01, 0x65, 0x88, 0x80, 0x41];
        let segment = segmenter
            .process_nal(&idr2, 100_000 + 11 * 300_000, true)
            .expect("segment");

        let data = &segment.data;
        assert_eq!(count_pattern(data, b"traf"), 2, "video + audio trafs");

        // The audio payloads (ADTS stripped) are the tail of the mdat.
        let mut audio_payload = f1[7..].to_vec();
        audio_payload.extend_from_slice(&f2[7..]);
        assert!(
            data.ends_with(&audio_payload),
            "mdat must end with the raw AAC frames"
        );

        // The audio trun's data_offset must point exactly at those bytes.
        let audio_trun_idx = data
            .windows(4)
            .enumerate()
            .filter(|(_, w)| w == b"trun")
            .map(|(i, _)| i)
            .nth(1)
            .expect("second trun (audio)");
        let off_pos = audio_trun_idx + 12; // type + verflags + sample_count
        let audio_offset = u32::from_be_bytes([
            data[off_pos],
            data[off_pos + 1],
            data[off_pos + 2],
            data[off_pos + 3],
        ]) as usize;
        assert_eq!(audio_offset, data.len() - audio_payload.len());

        // Audio tfdt: first audio sample at segment start → 0 ticks.
        // Per-sample duration is 1024 ticks (timescale = sample rate).
        let audio_sample_count = u32::from_be_bytes([
            data[audio_trun_idx + 8],
            data[audio_trun_idx + 9],
            data[audio_trun_idx + 10],
            data[audio_trun_idx + 11],
        ]);
        assert_eq!(audio_sample_count, 2);

        // Next segment starts with cleared audio accumulation.
        let p = vec![0x00, 0x00, 0x00, 0x01, 0x41, 0x9A, 0xFE];
        segmenter.process_nal(&p, 100_000 + 12 * 300_000, false);
        // (no panic / no stale audio — structural smoke check)
    }

    #[test]
    fn test_non_adts_audio_is_skipped() {
        let mut segmenter = HlsSegmenter::new(1, Duration::from_secs(2));
        segmenter.set_codec(VideoCodec::H264);
        segmenter.set_expect_audio(true);
        feed_video_preamble(&mut segmenter);

        // Raw AAC (no syncword) — cannot derive params.
        segmenter.process_audio(&[0x21, 0x1B, 0x80, 0x00], 0);
        assert!(segmenter.audio_params.is_none());
        // Init is still gated (bounded) — params never arrive, so after the
        // wait the init is video-only (covered by the deadline test above).
    }

    #[test]
    fn test_audio_after_video_only_init_is_dropped() {
        let mut segmenter = HlsSegmenter::new(1, Duration::from_secs(2));
        segmenter.set_codec(VideoCodec::H264);
        // expect_audio NOT set: init generates video-only immediately.
        feed_video_preamble(&mut segmenter);
        let init = segmenter.generate_init_segment().expect("video-only init");
        assert_eq!(init.audio_object_type, None);

        // Late audio cannot be added to a served init — dropped.
        let idr = vec![0x00, 0x00, 0x00, 0x01, 0x65, 0x88, 0x80, 0x40];
        segmenter.process_nal(&idr, 100_000, true);
        segmenter.process_audio(&adts_frame(16), 100_000);
        assert!(segmenter.current_segment_audio.is_empty());

        // Segment stays single-traf.
        for i in 1..=10u64 {
            let p = vec![0x00, 0x00, 0x00, 0x01, 0x41, 0x9A, i as u8];
            segmenter.process_nal(&p, 100_000 + i * 300_000, false);
        }
        let idr2 = vec![0x00, 0x00, 0x00, 0x01, 0x65, 0x88, 0x80, 0x41];
        let segment = segmenter
            .process_nal(&idr2, 100_000 + 11 * 300_000, true)
            .expect("segment");
        assert_eq!(count_pattern(&segment.data, b"traf"), 1);
    }
}
