//! On-demand HLS-VOD packaging of recorded event MP4s.
//!
//! Uses the ffmpeg libraries (libavformat demux via `ffmpeg-next`) — never the
//! `ffmpeg` binary. ffmpeg-next exposes no bitstream-filter API, so the
//! AVCC→Annex B conversion and the SPS/PPS/VPS extraction from the `avcC`/`hvcC`
//! extradata are done in-Rust here, then fed to the existing [`HlsSegmenter`].
//!
//! Results are cached per event id (a recorded file is immutable). Generation
//! is single-flighted so concurrent requests don't re-segment the same event.

use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use dashmap::DashMap;
use tokio::sync::Mutex;

use crate::streaming::hls::segmenter::{FMP4Segment, HlsSegmenter};
use crate::streaming::probe::MediaInfo;
use crate::streaming::source::fifo::VideoCodec;

/// Target VOD segment duration. Recorded clips are short, so a small value
/// keeps seek granularity reasonable without producing too many segments.
const VOD_SEGMENT_SECONDS: u64 = 4;

/// Packaged HLS-VOD assets for one event: the fMP4 init segment, the media
/// segments, and the VOD playlist that references them.
pub struct VodAssets {
    pub init: Vec<u8>,
    pub segments: Vec<Vec<u8>>,
    pub playlist: String,
}

fn cache() -> &'static DashMap<u64, Arc<VodAssets>> {
    static CACHE: OnceLock<DashMap<u64, Arc<VodAssets>>> = OnceLock::new();
    CACHE.get_or_init(DashMap::new)
}

fn locks() -> &'static DashMap<u64, Arc<Mutex<()>>> {
    static LOCKS: OnceLock<DashMap<u64, Arc<Mutex<()>>>> = OnceLock::new();
    LOCKS.get_or_init(DashMap::new)
}

/// Return the cached VOD assets for an event, packaging them on first request.
pub async fn get_or_build(
    event_id: u64,
    video_path: PathBuf,
    info: MediaInfo,
) -> Result<Arc<VodAssets>, String> {
    if let Some(a) = cache().get(&event_id) {
        return Ok(a.clone());
    }
    // Single-flight: serialize concurrent first-time builds for the same event.
    let lock = locks()
        .entry(event_id)
        .or_insert_with(|| Arc::new(Mutex::new(())))
        .clone();
    let _guard = lock.lock().await;
    if let Some(a) = cache().get(&event_id) {
        return Ok(a.clone());
    }

    let target = Duration::from_secs(VOD_SEGMENT_SECONDS);
    let assets = Arc::new(
        tokio::task::spawn_blocking(move || package_blocking(&video_path, info, target))
            .await
            .map_err(|e| format!("vod task failed: {e}"))??,
    );
    cache().insert(event_id, assets.clone());
    Ok(assets)
}

fn package_blocking(path: &Path, info: MediaInfo, target: Duration) -> Result<VodAssets, String> {
    use ffmpeg_next as ffmpeg;

    let mut ictx = ffmpeg::format::input(&path).map_err(|e| format!("open {path:?}: {e}"))?;

    // Pull the stream index, time base and extradata in a scoped borrow so the
    // mutable `packets()` borrow below is unencumbered.
    let (stream_index, time_base, extradata) = {
        let stream = ictx
            .streams()
            .best(ffmpeg::media::Type::Video)
            .ok_or("no video stream")?;
        (
            stream.index(),
            stream.time_base(),
            read_extradata(&stream.parameters()),
        )
    };

    let mut seg = HlsSegmenter::new(0, target);
    seg.set_codec(info.codec);
    seg.set_dimensions(info.width, info.height);

    // Feed SPS/PPS/VPS (from the avcC/hvcC extradata) so the segmenter can build
    // the init segment — MP4 packets do not carry them inband.
    let (param_nals, length_size) = parse_extradata(&extradata, info.codec)?;
    if param_nals.is_empty() {
        return Err("no parameter sets in extradata".to_string());
    }
    for nal in &param_nals {
        seg.process_nal(nal, 0, false);
    }
    let init = seg
        .generate_init_segment()
        .ok_or("failed to build init segment")?;

    let num = i128::from(time_base.numerator());
    let den = i128::from(time_base.denominator()).max(1);

    let mut segments: Vec<FMP4Segment> = Vec::new();
    for (stream, packet) in ictx.packets() {
        if stream.index() != stream_index {
            continue;
        }
        let Some(data) = packet.data() else { continue };
        let pts = packet.pts().or_else(|| packet.dts()).unwrap_or(0).max(0);
        let ts_us = (i128::from(pts) * num * 1_000_000 / den) as u64;
        let is_key = packet.is_key();
        for nal in avcc_to_annexb(data, length_size) {
            if let Some(s) = seg.process_nal(&nal, ts_us, is_key) {
                segments.push(s);
            }
        }
    }
    if let Some(s) = seg.flush() {
        segments.push(s);
    }
    if segments.is_empty() {
        return Err("no segments produced".to_string());
    }

    let playlist = build_playlist(&segments, target);
    Ok(VodAssets {
        init: init.data,
        segments: segments.into_iter().map(|s| s.data).collect(),
        playlist,
    })
}

/// Read a codec's extradata (the avcC/hvcC box) from ffmpeg parameters.
fn read_extradata(params: &ffmpeg_next::codec::Parameters) -> Vec<u8> {
    // SAFETY: `as_ptr()` yields a valid AVCodecParameters for the lifetime of
    // `params`; we only read the extradata slice it points at.
    unsafe {
        let p = params.as_ptr();
        let size = (*p).extradata_size as usize;
        if size == 0 || (*p).extradata.is_null() {
            return Vec::new();
        }
        std::slice::from_raw_parts((*p).extradata, size).to_vec()
    }
}

/// Convert a length-prefixed (AVCC) sample into Annex B NAL units.
fn avcc_to_annexb(data: &[u8], length_size: usize) -> Vec<Vec<u8>> {
    let mut nals = Vec::new();
    let mut i = 0;
    while i + length_size <= data.len() {
        let mut len = 0usize;
        for &b in &data[i..i + length_size] {
            len = (len << 8) | b as usize;
        }
        i += length_size;
        if len == 0 || i + len > data.len() {
            break;
        }
        let mut nal = vec![0x00, 0x00, 0x00, 0x01];
        nal.extend_from_slice(&data[i..i + len]);
        nals.push(nal);
        i += len;
    }
    nals
}

/// Parse SPS/PPS(/VPS) parameter sets and the NAL length size from avcC/hvcC.
/// Returns Annex B-framed parameter-set NALs and the AVCC length-prefix size.
fn parse_extradata(extradata: &[u8], codec: VideoCodec) -> Result<(Vec<Vec<u8>>, usize), String> {
    match codec {
        VideoCodec::H264 => parse_avcc(extradata),
        VideoCodec::H265 => parse_hvcc(extradata),
        VideoCodec::Unknown => Err("unknown codec".to_string()),
    }
}

fn annexb(nal: &[u8]) -> Vec<u8> {
    let mut v = vec![0x00, 0x00, 0x00, 0x01];
    v.extend_from_slice(nal);
    v
}

/// Parse an AVCDecoderConfigurationRecord (avcC).
fn parse_avcc(d: &[u8]) -> Result<(Vec<Vec<u8>>, usize), String> {
    if d.len() < 7 {
        return Err("avcC too short".to_string());
    }
    let length_size = (d[4] & 0x03) as usize + 1;
    let mut nals = Vec::new();
    let mut i = 5;
    let num_sps = (d[i] & 0x1f) as usize;
    i += 1;
    for _ in 0..num_sps {
        if i + 2 > d.len() {
            return Err("avcC SPS truncated".to_string());
        }
        let len = ((d[i] as usize) << 8) | d[i + 1] as usize;
        i += 2;
        if i + len > d.len() {
            return Err("avcC SPS truncated".to_string());
        }
        nals.push(annexb(&d[i..i + len]));
        i += len;
    }
    if i >= d.len() {
        return Err("avcC PPS missing".to_string());
    }
    let num_pps = d[i] as usize;
    i += 1;
    for _ in 0..num_pps {
        if i + 2 > d.len() {
            return Err("avcC PPS truncated".to_string());
        }
        let len = ((d[i] as usize) << 8) | d[i + 1] as usize;
        i += 2;
        if i + len > d.len() {
            return Err("avcC PPS truncated".to_string());
        }
        nals.push(annexb(&d[i..i + len]));
        i += len;
    }
    Ok((nals, length_size))
}

/// Parse an HEVCDecoderConfigurationRecord (hvcC).
fn parse_hvcc(d: &[u8]) -> Result<(Vec<Vec<u8>>, usize), String> {
    if d.len() < 23 {
        return Err("hvcC too short".to_string());
    }
    let length_size = (d[21] & 0x03) as usize + 1;
    let num_arrays = d[22] as usize;
    let mut nals = Vec::new();
    let mut i = 23;
    for _ in 0..num_arrays {
        if i + 3 > d.len() {
            return Err("hvcC array header truncated".to_string());
        }
        // d[i]: array_completeness(1) | reserved(1) | NAL_unit_type(6)
        i += 1; // NAL_unit_type not needed: any of VPS/SPS/PPS goes to the stream
        let num_nalus = ((d[i] as usize) << 8) | d[i + 1] as usize;
        i += 2;
        for _ in 0..num_nalus {
            if i + 2 > d.len() {
                return Err("hvcC nalu length truncated".to_string());
            }
            let len = ((d[i] as usize) << 8) | d[i + 1] as usize;
            i += 2;
            if i + len > d.len() {
                return Err("hvcC nalu truncated".to_string());
            }
            nals.push(annexb(&d[i..i + len]));
            i += len;
        }
    }
    Ok((nals, length_size))
}

/// Build a VOD media playlist (#EXT-X-PLAYLIST-TYPE:VOD) referencing the init
/// segment and the media segments by sequence index.
fn build_playlist(segments: &[FMP4Segment], target: Duration) -> String {
    let target_secs = segments
        .iter()
        .map(|s| s.duration.as_secs_f64())
        .fold(target.as_secs_f64(), f64::max)
        .ceil()
        .max(1.0) as u64;
    let mut p = String::with_capacity(128 + segments.len() * 48);
    p.push_str("#EXTM3U\n#EXT-X-VERSION:7\n");
    p.push_str(&format!("#EXT-X-TARGETDURATION:{target_secs}\n"));
    p.push_str("#EXT-X-PLAYLIST-TYPE:VOD\n");
    p.push_str("#EXT-X-INDEPENDENT-SEGMENTS\n");
    // URIs are relative to the playlist URL (…/stream/): "init.mp4" -> the
    // static init route; "segment/N" -> the parameterized segment route.
    p.push_str("#EXT-X-MAP:URI=\"init.mp4\"\n");
    for (i, s) in segments.iter().enumerate() {
        p.push_str(&format!(
            "#EXTINF:{:.3},\nsegment/{i}\n",
            s.duration.as_secs_f64()
        ));
    }
    p.push_str("#EXT-X-ENDLIST\n");
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn avcc_splits_length_prefixed_nals() {
        // Two NALs: [00 00 00 03][AA BB CC] [00 00 00 02][DD EE]
        let data = [0, 0, 0, 3, 0xAA, 0xBB, 0xCC, 0, 0, 0, 2, 0xDD, 0xEE];
        let nals = avcc_to_annexb(&data, 4);
        assert_eq!(nals.len(), 2);
        assert_eq!(nals[0], vec![0, 0, 0, 1, 0xAA, 0xBB, 0xCC]);
        assert_eq!(nals[1], vec![0, 0, 0, 1, 0xDD, 0xEE]);
    }

    #[test]
    fn avcc_stops_on_truncation() {
        // Declares length 9 but only 3 bytes follow.
        let data = [0, 0, 0, 9, 0xAA, 0xBB, 0xCC];
        assert!(avcc_to_annexb(&data, 4).is_empty());
    }

    #[test]
    fn parse_avcc_extracts_sps_pps() {
        // version, profile, compat, level, 0xFF (lenSize=4), 0xE1 (1 SPS),
        // SPS len=2 [67 42], 1 PPS, PPS len=2 [68 CE]
        let avcc = [
            1, 0x42, 0, 0x1f, 0xff, 0xe1, 0, 2, 0x67, 0x42, 1, 0, 2, 0x68, 0xce,
        ];
        let (nals, ls) = parse_avcc(&avcc).unwrap();
        assert_eq!(ls, 4);
        assert_eq!(nals.len(), 2);
        assert_eq!(nals[0], vec![0, 0, 0, 1, 0x67, 0x42]);
        assert_eq!(nals[1], vec![0, 0, 0, 1, 0x68, 0xce]);
    }

    #[test]
    fn parse_hvcc_extracts_vps_sps_pps() {
        // 21 bytes of config header, then byte 21 = lengthSizeMinusOne (0xFF ->
        // length_size 4), byte 22 = num_arrays (3). Each array: NAL-type byte,
        // num_nalus (u16), then [len(u16)][data] per nalu. Three arrays carry
        // one VPS (0x40 01), SPS (0x42 01) and PPS (0x44 01) respectively.
        let mut hvcc = vec![0u8; 21];
        hvcc.push(0xFF);
        hvcc.push(3);
        hvcc.extend_from_slice(&[0x20, 0x00, 0x01, 0x00, 0x02, 0x40, 0x01]); // VPS
        hvcc.extend_from_slice(&[0x21, 0x00, 0x01, 0x00, 0x02, 0x42, 0x01]); // SPS
        hvcc.extend_from_slice(&[0x22, 0x00, 0x01, 0x00, 0x02, 0x44, 0x01]); // PPS

        let (nals, ls) = parse_hvcc(&hvcc).unwrap();
        assert_eq!(ls, 4);
        assert_eq!(nals.len(), 3);
        assert_eq!(nals[0], vec![0, 0, 0, 1, 0x40, 0x01]);
        assert_eq!(nals[1], vec![0, 0, 0, 1, 0x42, 0x01]);
        assert_eq!(nals[2], vec![0, 0, 0, 1, 0x44, 0x01]);
    }

    #[test]
    fn parse_hvcc_rejects_short_record() {
        assert!(parse_hvcc(&[0u8; 10]).is_err());
    }

    fn seg(duration: Duration) -> FMP4Segment {
        FMP4Segment {
            sequence: 0,
            data: Vec::new(),
            duration,
            timestamp: 0,
            is_keyframe: true,
        }
    }

    #[test]
    fn build_playlist_is_well_formed_vod() {
        let segments = [
            seg(Duration::from_millis(4000)),
            seg(Duration::from_millis(3500)),
            seg(Duration::from_millis(1200)),
        ];
        let p = build_playlist(&segments, Duration::from_secs(4));

        assert!(p.starts_with("#EXTM3U"));
        assert!(p.contains("#EXT-X-VERSION:7"));
        assert!(p.contains("#EXT-X-PLAYLIST-TYPE:VOD"));
        assert!(p.contains("#EXT-X-INDEPENDENT-SEGMENTS"));
        // TARGETDURATION is the ceil of the longest segment / target.
        assert!(p.contains("#EXT-X-TARGETDURATION:4"));
        // Init segment is referenced via a relative MAP URI.
        assert!(p.contains("#EXT-X-MAP:URI=\"init.mp4\""));
        // Per-segment EXTINF + relative segment URIs, in order.
        assert!(p.contains("#EXTINF:4.000,\nsegment/0"));
        assert!(p.contains("#EXTINF:3.500,\nsegment/1"));
        assert!(p.contains("#EXTINF:1.200,\nsegment/2"));
        // VOD playlists must terminate with ENDLIST.
        assert!(p.trim_end().ends_with("#EXT-X-ENDLIST"));
        // Exactly one URI per segment.
        assert_eq!(p.matches("\nsegment/").count(), 3);
    }
}
