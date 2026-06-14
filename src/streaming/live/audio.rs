//! Live audio transcoding for WebRTC
//!
//! WebRTC has no AAC: browsers implement Opus and G.711 (PCMU/PCMA) only.
//! G.711 cameras pass through untouched, but the common AAC camera needs a
//! transcode. This module provides an ADTS-AAC → Opus transcoder built on
//! ffmpeg-next: decode AAC → resample to 48 kHz s16 → encode libopus in
//! 20 ms frames.
//!
//! Audio transcoding is cheap (well under a millisecond per frame), so one
//! transcoder per viewer session is acceptable — the per-monitor sharing in
//! docs/AUDIO_TASKS.md §3.2.2 is an optimization for many concurrent
//! viewers, not a requirement.
//!
//! The decoder relies on the ADTS header for codec parameters (the
//! stream-socket reader re-frames zmc's raw AAC as ADTS using the HELLO
//! AudioSpecificConfig); unframed AAC is rejected upstream.

use std::time::Duration;

use ffmpeg_next as ffmpeg;

/// One encoded Opus frame ready for a WebRTC `TrackLocalStaticSample`.
pub struct OpusFrame {
    pub data: Vec<u8>,
    pub duration: Duration,
}

/// Streaming ADTS-AAC → Opus transcoder.
///
/// The AAC decoder is created eagerly (it parses ADTS headers, so it needs
/// no extradata); the resampler and Opus encoder are created lazily from
/// the first decoded frame's format.
pub struct AacToOpusTranscoder {
    decoder: ffmpeg::decoder::Audio,
    resampler: Option<ffmpeg::software::resampling::Context>,
    encoder: Option<ffmpeg::encoder::Audio>,
    /// Interleaved s16 samples at 48 kHz awaiting a full encoder frame
    sample_buf: Vec<i16>,
    /// Output channel count (1 or 2, clamped from the source)
    channels: u16,
    /// Encoder frame size in samples per channel (960 = 20 ms at 48 kHz)
    frame_size: usize,
    /// Running output pts in samples
    next_pts: i64,
}

const OPUS_RATE: u32 = 48_000;

impl AacToOpusTranscoder {
    /// Create the transcoder. Fails when this ffmpeg build lacks the AAC
    /// decoder or the libopus encoder.
    pub fn new() -> Result<Self, String> {
        ffmpeg::init().map_err(|e| format!("ffmpeg init: {e}"))?;

        ffmpeg::encoder::find_by_name("libopus")
            .ok_or_else(|| "libopus encoder not available in this ffmpeg build".to_string())?;

        let codec = ffmpeg::decoder::find(ffmpeg::codec::Id::AAC)
            .ok_or_else(|| "AAC decoder not available".to_string())?;
        let decoder = ffmpeg::codec::context::Context::new_with_codec(codec)
            .decoder()
            .audio()
            .map_err(|e| format!("AAC decoder context: {e}"))?;

        Ok(Self {
            decoder,
            resampler: None,
            encoder: None,
            sample_buf: Vec::new(),
            channels: 0,
            frame_size: 0,
            next_pts: 0,
        })
    }

    /// Transcode one ADTS AAC frame; returns zero or more Opus frames.
    /// (Zero while the 20 ms output buffer is still filling.)
    pub fn transcode(&mut self, adts: &[u8]) -> Result<Vec<OpusFrame>, String> {
        let packet = ffmpeg::Packet::copy(adts);
        self.decoder
            .send_packet(&packet)
            .map_err(|e| format!("AAC decode: {e}"))?;

        let mut decoded = ffmpeg::frame::Audio::empty();
        while self.decoder.receive_frame(&mut decoded).is_ok() {
            self.ensure_pipeline(&decoded)?;
            self.resample_into_buf(&decoded)?;
        }

        self.drain_encoder()
    }

    /// Create the resampler + encoder from the first decoded frame's format.
    fn ensure_pipeline(&mut self, frame: &ffmpeg::frame::Audio) -> Result<(), String> {
        if self.encoder.is_some() {
            return Ok(());
        }

        let in_channels = frame.channel_layout().channels().max(1);
        self.channels = if in_channels >= 2 { 2 } else { 1 };
        let out_layout = if self.channels == 2 {
            ffmpeg::channel_layout::ChannelLayout::STEREO
        } else {
            ffmpeg::channel_layout::ChannelLayout::MONO
        };
        let out_format = ffmpeg::format::Sample::I16(ffmpeg::format::sample::Type::Packed);

        let resampler = ffmpeg::software::resampler(
            (frame.format(), frame.channel_layout(), frame.rate()),
            (out_format, out_layout, OPUS_RATE),
        )
        .map_err(|e| format!("resampler: {e}"))?;

        let codec = ffmpeg::encoder::find_by_name("libopus")
            .ok_or_else(|| "libopus encoder not available".to_string())?;
        let mut enc = ffmpeg::codec::context::Context::new_with_codec(codec)
            .encoder()
            .audio()
            .map_err(|e| format!("opus encoder context: {e}"))?;
        enc.set_rate(OPUS_RATE as i32);
        enc.set_format(out_format);
        enc.set_channel_layout(out_layout);
        enc.set_time_base(ffmpeg::Rational(1, OPUS_RATE as i32));
        let encoder = enc.open().map_err(|e| format!("opus encoder open: {e}"))?;

        // libopus defaults to 20 ms frames → 960 samples at 48 kHz.
        let frame_size = encoder.frame_size() as usize;
        self.frame_size = if frame_size == 0 { 960 } else { frame_size };
        self.encoder = Some(encoder);
        self.resampler = Some(resampler);
        Ok(())
    }

    /// Resample a decoded frame and append the interleaved s16 samples.
    fn resample_into_buf(&mut self, decoded: &ffmpeg::frame::Audio) -> Result<(), String> {
        let resampler = self.resampler.as_mut().expect("pipeline initialized");

        // Pre-allocate the output frame: `run` on an empty frame allocates
        // only `input.samples()` of capacity, which caps swr_convert and
        // silently buffers the rest when upsampling (16 kHz → 48 kHz would
        // deliver a third of the audio). Capacity must cover the converted
        // size plus whatever the resampler already has buffered.
        let in_rate = decoded.rate().max(1) as usize;
        let buffered = resampler
            .delay()
            .map(|d| d.output.max(0) as usize)
            .unwrap_or(0);
        let est = decoded.samples() * OPUS_RATE as usize / in_rate + buffered + 64;
        let out_layout = if self.channels == 2 {
            ffmpeg::channel_layout::ChannelLayout::STEREO
        } else {
            ffmpeg::channel_layout::ChannelLayout::MONO
        };
        let mut resampled = ffmpeg::frame::Audio::new(
            ffmpeg::format::Sample::I16(ffmpeg::format::sample::Type::Packed),
            est,
            out_layout,
        );
        resampler
            .run(decoded, &mut resampled)
            .map_err(|e| format!("resample: {e}"))?;

        if resampled.samples() == 0 {
            return Ok(());
        }
        // Packed s16: all channels interleaved in plane 0. `plane::<i16>` is
        // sized samples()*channels for packed formats via the data buffer.
        let total = resampled.samples() * self.channels as usize;
        let bytes = resampled.data(0);
        let mut samples = Vec::with_capacity(total);
        for i in 0..total {
            let lo = bytes[i * 2];
            let hi = bytes[i * 2 + 1];
            samples.push(i16::from_le_bytes([lo, hi]));
        }
        self.sample_buf.extend_from_slice(&samples);
        Ok(())
    }

    /// Encode every complete 20 ms chunk in the buffer.
    fn drain_encoder(&mut self) -> Result<Vec<OpusFrame>, String> {
        let Some(encoder) = self.encoder.as_mut() else {
            return Ok(Vec::new());
        };

        let chunk_len = self.frame_size * self.channels as usize;
        let mut out = Vec::new();

        while self.sample_buf.len() >= chunk_len {
            let chunk: Vec<i16> = self.sample_buf.drain(..chunk_len).collect();

            let out_layout = if self.channels == 2 {
                ffmpeg::channel_layout::ChannelLayout::STEREO
            } else {
                ffmpeg::channel_layout::ChannelLayout::MONO
            };
            let mut frame = ffmpeg::frame::Audio::new(
                ffmpeg::format::Sample::I16(ffmpeg::format::sample::Type::Packed),
                self.frame_size,
                out_layout,
            );
            frame.set_rate(OPUS_RATE);
            frame.set_pts(Some(self.next_pts));
            self.next_pts += self.frame_size as i64;

            let dst = frame.data_mut(0);
            for (i, s) in chunk.iter().enumerate() {
                let b = s.to_le_bytes();
                dst[i * 2] = b[0];
                dst[i * 2 + 1] = b[1];
            }

            encoder
                .send_frame(&frame)
                .map_err(|e| format!("opus encode: {e}"))?;

            let mut packet = ffmpeg::Packet::empty();
            while encoder.receive_packet(&mut packet).is_ok() {
                if let Some(data) = packet.data() {
                    out.push(OpusFrame {
                        data: data.to_vec(),
                        duration: Duration::from_micros(
                            self.frame_size as u64 * 1_000_000 / u64::from(OPUS_RATE),
                        ),
                    });
                }
            }
        }

        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::streaming::source::AdtsHeader;
    use std::path::Path;

    /// Encode `num_frames` of a sine wave to an ADTS .aac file with ffmpeg's
    /// native AAC encoder. Returns None when the encoder/muxer is missing.
    fn generate_test_adts(path: &Path, num_frames: usize) -> Option<()> {
        ffmpeg::init().ok();
        let codec = ffmpeg::codec::encoder::find_by_name("aac")?;
        let mut octx = ffmpeg::format::output(&path).ok()?;

        let mut enc_ctx = ffmpeg::codec::context::Context::new_with_codec(codec)
            .encoder()
            .audio()
            .ok()?;
        let rate = 16_000;
        enc_ctx.set_rate(rate);
        enc_ctx.set_format(ffmpeg::format::Sample::F32(
            ffmpeg::format::sample::Type::Planar,
        ));
        enc_ctx.set_channel_layout(ffmpeg::channel_layout::ChannelLayout::MONO);
        let tb = ffmpeg::Rational(1, rate);
        enc_ctx.set_time_base(tb);
        let mut enc = enc_ctx.open().ok()?;

        {
            let mut ost = octx.add_stream(codec).ok()?;
            ost.set_parameters(&enc);
            ost.set_time_base(tb);
        }
        octx.write_header().ok()?;
        let ost_tb = octx.stream(0)?.time_base();

        let frame_size = enc.frame_size() as usize; // 1024 for AAC
        let mut pts = 0i64;
        for _ in 0..num_frames {
            let mut frame = ffmpeg::frame::Audio::new(
                ffmpeg::format::Sample::F32(ffmpeg::format::sample::Type::Planar),
                frame_size,
                ffmpeg::channel_layout::ChannelLayout::MONO,
            );
            frame.set_rate(rate as u32);
            frame.set_pts(Some(pts));
            {
                let plane = frame.plane_mut::<f32>(0);
                for (i, s) in plane.iter_mut().enumerate() {
                    *s = (((pts as usize + i) as f32) * 0.05).sin() * 0.4;
                }
            }
            pts += frame_size as i64;

            enc.send_frame(&frame).ok()?;
            let mut pkt = ffmpeg::Packet::empty();
            while enc.receive_packet(&mut pkt).is_ok() {
                pkt.set_stream(0);
                pkt.rescale_ts(tb, ost_tb);
                pkt.write_interleaved(&mut octx).ok()?;
            }
        }
        enc.send_eof().ok()?;
        let mut pkt = ffmpeg::Packet::empty();
        while enc.receive_packet(&mut pkt).is_ok() {
            pkt.set_stream(0);
            pkt.rescale_ts(tb, ost_tb);
            pkt.write_interleaved(&mut octx).ok()?;
        }
        octx.write_trailer().ok()?;
        Some(())
    }

    /// Split an ADTS byte stream into frames using the header's frame_len.
    fn split_adts(stream: &[u8]) -> Vec<Vec<u8>> {
        let mut frames = Vec::new();
        let mut off = 0;
        while let Some(h) = AdtsHeader::parse(&stream[off..]) {
            let end = (off + h.frame_len).min(stream.len());
            frames.push(stream[off..end].to_vec());
            off = end;
            if off >= stream.len() {
                break;
            }
        }
        frames
    }

    #[test]
    fn transcodes_real_adts_aac_to_opus() {
        if ffmpeg::encoder::find_by_name("libopus").is_none() {
            eprintln!("libopus not in this ffmpeg build; skipping");
            return;
        }

        let dir = std::env::temp_dir().join(format!("zm_opus_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.aac");

        let Some(()) = generate_test_adts(&path, 50) else {
            eprintln!("aac encoder/adts muxer missing; skipping");
            return;
        };
        let stream = std::fs::read(&path).unwrap();
        let frames = split_adts(&stream);
        assert!(frames.len() >= 40, "expected ~50 ADTS frames");

        let mut t = AacToOpusTranscoder::new().expect("transcoder");
        let mut opus_frames = 0usize;
        for f in &frames {
            let out = t.transcode(f).expect("transcode");
            for of in out {
                assert!(!of.data.is_empty());
                assert_eq!(of.duration, Duration::from_millis(20));
                opus_frames += 1;
            }
        }
        // 50 AAC frames at 16 kHz = 3.2 s of audio → ~160 Opus 20 ms frames.
        assert!(
            opus_frames >= 120,
            "expected >=120 opus frames, got {opus_frames}"
        );

        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_dir(&dir);
    }
}
