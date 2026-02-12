//! fMP4 (Fragmented MP4) segment generation for HLS
//!
//! Generates ISO BMFF compliant fMP4 segments from H.264/H.265 NAL units.
//! Supports both initialization segments (ftyp + moov) and media segments (moof + mdat).

use bytes::{BufMut, BytesMut};
use std::time::Duration;

use super::h264;
use crate::streaming::source::fifo::VideoCodec;

/// fMP4 initialization segment containing ftyp and moov boxes
#[derive(Debug, Clone)]
pub struct InitSegment {
    pub data: Vec<u8>,
    pub codec: VideoCodec,
    pub width: u32,
    pub height: u32,
    pub timescale: u32,
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

        // Convert timestamp to timescale units
        let timestamp = (timestamp_us * self.timescale as u64) / 1_000_000;

        // Initialize segment start time
        if self.segment_start_time.is_none() {
            self.segment_start_time = Some(timestamp);
        }

        // Access unit aggregation: flush pending frame when a new frame begins.
        // A new frame starts when the timestamp changes. NALs with the same
        // timestamp belong to the same access unit (multi-slice frames).
        let mut result = None;
        if let Some(ref pending) = self.pending_frame {
            let is_new_frame = timestamp != pending.timestamp;
            if is_new_frame {
                // Flush the pending frame to completed samples
                let flushed = self.pending_frame.take().unwrap();
                self.current_segment_samples.push(SegmentSample {
                    nals: flushed.nals,
                    timestamp: flushed.timestamp,
                    is_keyframe: flushed.is_keyframe,
                });

                // Check if we should finalize segment (on keyframe after target duration)
                if is_keyframe && self.current_segment_samples.len() > 1 {
                    let seg_start_us =
                        self.segment_start_time.unwrap() * 1_000_000 / self.timescale as u64;
                    let segment_duration = Duration::from_micros(timestamp_us - seg_start_us);

                    if segment_duration >= self.target_duration {
                        result = self.finalize_segment();
                        self.segment_start_time = Some(timestamp);
                    }
                }
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
        let sps = self.sps.as_ref().unwrap();
        let pps = self.pps.as_ref().unwrap();

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
        let sps = self.sps.as_ref().unwrap();
        let pps = self.pps.as_ref().unwrap();
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

        mvex.into_bytes()
    }

    /// Finalize current segment and return it
    fn finalize_segment(&mut self) -> Option<FMP4Segment> {
        if self.current_segment_samples.is_empty() {
            return None;
        }

        let start_time = self.segment_start_time?;
        let has_keyframe = self.current_segment_samples.iter().any(|s| s.is_keyframe);

        // Calculate duration
        let last_timestamp = self
            .current_segment_samples
            .last()
            .map(|s| s.timestamp)
            .unwrap_or(start_time);
        let duration_ticks = last_timestamp.saturating_sub(start_time);
        let duration = Duration::from_micros(duration_ticks * 1_000_000 / self.timescale as u64);

        // Calculate data_offset: offset from start of moof box to first sample byte in mdat.
        // moof box = 8 (header) + 16 (mfhd) + 8 (traf header) + 16 (tfhd) + 20 (tfdt)
        //          + 12 (trun header) + 8 (sample_count + data_offset) + N*12 (per-sample)
        // = 88 + N*12
        // data_offset = moof_box_size + 8 (mdat header) = 96 + N*12
        let n = self.current_segment_samples.len();
        let data_offset = (96 + n * 12) as u32;

        // Build moof + mdat
        let mut builder = BoxBuilder::new();

        let moof = self.build_moof(start_time, data_offset);
        builder.write_box(b"moof", &moof);

        let mdat = self.build_mdat_data();
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
        self.segment_start_time = None;

        Some(segment)
    }

    /// Build moof box
    fn build_moof(&self, base_time: u64, data_offset: u32) -> Vec<u8> {
        let mut moof = BoxBuilder::new();

        // mfhd
        let mut mfhd = BytesMut::with_capacity(4);
        mfhd.put_u32(self.sequence as u32 + 1);
        moof.write_full_box(b"mfhd", 0, 0, &mfhd);

        // traf
        let traf = self.build_traf(base_time, data_offset);
        moof.write_box(b"traf", &traf);

        moof.into_bytes()
    }

    /// Build traf box
    fn build_traf(&self, base_time: u64, data_offset: u32) -> Vec<u8> {
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
        let trun = self.build_trun(data_offset);
        traf.write_full_box(b"trun", 0, 0x000701, &trun);

        traf.into_bytes()
    }

    /// Build trun box data
    fn build_trun(&self, data_offset: u32) -> Vec<u8> {
        let mut trun = BytesMut::with_capacity(256);
        trun.put_u32(self.current_segment_samples.len() as u32); // sample_count
        trun.put_u32(data_offset); // data_offset (relative to moof start)

        // Per-sample entries: duration + size + flags
        let mut prev_ts = self.segment_start_time.unwrap_or(0);
        for sample in &self.current_segment_samples {
            let duration = sample.timestamp.saturating_sub(prev_ts).max(1);
            prev_ts = sample.timestamp;

            // Sample size = sum of all length-prefixed NALs in the access unit
            let sample_size: usize = sample
                .nals
                .iter()
                .map(|nal| 4 + nal.len() - self.nal_start_code_len(nal))
                .sum();

            trun.put_u32(duration as u32); // sample_duration
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

    /// Build mdat box inner data (length-prefixed NAL units)
    fn build_mdat_data(&self) -> Vec<u8> {
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

        mdat.to_vec()
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

        // Feed two IDR slices with same timestamp (multi-slice frame)
        let idr_slice1 = vec![0x00, 0x00, 0x00, 0x01, 0x65, 0x88, 0x80, 0x40];
        let idr_slice2 = vec![0x00, 0x00, 0x00, 0x01, 0x65, 0x88, 0x80, 0x41];
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

        // Feed multi-slice keyframe (2 slices, same timestamp)
        let idr1 = vec![0x00, 0x00, 0x00, 0x01, 0x65, 0xAA, 0xBB];
        let idr2 = vec![0x00, 0x00, 0x00, 0x01, 0x65, 0xCC, 0xDD];
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
        assert_eq!(data[nal2_start + 5], 0xCC);
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
