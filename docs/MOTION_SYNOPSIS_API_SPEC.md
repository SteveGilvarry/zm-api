# Motion Synopsis — zm-api Hand-off Spec

> **Status (2026-06-28):** Active. Service skeleton exists at
> `src/service/synopsis/{compositor,optimiser,render,mod}.rs` and
> `EVENT_REVIEW_ASSETS = 0x0306` is wired in `src/streaming/source/protocol.rs`.
> P1 (composite still) + P2 (temporal optimiser scaffolding) appear in tree;
> P3 (mp4 render via libav) + P4 (range/overview, retention cleanup, soft-mask)
> still open. zm-next side is in a separate repo.

**Audience:** the zm-api (Rust/Axum) maintainer. This is the implementation contract for the
synopsis **optimiser + renderer + serving**. zm-next produces the ingredients (object "tubes" +
background plates) and announces them; zm-api consumes, optimises, renders to mp4, caches, and
serves. Design rationale and the zm-next side live in the **zm-next** repo at
`docs/Motion_Synopsis.md`. File paths below are verified against the current zm-api tree;
`zm-next`-side paths refer to the zm-next repo.

## Scope

```
zm-next  ──emits──▶  0x0306 review_assets EVENT (metadata: paths to tubes + plates)
                     + side files in the events tree (cutouts, plates, manifest.json)
                          │
zm-api owns:              ▼
  1. ingest 0x0306  (protocol.rs + service/zmnext/ingest.rs)
  2. persist a synopsis row  (SeaORM)
  3. temporal optimiser  (tube time-shift packing)
  4. composite + render mp4  (shell ffmpeg) + composite-still (P1)
  5. cache + on-demand + queued render
  6. serve  GET /events/{id}/synopsis  (ACL + auth, like events_playback)
```

**Anti-recompute rule (do not violate):** zm-api must **never re-decode the clip or re-run
detection/segmentation**. All pixels it needs are the pre-rendered cutout JPEGs + plate JPEGs
referenced by the manifest. zm-api only does compositing, time-shift optimisation, and muxing.

## 1. Input contract — the `0x0306 review_assets` EVENT

### Wire framing (recap)
Canonical stream socket, per-monitor. A 24-byte LE header
(`length,version=1,type=0x06 EVENT,stream=2 Monitor,flags,sequence,generation,pts_us`) followed
by an EVENT payload = `u16 code` + TLV tail. For this event: **`code = 0x0306`**, and the
manifest JSON is in **TLV tag `0x10` (json_detail)**, UTF-8. (`0x0305` is reserved for a future
`reasoning` event; `0x0306` is review_assets.)

### protocol.rs — add the code (additive, skip-on-unknown)
In `src/streaming/source/protocol.rs` (codes live at lines 99-116):
```rust
pub const EVENT_REVIEW_ASSETS: u16 = 0x0306; // after the existing 0x0301..0x0304
```
No parser changes: `parse_event` already skips unknown TLV tags (lines 295-336) and
`MessageType::from_u8` already returns `Option` (skip-on-unknown, lines 36-62).
`MonitorEvent.json_detail` carries the manifest exactly like `EVENT_DETECTION`.

> **zm-next guarantees** the manifest rides in TLV 0x10 (it adds `0x0306` to both `map_event_code`
> *and* `is_ai_code` in WorkerLink). If `json_detail` is empty and the text landed in the message
> tag, the zm-next side regressed — fail loudly in ingest rather than silently dropping.

### ingest — handle it (one-way; no control reply)
In `src/service/zmnext/ingest.rs` (`EventIngestor`, lines 100-161) add
`async fn handle_review_assets(monitor_id, ev: MonitorEvent)`:
1. parse `ev.json_detail` into `TubeManifest` (schema below);
2. resolve assets at `dirname(clip_path)/path_base` (`path_base` is always relative, e.g. `synopsis`);
3. upsert an `EventSynopsis` row (keyed by `event_id` when `!= 0`, else by
   `(monitor_id, clip_token)`), status `pending`, storing the manifest JSON + resolved asset dir.
No `ControlReply` is queued — `0x0306` is one-way zm-next → zm-api (unlike the
recording_opening → assign_recording handshake).

### The manifest schema (authoritative — matches zm-next byte-for-byte)
The discriminator is **`type`** (matches `WorkerLink::map_event_code`, which switches on the JSON
`type`/`event` keys — *not* `kind`).
```jsonc
{
  "type": "review_assets", "schema": 1,
  "monitor_id": 3,
  "event_id": 512,                  // 0 if the recording_opening→assign handshake didn't finish
  "clip_token": "3-1782129185-7",   // always present; fallback key when event_id == 0
  "clip_path": "/data/3/2026-06-25/512/512-video.mkv",
  "path_base": "synopsis",          // asset dir, ALWAYS relative to dirname(clip_path)
                                    //   (resolves whether clip_path is the ZM tree or own-naming)
  "t_start_us": 1782129185000000, "t_end_us": 1782129260000000,  // event media-clock span
  "source_w": 1280, "source_h": 720,   // coord space for every bbox/polygon/cutout
  "sample_fps": 4,
  "cause": "detection",             // ZM event cause (trigger type, or "continuous")
  "plates": [
    { "path": "plate-1782129180.jpg", "wallclock_ms": 1782129180000,
      "w": 640, "h": 360, "illum": "day" }     // plate at its OWN res — rescale to source_w/h
  ],
  "tubes": [
    { "track_id": 17, "label": "person", "class_id": 0,
      "t_start_us": 1782129186000000, "t_end_us": 1782129200000000,
      "samples": [
        { "pts_us": 1782129186250000, "wallclock_ms": 1782129186250,
          "bbox": [840,120,96,220],            // source coords
          "cutout": "t17/000001.jpg",          // premultiplied RGB JPEG, relative to path_base
          "cutout_w": 96, "cutout_h": 220,
          "mask": { "format": "alpha", "w": 28, "h": 64, "data": "<base64 8-bit, bbox-local>" }
          //  DEFAULT: soft per-pixel alpha (detect_seg emit_soft_mask). base64 of a w*h row-major
          //  8-bit buffer; bilinearly stretch w×h across the cutout to get the compositing alpha.
          //  FALLBACK: { "format": "polygon", "points": [[x,y], …] } (source coords) when the
          //  soft mask is unavailable — rasterise inside the cutout for a hard alpha.
        }
      ] }
  ]
}
```
Renderer notes: cutouts are **premultiplied JPEGs** (background → black). For the compositing alpha,
prefer the **soft alpha** (`mask.format=="alpha"`: base64-decode the `w*h` 8-bit buffer and
bilinearly stretch it across the cutout — this is the true per-pixel matte from detect_seg,
implemented); fall back to rasterising the **polygon** (source coords, offset by `bbox`) for a hard
alpha when only that is present. There is **no separate alpha image file and no WebP** — the soft
alpha rides compactly in the manifest. Plate `w/h` differ from `source_w/h` — **rescale the plate to
`source_w × source_h`** before compositing. Resolve all
relative paths under the event dir and **reject `..`** exactly like `resolve_event_storage_path`
(`src/handlers/events_playback.rs:231-259`).

### Robustness
- `event_id == 0`: key by `(monitor_id, clip_token)`; reconcile to the real id later if the
  EventClip row appears.
- referenced cutout/plate missing on disk (event pruned): render degrades — skip that
  tube/sample; if too few remain, fall back to the composite still; never 500 on partial data.

## 2. DB / model

Reuse the `Events` model (`src/entity/events.rs:7-84`) for the source event; add a dedicated
table for synopsis state (cleanest; avoids overloading `Events`):

`EventSynopsis` (SeaORM migration): `event_id` (FK, nullable until reconciled), `monitor_id`,
`clip_token`, `manifest_json` (Text), `asset_dir` (Text), `status`
ENUM(`pending`,`generating`,`ready`,`failed`), `rendered_path` (Text, nullable), `created_at`,
`expires_at` (drives the retention cleanup job). An optional `SynopsisAssets` child table buys
per-asset accounting; not required for MVP.

## 3. Temporal optimiser (P2)

Canonical synopsis energy over a chosen window of tubes, minimised by assigning each tube a new
start time (a **time shift** along the synopsis timeline):

```
E(M) = Σ_b Ea(b)  +  Σ_{b,b'} ( α·Et(b,b')  +  β·Ec(b,b') )
```
- **Ea** activity — prefer keeping active tubes (don't drop content).
- **Et** temporal-consistency — penalise reordering; weight strongly only for *interacting* pairs.
- **Ec** collision — penalise spatial overlap of simultaneously-shown tubes. **β is the primary
  knob**, trading synopsis length vs density.

**Expose one "condensation" control**, not raw weights: a *target synopsis length (s)* or a
*collision-area budget* (default **~5%** of tube pixels; range 2–10%); back-solve β.

**Method:** MVP = **greedy/online placement** (sort tubes by original start for chronology; place
each at the lowest-collision slot; decay β when forced to extend length) — fast, matches the
emit-ahead model. Keep **simulated annealing** as an optional offline "high quality" mode for
short windows; do **not** default to global SA.

**Preserve interactions:** group co-occurring tubes *before* rearrangement and shift each **group
as a rigid unit** (temporal overlap > ~75% AND closest-pixel distance < ~2× object width). Stops
splitting a hand-off / person+bag / two people talking across different synopsis times.

**Starting parameters** (per-camera tunable, P4): tube sample 3–5 fps; cutout max edge 128–256 px;
mask feather 1–2 px; collision budget ~5%; greedy shift granularity a few frames/violation; cap
simultaneous tubes/frame (overflow → longer synopsis or fall back to plain fast-forward when "too
crowded"). Allow **class filtering** via `class_id` (e.g. people-only synopsis).

Input: tubes for a time range (one event, or merged across events via `from`/`to`). Output:
per-tube `{start_shift, layout}` → fed to the renderer.

## 4. Rendering (P1 still, P3 video)

**Plate selection:** for each tube/frame pick the `plates[]` entry whose `wallclock_ms` is nearest
the tube's original time (time-varying background); rescale plate to `source_w × source_h`.

**Compositing:** per synopsis output frame, draw the time-appropriate plate, then for each active
tube draw its interpolated cutout (boxes interpolate between samples) using the mask as alpha. Use
a **fast alpha-matte path** for P1 (composite still) and preview; reserve **Poisson/
gradient-domain blending** for the final P3 render where seams matter. Feather the mask edge to
suppress halos. Optional per-tube timestamp label (on selection/hover, or a few at a time — avoid
clutter). The **P1 composite still reuses the same compositor** with all tubes stamped onto one
plate.

**mp4 encode — use the FFmpeg LIBRARIES, not the `ffmpeg` binary.** Do **not** shell out to the
`ffmpeg` CLI. `ffmpeg-next` (v8) is in `Cargo.toml` and already used for decode/scale
(`src/streaming/snapshot.rs:184-250`), but only its high-level wrappers; it exposes the full C API
under `ffmpeg_next::ffi`. Render composite RGB frames in Rust (e.g. the `image`/`imageproc`
crates), then encode + mux **in-process via libav**: an `AVCodecContext` H.264 encoder
(`avcodec_find_encoder(AV_CODEC_ID_H264)` / `libx264`) feeding an `AVFormatContext` mp4 muxer
(`avformat_alloc_output_context2(..., "mp4", path)`, `avformat_write_header` →
`av_interleaved_write_frame` → `av_write_trailer`), driven through `ffmpeg_next::ffi`. Wrap this in
a small `SynopsisEncoder` helper (the inverse of the existing `decode_to_jpeg_blocking`) and run it
on `spawn_blocking`. If H.264 isn't available in the linked FFmpeg, fail the render gracefully
(`status = failed`, no panic) — there is no binary fallback.

## 5. HTTP API

Add routes in `src/routes/events_playback.rs` (wrap with `media_auth_middleware`, line 22),
delegating to a new `src/service/synopsis.rs` `SynopsisService { db, cache: DashMap<…>, config }`
(mirror `SnapshotService`, `src/streaming/snapshot.rs:52-62`):

- `GET /api/v3/events/{id}/synopsis` → JSON `{ status, url, expires_at, tube_count }`. If not yet
  rendered: enqueue a render (tokio task, capped by `max_concurrent_renders`) and return
  `status:"generating"`; client polls. `render_or_get(event_id)` checks cache → DB status →
  spawns.
- `GET /api/v3/events/{id}/synopsis/mp4` → streams the cached mp4. Support **HTTP 206 range** like
  the existing `events/{id}/video`; send **ETag** (file hash) + `Cache-Control`, matching the HLS
  pattern.
- `GET /api/v3/events/synopsis?from=&to=&monitor_id=[&class=]` → range/overview synopsis across
  events (merge tubes from many manifests over a shared plate timeline).

**ACL + auth (mandatory):** every handler calls `MonitorScope::allows(monitor_id, Level)` exactly
like `events_playback` (`src/handlers/events_playback.rs:198-210`) and returns the same `NotFound`
for ACL-deny and missing event. Auth via JWT / `media_auth_middleware` (Bearer or `?token=` for
`<video>`). The monitor that grants `Events:View` grants synopsis view. Optional per-IP rate limit
(`src/routes/mod.rs:177-199`) on the render endpoints.

## 6. Per-camera enablement

Extend `ZmNextConfig` / pipeline generation (`src/configure/zmnext.rs:28-80`) with a per-monitor
`synopsis_enabled` flag. When set, the **daemon's pipeline generator must**:
1. include `detect_seg` in that camera's pipeline (masks are required);
2. configure `detect_seg` with `event_type:"detection"` **and** `mask_format:"polygon"` (NOT
   `"none"`, else no polygon is emitted and tubes degrade to bbox-only);
3. wire `tracker` after it, and `review_export` downstream of `decode_ffmpeg`;
4. enable `plate_export` on `motion_pixel_diff`.
Non-synopsis cameras pay none of this. This policy lives in zm-api (the only DB reader), consistent
with the split. Add a `[synopsis]` section to `settings/base.toml` (`enabled`, `encoder_backend`,
`render_timeout_seconds`, `max_concurrent_renders`, `retention_days`, `cache_dir`) +
`src/configure/synopsis.rs` and wire into `AppState`.

## 7. zm-api phases & acceptance

| Phase | Scope | Acceptance |
|---|---|---|
| **P1** | ingest 0x0306 + DB row + composite-still compositor + `GET …/review` | a glanceable still per event renders from the manifest; ACL enforced |
| **P2** | temporal optimiser (greedy) + interaction grouping + condensation knob | layout preview: tubes packed, no gross collisions, interactions intact |
| **P3** | render mp4 (shell ffmpeg) + cache + queue + `…/synopsis(/mp4)` | end-to-end synopsis clip; 206 ranges; ETag; concurrent-render cap honoured |
| **P4** | range/overview synopsis; retention cleanup; true-alpha (consume soft-mask side files); Poisson blend; per-camera tuning | overview across events; expired assets cleaned; quality/storage benchmarks |

## 8. Open questions for the zm-api team

- **Encoder:** confirm shell-ffmpeg for MVP vs investing in native ffmpeg-next encoder bindings.
- **Asset location/ownership:** zm-next writes assets into the **events tree** next to the clip
  (`{event_dir}/synopsis/`); zm-api reads by path. Confirm this matches your storage-scheme
  resolution (Deep/Medium/Shallow) and the traversal-rejection boundary.
- **Retention:** `expires_at` default (7 days?) and whether the hourly cleanup deletes only
  rendered mp4s (zm-api-owned) or also the source tube assets (zm-next-owned) — coordinate so
  neither side deletes the other's files out from under it.
- **Queue discipline:** FIFO (MVP) vs priority (user-triggered > background re-render).
- **Range synopsis cost:** merging tubes across many events can be large — cap window length /
  tube count and **log what was dropped** (never silently truncate).
- **Plate sharing:** plates are camera-level; copy into each event dir (simple, some dup) vs a
  per-camera plate store referenced by absolute path (less dup, more coupling).
