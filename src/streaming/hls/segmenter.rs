//! fMP4 (Fragmented MP4) segment generation for HLS
//!
//! Generates ISO BMFF compliant fMP4 segments from H.264/H.265 NAL units.
//! Supports both initialization segments (ftyp + moov) and media segments (moof + mdat).

use bytes::{BufMut, BytesMut};
use std::time::Duration;

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

    fn len(&self) -> usize {
        self.buffer.len()
    }
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
    current_segment_data: Vec<(Vec<u8>, u64, bool)>, // (nal_data, timestamp, is_keyframe)
    segment_start_time: Option<u64>,
    target_duration: Duration,
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
            current_segment_data: Vec::new(),
            segment_start_time: None,
            target_duration,
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

        // Convert timestamp to timescale units
        let timestamp = (timestamp_us * self.timescale as u64) / 1_000_000;

        // Initialize segment start time
        if self.segment_start_time.is_none() {
            self.segment_start_time = Some(timestamp);
        }

        // Add NAL to current segment
        self.current_segment_data
            .push((nal_data.to_vec(), timestamp, is_keyframe));

        // Check if we should finalize segment (on keyframe after target duration)
        let segment_duration = Duration::from_micros(
            timestamp_us - (self.segment_start_time.unwrap() * 1_000_000 / self.timescale as u64),
        );

        if is_keyframe
            && segment_duration >= self.target_duration
            && self.current_segment_data.len() > 1
        {
            // Remove the current keyframe (it will start the next segment)
            let next_segment_start = self.current_segment_data.pop();

            let segment = self.finalize_segment();

            // Start new segment with the keyframe
            if let Some(start_nal) = next_segment_start {
                self.current_segment_data.push(start_nal);
                self.segment_start_time = Some(timestamp);
            }

            return segment;
        }

        None
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

    /// Parse H.264 SPS for dimensions (simplified)
    fn parse_h264_sps(&mut self, _sps: &[u8]) {
        // In production, parse the SPS to extract actual dimensions
        // For now, use configured defaults
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
        if self.current_segment_data.is_empty() {
            return None;
        }

        let start_time = self.segment_start_time?;
        let has_keyframe = self.current_segment_data.iter().any(|(_, _, kf)| *kf);

        // Calculate duration
        let last_timestamp = self
            .current_segment_data
            .last()
            .map(|(_, ts, _)| *ts)
            .unwrap_or(start_time);
        let duration_ticks = last_timestamp.saturating_sub(start_time);
        let duration = Duration::from_micros(duration_ticks * 1_000_000 / self.timescale as u64);

        // Build moof + mdat
        let mut builder = BoxBuilder::new();

        // moof
        let moof = self.build_moof(start_time);
        builder.write_box(b"moof", &moof);

        let moof_size = builder.len();

        // mdat
        let mdat = self.build_mdat(moof_size);
        builder.write_box(b"mdat", &mdat);

        let segment = FMP4Segment {
            sequence: self.sequence,
            data: builder.into_bytes(),
            duration,
            timestamp: start_time,
            is_keyframe: has_keyframe,
        };

        self.sequence += 1;
        self.current_segment_data.clear();
        self.segment_start_time = None;

        Some(segment)
    }

    /// Build moof box
    fn build_moof(&self, base_time: u64) -> Vec<u8> {
        let mut moof = BoxBuilder::new();

        // mfhd
        let mut mfhd = BytesMut::with_capacity(4);
        mfhd.put_u32(self.sequence as u32 + 1);
        moof.write_full_box(b"mfhd", 0, 0, &mfhd);

        // traf
        let traf = self.build_traf(base_time);
        moof.write_box(b"traf", &traf);

        moof.into_bytes()
    }

    /// Build traf box
    fn build_traf(&self, base_time: u64) -> Vec<u8> {
        let mut traf = BoxBuilder::new();

        // tfhd
        let mut tfhd = BytesMut::with_capacity(4);
        tfhd.put_u32(1); // track_ID
        traf.write_full_box(b"tfhd", 0, 0x020000, &tfhd); // default-base-is-moof

        // tfdt
        let mut tfdt = BytesMut::with_capacity(8);
        tfdt.put_u64(base_time);
        traf.write_full_box(b"tfdt", 1, 0, &tfdt);

        // trun
        let trun = self.build_trun();
        traf.write_full_box(b"trun", 0, 0x000F01, &trun); // data-offset, first-sample-flags, sample-duration, sample-size, sample-flags

        traf.into_bytes()
    }

    /// Build trun box data
    fn build_trun(&self) -> Vec<u8> {
        let mut trun = BytesMut::with_capacity(256);
        trun.put_u32(self.current_segment_data.len() as u32); // sample_count
        trun.put_u32(0); // data_offset (will be patched)

        // First sample flags (will be set for keyframe)
        let first_is_keyframe = self
            .current_segment_data
            .first()
            .map(|(_, _, kf)| *kf)
            .unwrap_or(false);
        if first_is_keyframe {
            trun.put_u32(0x02000000); // depends on nothing
        } else {
            trun.put_u32(0x01010000); // non-sync sample
        }

        // Sample entries
        let mut prev_ts = self.segment_start_time.unwrap_or(0);
        for (nal_data, ts, is_keyframe) in &self.current_segment_data {
            let duration = ts.saturating_sub(prev_ts).max(1);
            prev_ts = *ts;

            // Convert NAL to length-prefixed format
            let sample_size = 4 + nal_data.len() - self.nal_start_code_len(nal_data);

            trun.put_u32(duration as u32); // sample_duration
            trun.put_u32(sample_size as u32); // sample_size
            if *is_keyframe {
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

    /// Build mdat box data
    fn build_mdat(&self, moof_size: usize) -> Vec<u8> {
        let mut mdat = BytesMut::with_capacity(65536);

        for (nal_data, _, _) in &self.current_segment_data {
            let start_code_len = self.nal_start_code_len(nal_data);
            let nal_content = &nal_data[start_code_len..];

            // Write length-prefixed NAL unit
            mdat.put_u32(nal_content.len() as u32);
            mdat.put_slice(nal_content);
        }

        // Patch data_offset in trun (after moof)
        // The data_offset should point to the first byte of sample data in mdat
        let _data_offset = moof_size + 8; // moof_size + mdat header

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
        assert!(init.unwrap().data.len() > 0);
    }
}
