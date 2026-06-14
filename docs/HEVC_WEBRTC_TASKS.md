# HEVC over WebRTC - Implementation Tasks

Goal: stream H.265 monitors live over WebRTC to clients that can decode it —
Safari today, the native Swift/Kotlin apps next — with automatic HLS fallback
for clients that can't.

## Status Legend
- `[ ]` - Not started
- `[~]` - In progress
- `[x]` - Complete
- `[!]` - Blocked

## Current State (updated 2026-06-11)

**Phase 1 (server correctness) is implemented and unit-tested.** The
access-unit assembler, keyframe cache, and SDP fmtp are H.265-aware; what
remains is browser verification (Phase 2) onward.


More is in place than expected:

- **RTP packetization exists.** The pinned `webrtc` crate (0.17.1) ships an
  RFC 7798 `HevcPayloader`, selected automatically when a track's mime type
  is `video/H265` (`webrtc-0.17.1/src/rtp_transceiver/rtp_codec.rs:74`).
- **The offer path exists.** `WebRtcLiveManager::create_session` already
  advertises `video/H265` when the monitor codec is H265
  (`src/streaming/live/webrtc.rs:499-505`) and the handler passes the
  detected codec through (`src/handlers/live.rs:687-734`). No hard rejection
  anywhere.
- **Codec detection works.** FIFO layer detects H.265 from NAL types
  (`src/streaming/source/fifo.rs:526-529`, `h265_nal_type` at :617).
- **HLS fallback is done.** Live segmenter has full `hvc1` support
  (`src/streaming/hls/segmenter.rs:318-758`), so the fallback path for
  non-HEVC-capable browsers already ships.

The actual gaps:

1. **`AccessUnitAssembler` is H.264-only** (`src/streaming/live/webrtc.rs:62-226`):
   NAL parsing uses `h264_nal_type`, AU delimiters are SPS=7/PPS=8/AUD=9, and
   multi-slice handling parses `first_mb_in_slice`. H.265 frames fed through
   it will be mis-assembled (wrong AU boundaries, keyframe gating never
   fires).
2. **Empty H.265 fmtp line** — Safari may require RFC 7798 params
   (`profile-id`, `tier-flag`, `level-id`) to accept the m-line.
3. **Keyframe cache / startup-injection is H.264-shaped** (SPS
   `profile_level_id` extraction; keyframe = SPS+PPS+IDR). H.265 needs
   VPS+SPS+PPS (+IDR/CRA) treated as the injectable keyframe unit.
4. **Never browser-verified.**

Client-side reality (drives the phases):

- **Safari / iOS WebKit**: receives H.265 over WebRTC on Apple hardware
  (VideoToolbox decode). The primary first target.
- **Chrome/Edge**: hardware-gated HEVC receive is rolling out; treat as
  opportunistic, not a target. Dashboard must feature-detect and fall back.
- **Native apps (Swift/Kotlin)**: we control the stack, so HEVC is viable —
  libwebrtc needs an H.265-enabled build (`rtc_use_h265`), decode hardware is
  near-universal on both platforms. This is where HEVC-over-WebRTC pays off
  most: no hls.js, no MSE, just hardware decode.
- **Fallback rule**: a client that rejects the H.265 m-line answers with
  `m=video 0`; the server already errors and drops. Client falls back to HLS.

---

## Phase 1: Server Correctness — ✅ COMPLETE (code)

Implementation notes:
- `classify_nal` in `src/streaming/live/webrtc.rs` dispatches NAL typing,
  VCL detection and AU-delimiters per `FifoPacket::codec`; the H.264 path
  is byte-for-byte unchanged.
- `KeyframeCacheBuilder` in `src/streaming/source/router.rs` assembles
  SPS+PPS+IDR (H.264) or VPS+SPS+PPS+IRAP (H.265); `CachedKeyframe.
  profile_level_id` is empty for H.265 (an H.264 SDP concept, unused there).
- H.265 fmtp: `profile-id=1;tier-flag=0;level-id=120` (RFC 7798 Main
  profile); an SDP-level test asserts the offer carries `H265` + fmtp.
- Tests: H.265 AU assembly (VPS/SPS/PPS grouping, 24-slice 4K picture,
  keyframe gating, CRA, AUD), keyframe-cache builder for both codecs.

### 1.1 H.265 access-unit assembly
- [x] **1.1.1** Make `AccessUnitAssembler` codec-aware (construct with
  `VideoCodec`, or a parallel H.265 implementation — prefer whichever keeps
  the H.264 path untouched).
- [x] **1.1.2** H.265 AU-start delimiters: VPS=32, SPS=33, PPS=34, AUD=35
  (vs H.264's 7/8/9).
- [x] **1.1.3** VCL detection: H.265 VCL NAL types are 0–31; keyframe types
  are IDR_W_RADL=19, IDR_N_LP=20, CRA=21 (treat BLA 16–18 as keyframes too).
- [x] **1.1.4** Multi-slice pictures: split AUs on
  `first_slice_segment_in_pic_flag` (first bit of the slice header payload)
  — the H.265 analogue of `first_mb_in_slice == 0`. Required for 4K cameras.
- [x] **1.1.5** Unit tests mirroring the existing H.264 multi-slice 4K tests
  (`src/streaming/live/webrtc.rs:1003+`), with H.265 NAL fixtures.

### 1.2 Keyframe cache and startup injection
- [x] **1.2.1** Keyframe cache must store VPS+SPS+PPS+IDR as the injectable
  unit for H.265 monitors (decoder cannot init without VPS).
- [x] **1.2.2** Skip H.264 SPS `profile_level_id` extraction for H.265
  (currently `Option`, verify the `None` path is clean end-to-end).
- [x] **1.2.3** Cold-start path (`detect codec from source`) verified for
  H.265 monitors.

### 1.3 SDP / fmtp
- [x] **1.3.1** Populate the H.265 fmtp line per RFC 7798. Start with
  `level-id=120;profile-id=1;tier-flag=0` (Main profile — what surveillance
  cameras emit) and adjust against Safari's actual answer.
- [x] **1.3.2** Mirror the H.264 lesson (`h264_offer_profile_level_ids` doc
  comment): advertise what the *decoder* accepts, pass through the camera's
  real bitstream. Document the H.265 equivalent next to it.
- [ ] **1.3.3** Capture and commit (in this doc) a real Safari offer/answer
  SDP pair for reference.

### 1.4 Quality gates
- [x] **1.4.1** `cargo fmt` / `clippy -D warnings` / `cargo test
  --all-features` green with the new assembler tests.

---

## Phase 2: Safari Verification (first real client)

- [ ] **2.1** Manual test: H.265 monitor → dashboard in Safari (macOS first,
  then iOS). Verify picture, startup latency, 4K multi-slice.
- [ ] **2.2** Dashboard feature-detect: only attempt WebRTC for an H.265
  monitor when the browser advertises H.265 receive
  (`RTCRtpReceiver.getCapabilities('video')` contains `video/H265`);
  otherwise go straight to HLS. Avoids a guaranteed-failing negotiation
  round-trip.
- [ ] **2.3** Failure-path UX: when negotiation fails anyway (rejected
  m-line → server `error` WS message), client falls back to HLS without user
  intervention. Verify the server-side error message is actionable.
- [ ] **2.4** Re-test Chrome with HEVC hardware decode if available; record
  results here (informational, not a gate).

## Phase 3: Native Mobile Apps (Swift / Kotlin)

- [ ] **3.1** Decide the iOS WebRTC stack: libwebrtc built with
  `rtc_use_h265=true` vs a WebKit-based view. Record the decision and
  rationale here.
- [ ] **3.2** Android: libwebrtc with H.265 enabled + MediaCodec hardware
  decode capability check at runtime; fall back to HLS (ExoPlayer handles
  HEVC fMP4 natively) when absent.
- [ ] **3.3** Both apps reuse the existing WS signaling protocol (documented
  in `src/handlers/live.rs`) — token via query param, server-offer,
  trickle ICE, `ready` gate. No protocol changes expected; if any prove
  necessary, version them explicitly.
- [ ] **3.4** End-to-end test matrix: {H.264, H.265} × {iPhone, Android
  hw-decode device} over {LAN, TURN}.

## Phase 4: Hardening / Later

- [ ] **4.1** Surface negotiated codec in `/live/{id}/stats` and the `ready`
  message so clients/dashboards can display what they're actually receiving.
- [ ] **4.2** Consider RTCP feedback params (`nack`, `nack pli`) on the H.265
  capability once basic flow is proven (H.264 path currently registers none
  either — evaluate both together, separate task).
- [ ] **4.3** Document the recommended camera setup in README: dual-stream
  (H.264 substream for universal WebRTC live view, H.265 mainstream for
  recording) remains the zero-cost answer for non-HEVC clients.

## Out of Scope

- Server-side HEVC→H.264 video transcoding (CPU cost; dual-streaming and HLS
  fallback cover the need).
- HEVC over WebRTC for desktop Chrome/Firefox as a *requirement* — they get
  HLS until their HEVC support stabilizes.
