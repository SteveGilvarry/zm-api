# Live Audio - Implementation Tasks

> **Status (2026-06-28):** Done — verify. Phases 1-3 (HLS AAC pass-through,
> WebRTC G.711 pass-through, AAC→Opus transcode) implemented and unit-tested.
> Open: 2.2.2 browser/camera verification, Phase 4 stretch (G.711→AAC,
> VOD audio). The Phase 4 "raw-AAC ASC recovery" stretch item is obsolete —
> stream-socket HELLO now carries the ASC.

This document tracks implementation tasks for live audio support in HLS and
WebRTC streaming. Recorded-event (VOD) audio is a stretch phase at the end.

> **2026-06-12 transport note:** the audio FIFO described throughout this
> document was replaced by zmc's per-monitor stream socket
> (`{ZM_PATH_SOCKS}/stream_{id}.sock`, ZoneMinder `feature/stream-socket-core`).
> `ZmAudioFifoReader`/`SharedPtsBase` are gone — both streams now arrive on one
> connection with a shared pts clock, and AAC arrives raw with its
> AudioSpecificConfig in the HELLO handshake (`src/streaming/source/
> stream_socket.rs` re-frames it as ADTS, so everything downstream of the
> reader described here is unchanged). The Phase 4 "raw-AAC ASC recovery"
> stretch item is obsolete. FIFO references below are historical.

## Status Legend
- `[ ]` - Not started
- `[~]` - In progress
- `[x]` - Complete
- `[!]` - Blocked

## Current State (2026-06-11)

**Phases 1–3 are implemented and unit-tested.** What remains is browser/
player verification against a real camera (2.2.2, 3.3.x manual checks) and
the Phase 4 stretch items.

Key facts established from the ZoneMinder source (`zm_fifo.cpp`,
`zm_monitor.cpp`, `zm_packet.h`) that **superseded the original plan**:

- The audio FIFO uses the **same `ZM <byte_count> <pts>\n` framing** as
  video (`Fifo::writePacket`). No ADTS sync-scanning or fixed-duration
  chunking is needed for framing — one ZM packet is one `AVPacket`.
- The framing `pts` is in **AV_TIME_BASE_Q (microseconds) on the same clock
  for both streams** (`zm_packet.h`: "MUST be in AV_TIME_BASE_Q"). A/V sync
  therefore comes from a **shared pts base** per monitor
  (`SharedPtsBase` in `src/streaming/source/audio_fifo.rs`), not from
  arrival-time heuristics. Normalizing each stream against its own first
  packet would skew A/V by the FIFO backlog.
- ZoneMinder writes the **raw AVPacket with no ADTS bitstream filter**, and
  (unlike video, which writes extradata ahead of keyframes) **never writes
  the audio extradata** (AudioSpecificConfig) to the FIFO. ADTS-framed AAC
  is self-describing and fully supported; raw AAC (typical RTSP
  depacketizer output) cannot be described in an fMP4 `esds` or decoded
  without out-of-band parameters, and is skipped with a one-shot warning.
  **Verify against a real camera** which form ZoneMinder actually emits:
  `sudo timeout 3 dd if=/run/zm/audio_fifo_1.aac bs=4096 count=4 status=none | xxd | head`
  (ADTS starts with sync bytes `ff f1`/`ff f9` after the `ZM …\n` header).

## Codec Strategy (the cross-over)

| Camera codec | → HLS                  | → WebRTC                       |
|--------------|------------------------|--------------------------------|
| AAC (ADTS)   | pass-through (mux)     | transcode → Opus (ffmpeg-next) |
| AAC (raw)    | skipped (no ASC)       | skipped (no decoder params)    |
| G.711 a/µ    | transcode → AAC (P4)   | pass-through (PCMA/PCMU)       |

Audio transcoding is cheap (well under a millisecond per frame); video
transcoding is explicitly out of scope.

---

## Phase 1: Audio Source Pipeline — ✅ COMPLETE

`src/streaming/source/audio_fifo.rs` (new), `router.rs`, `fifo.rs`.

- [x] **1.1.1** `ZmAudioFifoReader` mirrors the video reader: `O_RDWR |
  O_NONBLOCK` open, epoll reads via `AsyncFd`, reuses `parse_zm_frame` /
  `resync_zm`. Router spawns it in `start_reader`, aborts in `stop_reader`.
- [x] **1.1.2** Framing comes from ZM headers; multi-frame ADTS payloads are
  split with per-frame timestamp spacing (`split_adts_frames`).
- [x] **1.1.3** Timestamps from ZM pts, normalized via the shared
  `SharedPtsBase` (video reader resets it on each FIFO (re)open; both
  streams re-base together). `AdtsHeader` parser provides per-frame
  durations and the 2-byte AudioSpecificConfig.
- [x] **1.1.4** Packets broadcast on `MonitorSource.audio_tx`.
- [x] **1.1.5** Audio reader aborted with the video reader; backlog
  (up to 64 KiB of stale audio) drained and discarded on open.
- [x] **1.2.1** `audio_packets` counter on `MonitorSource` / `SourceStats`.
- [x] **1.2.2** `has_audio` / `audio_subscribers` now live values.
- [x] **1.3.x** Sync strategy: shared µs clock from ZM (see Current State).
- [x] **1.4.x** Tests: ADTS parse/reject/ASC, frame splitting + timestamps,
  shared-base normalization, real-named-pipe reader test, end-to-end router
  broadcast test.

## Phase 2: HLS Live Audio (AAC pass-through) — ✅ COMPLETE (code)

`src/streaming/hls/segmenter.rs`, `session.rs`, `live/coordinator.rs`.

- [x] **2.1.1** ASC built manually from the ADTS header (no BSF needed).
- [x] **2.1.2** Audio `trak` (track 2: tkhd/mdhd/smhd/stbl with
  `mp4a`+`esds`) + second `trex`; audio timescale = sample rate, so one AAC
  frame is exactly 1024 ticks.
- [x] **2.1.3** ADTS stripped; raw AAC frames in mdat, constant 1024-tick
  sample durations, per-fragment `tfdt` re-anchoring.
- [x] **2.1.4** Separate audio `traf` per `moof`; video bytes first in mdat,
  audio appended; data offsets computed for both trafs.
- [x] **2.2.1** Playlist CODECS: `avc1.…,mp4a.40.{aot}` when audio is muxed.
- [ ] **2.2.2** Manual check: hls.js + Safari play muxed A/V from a real
  camera (needs a live ZM source with ADTS audio).
- [x] **2.3.1** No new config knob: audio rides along whenever the monitor
  has an audio FIFO (absence = video-only, as before). Init-segment
  generation waits (bounded, ~100 video samples) for audio params when the
  source has an audio FIFO, then falls back to video-only.
- [x] **2.3.2** Monitors without audio FIFOs unchanged (regression-tested).
- [x] **2.4.x** Tests: init-with-audio boxes, video-only fallback deadline,
  two-traf segment with verified audio trun data_offset, non-ADTS skip,
  late-audio drop.

## Phase 3: WebRTC Live Audio — ✅ COMPLETE (code)

`src/streaming/live/audio.rs` (new), `webrtc.rs`, `handlers/live.rs`.

- [x] **3.1.1** Audio `TrackLocalStaticSample` (sendonly) added in
  `create_session` via `AudioTrackKind` (Pcmu | Pcma | Opus).
- [x] **3.1.2** PCMA (PT 8) and PCMU (PT 0) registered per kind.
- [x] **3.1.3** G.711 pass-through with byte-accurate durations
  (8 kHz, 1 byte/sample).
- [x] **3.2.1** `AacToOpusTranscoder`: AAC decode (ADTS self-describing) →
  swresample to 48 kHz s16 (output frame pre-allocated — ffmpeg-next's
  `run` otherwise caps output at the *input* sample count and silently
  buffers ⅔ of upsampled audio) → libopus 20 ms frames. Missing libopus
  downgrades the offer to video-only with a warning.
- [x] **3.2.2** *Deviation from plan:* transcoder is **per-session**, not
  per-monitor — `WebRtcLiveManager` is per-connection, and audio transcode
  is <1% of a core. Revisit only if many concurrent viewers per monitor
  become a real workload.
- [x] **3.2.3** Opus track fed from the transcoder in the handler loop.
- [x] **3.3.1** Audio m-line only for audio-capable monitors
  (SDP-level tests cover audio/no-audio/G.711 offers).
- [x] **3.3.2** A/V sync foundation: shared pts base + sample durations.
  Lip-sync validation against a real camera still pending (manual).
- [x] **3.3.3** `ready` message now carries `has_audio`; WS protocol doc
  comment updated.
- [x] **3.4.1** SDP tests: audio m-line present/absent per session kind.
- [x] **3.4.2** Real AAC→Opus round-trip test (skips when libopus absent).

## Phase 4: Stretch / Later

- [ ] **4.1** G.711 → AAC transcode so G.711 cameras get HLS audio too.
- [ ] **4.2** VOD audio: check whether ZM event MP4s carry audio and extend
  `src/streaming/hls/vod.rs` to expose the audio track during event playback.
- [ ] **4.3** Raw (non-ADTS) AAC support: recover the AudioSpecificConfig
  out-of-band — e.g. probe the monitor's most recent event MP4 (it carries
  the esds) or the camera's RTSP stream directly.
- [ ] **4.4** Dashboard handoff doc: audio is muted by default in browsers
  until user gesture (`autoplay` policies) — client must surface an unmute
  control. The WS `ready.has_audio` flag tells the client when to show it.

## Operational Prerequisite

ZoneMinder only creates `audio_fifo_*` when the monitor has `record_audio`
enabled and an RTSP server stream id (`zm_monitor.cpp:3395-3404`). Document
the required monitor settings before browser verification.
