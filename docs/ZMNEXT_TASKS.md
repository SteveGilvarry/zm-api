# zm-next Worker Integration

> **Status (2026-06-28):** Done — coord pending. Tasks 1-5 landed in zm-api.
> Outstanding cross-repo items: ZoneMinder fork's `Monitors.UseZmNext
> TINYINT NOT NULL DEFAULT 0` migration; zm-next `store` plugin's
> `recording_opening` / `assign_recording` handshake + stage-then-rename
> behavior. Until both ship, `repo::monitors::use_zmnext` returns false on
> the missing column and the integration is inert.

Status as of this branch: zm-api can drive and ingest the per-monitor `zm-next`
worker (`zm-core`) over the existing stream-socket protocol. The feature is
**off by default** and reversible per camera.

## What landed

| Task | Area | Where |
|------|------|-------|
| 1 | EVENT (0x06) + `Monitor` stream parsing | `src/streaming/source/protocol.rs`, `stream_socket.rs` |
| 1 | Router → ingest sink | `src/streaming/source/router.rs` (`MonitorEventEnvelope`, `set_event_sink`) |
| 5 | EVENT → Events/Frames ingest | `src/service/zmnext/{detail,ingest}.rs` |
| 2 | Daemon spawns/supervises the worker | `src/daemon/manager.rs` |
| 3 | Pipeline JSON generator | `src/service/zmnext/pipeline.rs` |
| 4 | Per-monitor `UseZmNext` flag (graceful) | `src/repo/monitors.rs::use_zmnext` |
| — | Config + wiring | `src/configure/zmnext.rs`, `src/server/state.rs` |

## Configuration

All under a new `[zmnext]` section; absent ⇒ disabled ⇒ zero behaviour change.

```toml
[zmnext]
enabled = true                       # master switch

[zmnext.worker]
binary = "zm-core"                   # resolved under daemon.bin_path

[zmnext.pipeline]
dir = "/var/lib/zm_api/pipelines"    # generated monitor_{id}.json files
model_path = "/var/lib/zm_api/models/yolo26n.onnx"
detect_hw = "auto"
detect_input_size = 640
detect_conf_threshold = 0.35
rtsp_transport = "tcp"
# mqtt_url = "mqtt://localhost:1883"  # optional output_mqtt stage

[zmnext.ingest]
channel_capacity = 256
event_name = "zm-next"
# default_storage_id = 1             # else lowest-id storage
idle_finalize_seconds = 0
```

The daemon controller (`[daemon].enabled`) and streaming (`[streaming].enabled`)
must be on for the worker to be spawned and its media/events consumed.

## How a monitor is routed

`DaemonManager` checks, per monitor, `[zmnext].enabled && Monitors.UseZmNext`.
When true it spawns **one** `zm-core` worker instead of `zmc`/`zma`/`zmcontrol`:

```
zm-core --monitor-id <N> --pipeline <dir>/monitor_<N>.json --socket <socks>/stream_<N>.sock
```

* Stable process-map id: `zm-core --monitor-id <N>` (paths ride as extra args).
* Stop: SIGTERM, like every other daemon.
* "Reload": regenerate the pipeline JSON from current DB rows, then restart
  (zm-next has no live reload).
* Supervision/backoff/watchdog/reconcile are the shared `ManagedProcess` paths.
* Flipping the flag tears the other daemon down on the next `start_monitor`.

## Recorder pipeline (merged `store` plugin)

zm-next folded `store_filesystem` + `store_event` into one `store` plugin with a
`mode`. The generator emits a single `store` stage and maps the monitor's
ZoneMinder function to the mode:

| ZM function | `store.mode` |
|-------------|--------------|
| `Record` | `continuous` |
| `Mocord` | `both` |
| `Modect`, `Nodect` | `event` |
| `Monitor`/`None` | `continuous` (default) |

`continuous`/`both` emit `max_secs`; `event`/`both` emit `pre_roll_sec`,
`post_roll_sec`, `max_buffer_sec`, `trigger_types` (all tunable in
`[zmnext.pipeline]`). Every segment (continuous) and every clip (event) is one
ZM event and runs the handshake below.

> **Same-filesystem requirement:** the `store` `root` and the `dir` zm-api hands
> back in `assign_recording` must be on the **same mount** — the worker renames
> the in-progress file into `dir` with an open fd, and a cross-fs target fails
> (the clip then keeps the worker's own name). The generator points `store.root`
> at the monitor's storage path, which is also what zm-api derives `dir` from, so
> they align by construction as long as the storage tree is one filesystem.

## Event-id assignment handshake (correlation + clip path)

The event id is owned by zm-api end to end and handed to the `store` plugin at
clip-open, so clips land natively in ZoneMinder's tree — no schema change, no
file relocation. The contract zm-next must implement:

1. **Open** — when the `store` plugin begins a segment it emits an EVENT (0x06),
   code **`0x0304 recording_opening`** on the Monitor stream, with
   `wall_clock_us` = start and `json_detail`:
   ```json
   { "event": "RecordingOpening", "clip_token": "<opaque handle>", "trigger": "continuous" }
   ```
   `trigger` is `"continuous"` for a continuous segment, else the trigger type
   (`"detection"`, `"motion"`, `"audio_event"`, `"tracked_detection"`, …); zm-api
   maps it to the event `Cause`.
2. **Assign** — zm-api allocates (or adopts) the `Events` row, sets `Cause` from
   `trigger`, computes the Medium-scheme location
   `{Storage.Path}/{monitor}/{YYYY-MM-DD}/{event_id}` + `{event_id}-video.mp4`,
   sets `Scheme=Medium`/`DefaultVideo`, and replies (fire-and-forget) with a
   **`0x11 Command`** (JSON) on the same connection:
   ```json
   { "cmd": "assign_recording", "clip_token": "<echoed verbatim>",
     "event_id": 512, "dir": ".../3/2026-06-22/512", "video_name": "512-video.mp4" }
   ```
   The worker **stages then renames**: record to a temp file immediately, then
   move to `dir/video_name` once the assignment arrives (before emitting
   `recording_saved`) — never blocks capture, tolerates reply latency.
3. **Save** — on close, `recording_saved` (0x0303) `json_detail`:
   ```json
   { "event": "EventClip", "event_id": 512, "path": "<final path>",
     "cause": "<cause>", "duration": 12.5, "frames": 300 }
   ```
   `duration` is **seconds**. zm-api finalizes the row by `event_id` (end time,
   frames, duration, stored video name). `event_id` is `0` (or absent) if the
   clip closed before the assignment arrived — zm-api then falls back to the open
   session, or creates a row from the clip.

ZoneMinder's playback then derives the same path; the clip is fully native.
One event is open per monitor at a time; `detection`/`description` decorate the
open row (and open one themselves if they arrive before `recording_opening`,
which then adopts it).

## Open coordination items (ZoneMinder fork / zm-next)

* **`Monitors.UseZmNext TINYINT NOT NULL DEFAULT 0`** — owned by the fork's
  migrations. Until it ships, `repo::monitors::use_zmnext` degrades to `false`
  (legacy) on the missing column, so this code is inert and safe. It activates
  automatically once the column exists. **Do not** add `UseZmNext` to the
  `monitors` SeaORM entity — that widens the base `SELECT` and breaks every
  monitor query on pre-migration databases.
* **`store` plugin handshake (above)** — emit `recording_opening`, consume
  `assign_recording` (stage-then-rename, same-filesystem `dir`), and echo
  `event_id` in `recording_saved`. This is the one zm-next change the
  integration requires.

## Per-camera migration (Task 6)

The flag makes every step reversible:

1. `UPDATE Monitors SET UseZmNext=1 WHERE Id=<N>;`
2. Reconcile (≤60 s) or `restart_monitor(<N>)`: zm-api stops legacy zmc/zma and
   spawns the zm-next worker, which opens its **own** RTSP connection (no shared
   `/dev/shm`, no DB writes from zm-next — a clean parallel run).
3. Validate parity vs. the same camera on legacy: motion/detection, recording
   continuity, live latency (WebRTC/HLS pull from the worker's stream socket
   exactly as from zmc).
4. Happy ⇒ leave it. Unhappy ⇒ `UPDATE Monitors SET UseZmNext=0` and reconcile;
   the worker is torn down and zmc/zma resume.

PTZ and two-way audio for zm-next monitors go through zm-api's ONVIF client
straight to the camera, **not** the stream socket.
