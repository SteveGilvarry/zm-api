//! Event videos made in-process with libav (`ffmpeg-next`), not by running
//! the ffmpeg program.
//!
//! The binary path (`ZM_PATH_FFMPEG`) and its free-text options come from
//! the Config table, so running ffmpeg let anyone who could write that table
//! choose the program and its arguments. Here nothing is run: the input is
//! demuxed (or the capture JPEGs decoded), passed through a filter graph
//! built only from numbers, encoded and muxed. The option strings are
//! reduced to an allowlist of encoder settings ([`options`]).

pub mod options;

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use ffmpeg::format::Pixel;
use ffmpeg::{
    codec, decoder, encoder, filter, format, frame, media, Dictionary, Packet, Rational, Rescale,
};
use ffmpeg_next as ffmpeg;

pub use options::EncodeOptions;

/// What the video is made from.
#[derive(Debug, Clone)]
pub enum Source {
    /// The event's recording.
    Recording(PathBuf),
    /// The capture JPEGs `{n:0digits}-capture.jpg` in `dir`, read at `fps`.
    Jpegs {
        dir: PathBuf,
        digits: usize,
        fps: f64,
    },
}

/// One video to make.
#[derive(Debug, Clone)]
pub struct Render {
    pub source: Source,
    /// Play a recording this many times faster.
    pub speed: Option<f64>,
    /// Output frame rate; `None` keeps the input's.
    pub fps: Option<f64>,
    pub width: u32,
    pub height: u32,
    pub greyscale: bool,
    pub options: EncodeOptions,
    /// Where to write. Its extension picks the container.
    pub output: PathBuf,
}

fn err(stage: &'static str) -> impl Fn(ffmpeg::Error) -> String {
    move |e| format!("{stage}: {e}")
}

/// A stored file name (`Events.DefaultVideo`) as a path inside `dir`: a
/// plain name only, so a row can't point the encoder at another file or at
/// a URL.
pub fn file_in(dir: &Path, name: &str) -> Result<PathBuf, String> {
    let plain = !name.is_empty()
        && name != "."
        && name != ".."
        && !name.starts_with('-')
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'));
    if plain {
        Ok(dir.join(name))
    } else {
        Err(format!("refusing the recording name {name:?}"))
    }
}

/// `f` as an exact-enough fraction with a small denominator. Older
/// encoders (mpeg4) refuse time bases over 16 bits.
fn rational(f: f64) -> Rational {
    let num = (f * 1000.0).round() as i32;
    let (mut a, mut b) = (num.max(1), 1000);
    let g = {
        let (mut x, mut y) = (a, b);
        while y != 0 {
            (x, y) = (y, x % y);
        }
        x
    };
    a /= g;
    b /= g;
    Rational(a, b)
}

/// The filter graph between the decoded frames and the encoder. Every value
/// is a number or a name libav gave us.
fn filter_spec(job: &Render, from_recording: bool, rate: Rational, pix: Pixel) -> String {
    let mut parts = Vec::new();
    if from_recording {
        if let Some(s) = job.speed.filter(|s| (*s - 1.0).abs() > f64::EPSILON) {
            parts.push(format!("setpts=PTS/{s}"));
        }
    }
    if job.greyscale {
        parts.push("hue=s=0".to_string());
    }
    parts.push(format!("scale={}:{}", job.width, job.height));
    // Always: the encoder then gets one frame per tick of its time base.
    parts.push(format!("fps={}/{}", rate.numerator(), rate.denominator()));
    let name = pix.descriptor().map(|d| d.name()).unwrap_or("yuv420p");
    parts.push(format!("format={name}"));
    parts.join(",")
}

/// The encoder: `-c:v` if it names a video encoder libav has, else the
/// container's default.
fn pick_encoder(job: &Render, octx: &format::context::Output) -> Result<ffmpeg::Codec, String> {
    if let Some(name) = &job.options.codec {
        return encoder::find_by_name(name)
            .filter(|c| c.medium() == media::Type::Video)
            .ok_or_else(|| format!("no video encoder named {name}"));
    }
    let id = octx.format().codec(&job.output, media::Type::Video);
    encoder::find(id).ok_or_else(|| format!("no encoder for {id:?}"))
}

/// `-pix_fmt` if the encoder takes it, else yuv420p, else the encoder's first.
fn pick_pixel(job: &Render, codec: &ffmpeg::Codec) -> Pixel {
    let supported: Vec<Pixel> = codec
        .video()
        .ok()
        .and_then(|v| v.formats())
        .map(|f| f.collect())
        .unwrap_or_default();
    let ok = |p: Pixel| supported.is_empty() || supported.contains(&p);
    if let Some(p) = job
        .options
        .pix_fmt
        .as_deref()
        .and_then(|n| n.parse::<Pixel>().ok())
        .filter(|p| ok(*p))
    {
        return p;
    }
    if ok(Pixel::YUV420P) {
        return Pixel::YUV420P;
    }
    supported[0]
}

/// The output rate the encoder will take: `want`, moved to the nearest the
/// encoder supports when it only takes a list (MPEG-1/2).
fn pick_rate(want: f64, codec: &ffmpeg::Codec) -> Rational {
    // MPEG-1/2 list eight standard rates, then extensions that only open
    // with `strict=-2` and that players may not handle.
    let standard = if matches!(codec.name(), "mpeg1video" | "mpeg2video") {
        8
    } else {
        usize::MAX
    };
    let rates: Vec<Rational> = codec
        .video()
        .ok()
        .and_then(|v| v.rates())
        .map(|r| r.take(standard).collect())
        .unwrap_or_default();
    rates
        .into_iter()
        .min_by(|a, b| {
            (f64::from(*a) - want)
                .abs()
                .total_cmp(&(f64::from(*b) - want).abs())
        })
        .unwrap_or_else(|| rational(want))
}

struct Pipeline {
    graph: filter::Graph,
    enc: encoder::Video,
    octx: format::context::Output,
    ost: usize,
    /// The filter output's time base, and the encoder's.
    sink_tb: Rational,
    enc_tb: Rational,
    frames: u64,
}

impl Pipeline {
    fn push(&mut self, f: &frame::Video) -> Result<(), String> {
        let mut src = self.graph.get("in").ok_or("no filter input")?;
        src.source().add(f).map_err(err("filter input"))?;
        self.pull()
    }

    fn pull(&mut self) -> Result<(), String> {
        let mut out = frame::Video::empty();
        loop {
            let got = self
                .graph
                .get("out")
                .ok_or("no filter output")?
                .sink()
                .frame(&mut out);
            if got.is_err() {
                return Ok(());
            }
            let pts = out.pts().map(|t| t.rescale(self.sink_tb, self.enc_tb));
            out.set_pts(pts);
            self.enc.send_frame(&out).map_err(err("encode"))?;
            self.frames += 1;
            self.drain()?;
        }
    }

    fn drain(&mut self) -> Result<(), String> {
        let ost_tb = self
            .octx
            .stream(self.ost)
            .ok_or("no output stream")?
            .time_base();
        let mut packet = Packet::empty();
        while self.enc.receive_packet(&mut packet).is_ok() {
            packet.set_stream(self.ost);
            // One tick of the encoder's time base is one frame. Without a
            // duration the container ends before the last frame.
            if packet.duration() == 0 {
                packet.set_duration(1);
            }
            packet.rescale_ts(self.enc_tb, ost_tb);
            packet
                .write_interleaved(&mut self.octx)
                .map_err(err("write"))?;
        }
        Ok(())
    }

    /// End the input at `end_pts` (input time base): the end of the last
    /// frame, so the fps filter keeps it.
    fn finish(mut self, end_pts: i64) -> Result<u64, String> {
        let mut src = self.graph.get("in").ok_or("no filter input")?;
        src.source().close(end_pts).map_err(err("filter end"))?;
        self.pull()?;
        self.enc.send_eof().map_err(err("encode end"))?;
        self.drain()?;
        self.octx.write_trailer().map_err(err("write trailer"))?;
        Ok(self.frames)
    }
}

/// Set up the filter graph, encoder and muxer for frames of `pix` at
/// `in_w`x`in_h` in time base `in_tb`, arriving at about `in_fps`.
fn pipeline(
    job: &Render,
    from_recording: bool,
    in_w: u32,
    in_h: u32,
    pix: Pixel,
    in_tb: Rational,
    in_fps: f64,
) -> Result<Pipeline, String> {
    let mut octx = format::output(&job.output).map_err(err("open output"))?;
    let codec = pick_encoder(job, &octx)?;
    let out_pix = pick_pixel(job, &codec);
    let speed = if from_recording {
        job.speed.unwrap_or(1.0)
    } else {
        1.0
    };
    let rate = pick_rate(
        job.fps.or(job.options.fps).unwrap_or(in_fps * speed),
        &codec,
    );

    let mut graph = filter::Graph::new();
    let args = format!(
        "video_size={in_w}x{in_h}:pix_fmt={}:time_base={}/{}:pixel_aspect=1/1",
        pix.descriptor()
            .map(|d| d.name())
            .ok_or("unknown pixel format")?,
        in_tb.numerator(),
        in_tb.denominator(),
    );
    graph
        .add(
            &filter::find("buffer").ok_or("no buffer filter")?,
            "in",
            &args,
        )
        .map_err(err("filter source"))?;
    graph
        .add(
            &filter::find("buffersink").ok_or("no buffersink")?,
            "out",
            "",
        )
        .map_err(err("filter sink"))?;
    let spec = filter_spec(job, from_recording, rate, out_pix);
    graph
        .output("in", 0)
        .and_then(|p| p.input("out", 0))
        .and_then(|p| p.parse(&spec))
        .map_err(err("filter graph"))?;
    graph.validate().map_err(err("filter graph"))?;
    let enc_tb = Rational(rate.denominator(), rate.numerator());
    let sink_tb = graph
        .get("out")
        .map(|mut s| s.sink().time_base())
        .unwrap_or(enc_tb);

    let global_header = octx.format().flags().contains(format::Flags::GLOBAL_HEADER);
    let mut ost = octx.add_stream(codec).map_err(err("add stream"))?;
    let ost_index = ost.index();
    let mut enc = codec::context::Context::new_with_codec(codec)
        .encoder()
        .video()
        .map_err(err("encoder"))?;
    enc.set_width(job.width);
    enc.set_height(job.height);
    enc.set_format(out_pix);
    enc.set_time_base(enc_tb);
    enc.set_frame_rate(Some(rate));
    if global_header {
        enc.set_flags(codec::Flags::GLOBAL_HEADER);
    }
    let mut opts = Dictionary::new();
    for (k, v) in &job.options.encoder {
        opts.set(k, v);
    }
    let enc = enc.open_with(opts).map_err(err("open encoder"))?;
    ost.set_parameters(&enc);
    ost.set_time_base(enc_tb);
    let mut mux = Dictionary::new();
    for (k, v) in &job.options.muxer {
        mux.set(k, v);
    }
    octx.write_header_with(mux).map_err(err("write header"))?;

    Ok(Pipeline {
        graph,
        enc,
        octx,
        ost: ost_index,
        sink_tb,
        enc_tb,
        frames: 0,
    })
}

/// Make the video. Blocking: run it on a blocking thread. `cancel` stops it
/// between frames.
pub fn render(job: &Render, cancel: &AtomicBool) -> Result<u64, String> {
    ffmpeg::init().map_err(err("init"))?;
    let frames = match &job.source {
        Source::Recording(path) => render_recording(job, path, cancel)?,
        Source::Jpegs { dir, digits, fps } => render_jpegs(job, dir, *digits, *fps, cancel)?,
    };
    if frames == 0 {
        return Err("no frames to encode".into());
    }
    Ok(frames)
}

fn render_recording(job: &Render, path: &Path, cancel: &AtomicBool) -> Result<u64, String> {
    // Local files only, and only the containers zmc records in: a file can't
    // make the demuxer fetch a URL or read a list of other files.
    let mut opts = Dictionary::new();
    opts.set("protocol_whitelist", "file");
    opts.set("format_whitelist", "mov,mp4,matroska,webm");
    let mut ictx = format::input_with_dictionary(path, opts).map_err(err("open recording"))?;
    let ist = ictx
        .streams()
        .best(media::Type::Video)
        .ok_or("the recording has no video")?;
    let ist_index = ist.index();
    let in_tb = ist.time_base();
    let in_fps = {
        let r = ist.avg_frame_rate();
        if r.numerator() > 0 && r.denominator() > 0 {
            f64::from(r)
        } else {
            25.0
        }
    };
    let mut dec: decoder::Video = codec::context::Context::from_parameters(ist.parameters())
        .and_then(|c| c.decoder().video())
        .map_err(err("decoder"))?;
    let mut p = pipeline(
        job,
        true,
        dec.width(),
        dec.height(),
        dec.format(),
        in_tb,
        in_fps,
    )?;
    let mut decoded = frame::Video::empty();
    // The end of the last frame: its pts plus one frame at the stream's rate.
    let frame_len = (f64::from(in_tb.denominator()) / f64::from(in_tb.numerator()) / in_fps)
        .round()
        .max(1.0) as i64;
    let mut end = 0i64;
    for (stream, packet) in ictx.packets() {
        if cancel.load(Ordering::Relaxed) {
            return Err("cancelled".into());
        }
        if stream.index() != ist_index {
            continue;
        }
        dec.send_packet(&packet).map_err(err("decode"))?;
        while dec.receive_frame(&mut decoded).is_ok() {
            let ts = decoded.timestamp();
            decoded.set_pts(ts);
            end = end.max(ts.unwrap_or(0) + frame_len);
            p.push(&decoded)?;
        }
    }
    dec.send_eof().map_err(err("decode end"))?;
    while dec.receive_frame(&mut decoded).is_ok() {
        let ts = decoded.timestamp();
        decoded.set_pts(ts);
        end = end.max(ts.unwrap_or(0) + frame_len);
        p.push(&decoded)?;
    }
    p.finish(end)
}

/// The capture JPEGs in order, from the first of 0-4 that exists (as
/// ffmpeg's image2 reader starts) to the first gap.
fn jpeg_paths(dir: &Path, digits: usize) -> Vec<PathBuf> {
    let name = |n: usize| dir.join(format!("{n:0digits$}-capture.jpg"));
    let Some(first) = (0..=4).find(|n| name(*n).is_file()) else {
        return Vec::new();
    };
    (first..).map(name).take_while(|p| p.is_file()).collect()
}

fn render_jpegs(
    job: &Render,
    dir: &Path,
    digits: usize,
    fps: f64,
    cancel: &AtomicBool,
) -> Result<u64, String> {
    let paths = jpeg_paths(dir, digits);
    let first = paths.first().ok_or("the event has no capture JPEGs")?;
    let first = image::open(first)
        .map_err(|e| format!("read {}: {e}", first.display()))?
        .to_rgb8();
    let (w, h) = first.dimensions();
    let fps = job.options.input_fps.unwrap_or(fps);
    let tb = {
        let r = rational(fps);
        Rational(r.denominator(), r.numerator())
    };
    let mut p = pipeline(job, false, w, h, Pixel::RGB24, tb, fps)?;
    for (i, path) in paths.iter().enumerate() {
        if cancel.load(Ordering::Relaxed) {
            return Err("cancelled".into());
        }
        let img = if i == 0 {
            first.clone()
        } else {
            match image::open(path) {
                Ok(img) => img.to_rgb8(),
                Err(e) => {
                    tracing::warn!("video: skipping {}: {e}", path.display());
                    continue;
                }
            }
        };
        if img.dimensions() != (w, h) {
            tracing::warn!("video: skipping {}: size changed", path.display());
            continue;
        }
        // A new frame each time: the filter graph takes the last one's buffers.
        let mut rgb = frame::Video::new(Pixel::RGB24, w, h);
        let stride = rgb.stride(0);
        let row = w as usize * 3;
        let data = rgb.data_mut(0);
        for (y, src) in img.as_raw().chunks_exact(row).enumerate() {
            data[y * stride..y * stride + row].copy_from_slice(src);
        }
        rgb.set_pts(Some(i as i64));
        p.push(&rgb)?;
    }
    p.finish(paths.len() as i64)
}

#[cfg(test)]
mod tests;
