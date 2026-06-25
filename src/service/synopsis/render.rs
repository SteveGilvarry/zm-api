//! In-process mp4 renderer for the motion synopsis (P3).
//!
//! Composites each synopsis output frame (plate + time-shifted cutouts) and
//! encodes the sequence to H.264 in an mp4 container **entirely via ffmpeg-next**
//! (libavcodec + libavformat) — zm-api never shells out to the `ffmpeg` binary.
//! All pixels come from the pre-rendered cutout/plate JPEGs (anti-recompute).

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use ffmpeg::format::Pixel;
use ffmpeg::{codec, encoder, format, frame, software, Dictionary, Packet, Rational};
use ffmpeg_next as ffmpeg;
use image::RgbImage;

use super::compositor::{self, Canvas};
use super::optimiser::SynopsisLayout;
use super::SynopsisError;
use crate::service::zmnext::detail::{TubeManifest, TubeSample};

/// Caches decoded cutout JPEGs across frames so each is read from disk once.
struct CutoutCache<'a> {
    dir: &'a Path,
    map: HashMap<String, Option<RgbImage>>,
}

impl<'a> CutoutCache<'a> {
    fn new(dir: &'a Path) -> Self {
        Self {
            dir,
            map: HashMap::new(),
        }
    }
    /// Decoded cutout for a relative path (None if missing/undecodable/unsafe).
    fn get(&mut self, rel: &str) -> Option<&RgbImage> {
        if !self.map.contains_key(rel) {
            let img =
                compositor::safe_asset_path(self.dir, rel).and_then(|p| compositor::load_rgb(&p));
            self.map.insert(rel.to_string(), img);
        }
        self.map.get(rel).and_then(|o| o.as_ref())
    }
}

/// The sample of a tube nearest a given media-clock time.
fn nearest_sample(samples: &[TubeSample], t_us: i64) -> Option<&TubeSample> {
    samples
        .iter()
        .filter(|s| !s.cutout.is_empty())
        .min_by_key(|s| (s.pts_us - t_us).abs())
}

/// Render one synopsis output frame at frame index `f`: a time-appropriate plate
/// with every active (time-shifted) tube's nearest cutout composited on.
#[allow(clippy::too_many_arguments)]
fn render_frame(
    manifest: &TubeManifest,
    asset_dir: &Path,
    shifts: &HashMap<i64, i64>,
    f: i64,
    total_frames: i64,
    uspf: i64,
    feather: u32,
    cache: &mut CutoutCache,
) -> Canvas {
    let (w, h) = (manifest.source_w, manifest.source_h);
    let synopsis_us = f * uspf;

    // Map synopsis progress linearly back to original media time for the plate,
    // so a day→night background still evolves smoothly across the condensed clip.
    let span = (manifest.t_end_us - manifest.t_start_us).max(0);
    let orig_us = if total_frames > 1 {
        manifest.t_start_us + span * f / total_frames
    } else {
        manifest.t_start_us
    };
    let mut canvas = compositor::build_plate_canvas(manifest, asset_dir, orig_us / 1000, w, h);

    for tube in &manifest.tubes {
        let Some(&shift) = shifts.get(&tube.track_id) else {
            continue; // dropped by the optimiser
        };
        let syn_start = tube.t_start_us + shift;
        let syn_end = tube.t_end_us + shift;
        if synopsis_us < syn_start || synopsis_us > syn_end {
            continue; // tube not active in this synopsis frame
        }
        // Original media time this synopsis instant maps to for the tube.
        let tube_orig_us = synopsis_us - shift;
        let Some(sample) = nearest_sample(&tube.samples, tube_orig_us) else {
            continue;
        };
        let Some(cutout) = cache.get(&sample.cutout).cloned() else {
            continue;
        };
        let alpha = compositor::rasterize_mask(
            sample.mask.as_ref(),
            sample.bbox,
            cutout.width(),
            cutout.height(),
        );
        compositor::composite_cutout(&mut canvas, &cutout, sample.bbox, &alpha, feather);
    }
    canvas
}

/// Copy a tightly-packed RGB canvas into an `RGB24` AVFrame, honouring its line
/// stride (which may be wider than `w*3`).
fn canvas_to_frame(canvas: &Canvas, dst: &mut frame::Video) {
    let w = canvas.width() as usize;
    let row_bytes = w * 3;
    let stride = dst.stride(0);
    let data = dst.data_mut(0);
    let src = canvas.as_raw();
    for y in 0..canvas.height() as usize {
        let s = &src[y * row_bytes..y * row_bytes + row_bytes];
        let d = &mut data[y * stride..y * stride + row_bytes];
        d.copy_from_slice(s);
    }
}

/// Drain finished packets from the encoder and write them to the muxer.
fn drain(
    enc: &mut encoder::video::Encoder,
    octx: &mut format::context::Output,
    ost_index: usize,
    enc_tb: Rational,
    ost_tb: Rational,
) -> Result<(), SynopsisError> {
    let mut packet = Packet::empty();
    while enc.receive_packet(&mut packet).is_ok() {
        packet.set_stream(ost_index);
        packet.rescale_ts(enc_tb, ost_tb);
        packet
            .write_interleaved(octx)
            .map_err(|e| SynopsisError::RenderFailed(format!("mux write failed: {e}")))?;
    }
    Ok(())
}

fn enc_err(stage: &str) -> impl Fn(ffmpeg::Error) -> SynopsisError + '_ {
    move |e| SynopsisError::RenderFailed(format!("{stage}: {e}"))
}

/// Render the synopsis to an H.264 mp4 at `out_path`. The parent directory must
/// exist. Fails with [`SynopsisError::EncoderUnavailable`] when no H.264 encoder
/// is built into the linked ffmpeg, and [`SynopsisError::InvalidManifest`] when
/// the manifest lacks source dimensions — never panics.
pub fn render_mp4(
    manifest: &TubeManifest,
    asset_dir: &Path,
    layout: &SynopsisLayout,
    fps: u32,
    feather: u32,
    out_path: &Path,
) -> Result<(), SynopsisError> {
    let _ = ffmpeg::init();

    let (w, h) = (manifest.source_w, manifest.source_h);
    if w == 0 || h == 0 {
        return Err(SynopsisError::InvalidManifest(
            "manifest missing source_w/source_h".to_string(),
        ));
    }
    let fps = fps.max(1);
    let uspf = 1_000_000i64 / fps as i64;
    let total_frames = (layout.length_us / uspf).max(1);

    let shifts: HashMap<i64, i64> = layout
        .placements
        .iter()
        .map(|p| (p.track_id, p.start_shift_us))
        .collect();

    // --- muxer + encoder setup ---------------------------------------------
    let codec = encoder::find(codec::Id::H264).ok_or_else(|| {
        SynopsisError::EncoderUnavailable("no H.264 encoder in linked ffmpeg".into())
    })?;

    let mut octx = format::output(&out_path).map_err(enc_err("open output"))?;
    let global_header = octx.format().flags().contains(format::Flags::GLOBAL_HEADER);

    let mut ost = octx.add_stream(codec).map_err(enc_err("add stream"))?;
    let ost_index = ost.index();

    let mut enc = codec::context::Context::new_with_codec(codec)
        .encoder()
        .video()
        .map_err(enc_err("encoder ctx"))?;
    enc.set_width(w);
    enc.set_height(h);
    enc.set_format(Pixel::YUV420P);
    enc.set_time_base(Rational(1, fps as i32));
    enc.set_frame_rate(Some(Rational(fps as i32, 1)));
    if global_header {
        enc.set_flags(codec::Flags::GLOBAL_HEADER);
    }

    let mut opts = Dictionary::new();
    opts.set("preset", "veryfast");
    opts.set("crf", "23");
    let mut enc = enc.open_with(opts).map_err(enc_err("open encoder"))?;
    ost.set_parameters(&enc);
    // The encoder's authoritative time_base after open (used for packet pts/dts).
    let enc_tb = enc.time_base();

    octx.write_header().map_err(enc_err("write header"))?;
    // The muxer finalises the stream time_base during write_header — read it now,
    // after the header, so packet timestamps rescale to the right base.
    let ost_tb = octx
        .stream(ost_index)
        .expect("output stream exists")
        .time_base();

    // RGB24 → YUV420P scaler reused for every frame.
    let mut scaler = software::scaling::Context::get(
        Pixel::RGB24,
        w,
        h,
        Pixel::YUV420P,
        w,
        h,
        software::scaling::Flags::BILINEAR,
    )
    .map_err(enc_err("scaler init"))?;

    let mut cache = CutoutCache::new(asset_dir);

    // --- frame loop ---------------------------------------------------------
    for f in 0..total_frames {
        let canvas = render_frame(
            manifest,
            asset_dir,
            &shifts,
            f,
            total_frames,
            uspf,
            feather,
            &mut cache,
        );

        let mut rgb = frame::Video::new(Pixel::RGB24, w, h);
        canvas_to_frame(&canvas, &mut rgb);

        let mut yuv = frame::Video::empty();
        scaler.run(&rgb, &mut yuv).map_err(enc_err("scale frame"))?;
        yuv.set_pts(Some(f));

        enc.send_frame(&yuv).map_err(enc_err("send frame"))?;
        drain(&mut enc, &mut octx, ost_index, enc_tb, ost_tb)?;
    }

    // Flush the encoder and finalise the container.
    enc.send_eof().map_err(enc_err("send eof"))?;
    drain(&mut enc, &mut octx, ost_index, enc_tb, ost_tb)?;
    octx.write_trailer().map_err(enc_err("write trailer"))?;
    Ok(())
}

/// Convenience: the cache path for an event's rendered synopsis mp4.
pub fn mp4_cache_path(cache_dir: &Path, event_id: u64) -> PathBuf {
    cache_dir.join(format!("event-{event_id}.mp4"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::synopsis::optimiser::{SynopsisLayout, TubePlacement};
    use crate::service::zmnext::detail::{Tube, TubeSample};

    fn tiny_manifest() -> TubeManifest {
        TubeManifest {
            source_w: 32,
            source_h: 24,
            t_start_us: 0,
            t_end_us: 1_000_000,
            tubes: vec![Tube {
                track_id: 1,
                t_start_us: 0,
                t_end_us: 1_000_000,
                samples: vec![TubeSample {
                    pts_us: 0,
                    bbox: [2, 2, 8, 8],
                    cutout: "missing.jpg".to_string(),
                    cutout_w: 8,
                    cutout_h: 8,
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        }
    }

    #[test]
    fn render_mp4_writes_a_valid_container_or_reports_no_encoder() {
        // A few black frames (the cutout is missing → skipped). Renders to a temp
        // file. Where the linked ffmpeg has an H.264 encoder we assert a real mp4
        // (`ftyp` box at offset 4); where it doesn't, we accept the graceful
        // EncoderUnavailable — never a panic.
        let manifest = tiny_manifest();
        let layout = SynopsisLayout {
            placements: vec![TubePlacement {
                track_id: 1,
                group_id: 0,
                start_shift_us: 0,
            }],
            length_us: 500_000,
            dropped: vec![],
        };
        let dir = std::env::temp_dir();
        let out = dir.join(format!("zmapi-synopsis-test-{}.mp4", std::process::id()));
        let _ = std::fs::remove_file(&out);

        let result = render_mp4(&manifest, Path::new("/nonexistent"), &layout, 10, 1, &out);

        match result {
            Ok(()) => {
                let bytes = std::fs::read(&out).expect("read rendered mp4");
                assert!(bytes.len() > 64, "mp4 has content");
                assert_eq!(&bytes[4..8], b"ftyp", "ISO-BMFF ftyp box present");
            }
            Err(SynopsisError::EncoderUnavailable(_)) => { /* ffmpeg without libx264 */ }
            Err(e) => panic!("unexpected render error: {e}"),
        }
        let _ = std::fs::remove_file(&out);
    }
}
