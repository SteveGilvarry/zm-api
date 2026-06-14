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
use crate::streaming::source::media::{avcc_to_annexb, parse_extradata};

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
    use crate::streaming::source::VideoCodec;

    fn seg(duration: Duration) -> FMP4Segment {
        FMP4Segment {
            sequence: 0,
            data: Vec::new(),
            duration,
            timestamp: 0,
            is_keyframe: true,
        }
    }

    /// Mux `num_frames` blank 64×64 frames into a real MP4 at `path` using the
    /// named libavcodec encoder (GLOBAL_HEADER set, so the avcC/hvcC lands in
    /// the container — exactly what `package_blocking` reads). Returns the frame
    /// count, or `None` if the encoder isn't in this ffmpeg build (skip).
    fn generate_test_mp4(encoder_name: &str, path: &Path, num_frames: i64) -> Option<i64> {
        use ffmpeg_next as ffmpeg;
        ffmpeg::init().ok();

        let codec = ffmpeg::codec::encoder::find_by_name(encoder_name)?;
        let mut octx = ffmpeg::format::output(&path).ok()?;
        let global_header = octx
            .format()
            .flags()
            .contains(ffmpeg::format::Flags::GLOBAL_HEADER);

        let mut enc_ctx = ffmpeg::codec::context::Context::new_with_codec(codec)
            .encoder()
            .video()
            .ok()?;
        enc_ctx.set_width(64);
        enc_ctx.set_height(64);
        enc_ctx.set_format(ffmpeg::format::Pixel::YUV420P);
        let tb = ffmpeg::Rational(1, 25);
        enc_ctx.set_time_base(tb);
        enc_ctx.set_gop(10);
        if global_header {
            enc_ctx.set_flags(ffmpeg::codec::Flags::GLOBAL_HEADER);
        }
        let mut opts = ffmpeg::Dictionary::new();
        opts.set("preset", "ultrafast");
        let mut enc = enc_ctx.open_with(opts).ok()?;

        {
            let mut ost = octx.add_stream(codec).ok()?;
            ost.set_parameters(&enc);
            ost.set_time_base(tb);
        }
        octx.write_header().ok()?;
        let ost_tb = octx.stream(0)?.time_base();

        fn drain(
            enc: &mut ffmpeg_next::encoder::Video,
            octx: &mut ffmpeg_next::format::context::Output,
            tb: ffmpeg_next::Rational,
            ost_tb: ffmpeg_next::Rational,
        ) -> Option<()> {
            let mut pkt = ffmpeg_next::Packet::empty();
            while enc.receive_packet(&mut pkt).is_ok() {
                pkt.set_stream(0);
                pkt.rescale_ts(tb, ost_tb);
                pkt.write_interleaved(octx).ok()?;
            }
            Some(())
        }

        for i in 0..num_frames {
            let mut frame = ffmpeg::frame::Video::new(ffmpeg::format::Pixel::YUV420P, 64, 64);
            // Vary the luma per frame so encoded packets aren't degenerate.
            for plane in 0..3usize {
                let v = if plane == 0 {
                    (i as u8).wrapping_mul(7)
                } else {
                    128
                };
                frame.data_mut(plane).fill(v);
            }
            frame.set_pts(Some(i));
            enc.send_frame(&frame).ok()?;
            drain(&mut enc, &mut octx, tb, ost_tb)?;
        }
        enc.send_eof().ok()?;
        drain(&mut enc, &mut octx, tb, ost_tb)?;
        octx.write_trailer().ok()?;
        Some(num_frames)
    }

    /// Demux `path` and return (video packet count, codec id).
    fn count_video_packets(path: &Path) -> Option<(usize, ffmpeg_next::codec::Id)> {
        use ffmpeg_next as ffmpeg;
        let mut ictx = ffmpeg::format::input(&path).ok()?;
        let (idx, id) = {
            let st = ictx.streams().best(ffmpeg::media::Type::Video)?;
            let id = ffmpeg::codec::context::Context::from_parameters(st.parameters())
                .ok()?
                .id();
            (st.index(), id)
        };
        let count = ictx.packets().filter(|(s, _)| s.index() == idx).count();
        Some((count, id))
    }

    /// End-to-end: generate a real MP4, run the full probe + package pipeline,
    /// reconstruct a fragmented MP4 from init+segments, and assert every input
    /// frame survives the round-trip. Gated on encoder availability.
    async fn assert_real_mp4_roundtrips(encoder: &str, expect_codec: VideoCodec, event_id: u64) {
        use ffmpeg_next as ffmpeg;
        let dir = tempfile::tempdir().expect("tempdir");
        let src = dir.path().join("source.mp4");

        let num_frames = 50;
        let Some(_) = generate_test_mp4(encoder, &src, num_frames) else {
            eprintln!("skipping: encoder {encoder} not available in this ffmpeg build");
            return;
        };

        // Probe the real file (exercises probe_event_media end-to-end).
        let info = crate::streaming::probe::probe_event_media(event_id, src.clone())
            .await
            .expect("probe should succeed on the generated mp4");
        assert_eq!(info.codec, expect_codec, "probed codec mismatch");
        assert_eq!((info.width, info.height), (64, 64), "probed dimensions");

        // Package into HLS-VOD with a small target so multiple segments form.
        let assets = package_blocking(&src, info, Duration::from_millis(500))
            .expect("package_blocking should succeed");

        assert_eq!(
            &assets.init[4..8],
            b"ftyp",
            "init must begin with an ftyp box"
        );
        assert!(!assets.segments.is_empty(), "must produce media segments");
        assert!(
            assets.playlist.contains("#EXT-X-ENDLIST"),
            "VOD playlist must terminate with ENDLIST"
        );

        // Source packet count (frames) for parity.
        let (src_count, src_id) = count_video_packets(&src).expect("demux source");
        assert!(src_count > 0, "source must contain packets");
        let expect_id = match expect_codec {
            VideoCodec::H264 => ffmpeg::codec::Id::H264,
            VideoCodec::H265 => ffmpeg::codec::Id::HEVC,
            VideoCodec::Unknown => unreachable!(),
        };
        assert_eq!(src_id, expect_id, "source codec id");

        // Reconstruct a fragmented MP4: init segment followed by every media
        // segment. ffmpeg demuxes this as one stream — packet count must match.
        let recon = dir.path().join("recon.mp4");
        let mut buf = assets.init.clone();
        for seg in &assets.segments {
            buf.extend_from_slice(seg);
        }
        std::fs::write(&recon, &buf).expect("write recon");

        let (recon_count, recon_id) = count_video_packets(&recon).expect("demux recon");
        assert_eq!(
            recon_id, expect_id,
            "reconstructed codec id must match source"
        );
        assert_eq!(
            recon_count, src_count,
            "every source frame must survive the VOD round-trip ({recon_count} vs {src_count})"
        );
    }

    #[tokio::test]
    async fn vod_packages_real_h264_mp4_without_frame_loss() {
        assert_real_mp4_roundtrips("libx264", VideoCodec::H264, 9_900_001).await;
    }

    #[tokio::test]
    async fn vod_packages_real_h265_mp4_without_frame_loss() {
        assert_real_mp4_roundtrips("libx265", VideoCodec::H265, 9_900_002).await;
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
