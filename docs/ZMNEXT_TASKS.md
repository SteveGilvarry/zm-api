# zm-next Worker Integration

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

## Correlation & storage contract (read before going to production)

Two integration contracts are **best-effort** in this pass and need validation
against a live zm-next:

1. **Event correlation.** EVENT frames carry no event id, so ingest correlates
   per monitor as a *session*: first `detection`/`description` opens an `Events`
   row, detections append alarm `Frames` + bump scores, `recording_saved`
   finalizes it (file name, duration, end time) and closes the session. One open
   event per monitor at a time. If zm-next can run overlapping events per
   monitor, this needs revisiting.

2. **Clip path.** `Events.DefaultVideo` stores only the file name; ZoneMinder
   derives the full path from `Storage.Path` + `Scheme` + event id. zm-next's
   `store_event` therefore **must** write its clip where ZM expects it, or
   playback won't resolve. The generator points `store_event.root` at the
   monitor's storage path; the per-event subdirectory layout (event id is
   assigned by zm-api on the opening detection, not known at pipeline-generation
   time) is the open item to nail down with the fork owner — options: zm-next
   names by timestamp and zm-api stores an absolute path (needs a schema/path
   column), or zm-api moves the clip into the scheme location on
   `recording_saved`. The reported absolute path is currently logged.

## Open coordination items (ZoneMinder fork)

* **`Monitors.UseZmNext TINYINT NOT NULL DEFAULT 0`** — owned by the fork's
  migrations. Until it ships, `repo::monitors::use_zmnext` degrades to `false`
  (legacy) on the missing column, so this code is inert and safe. It activates
  automatically once the column exists. **Do not** add `UseZmNext` to the
  `monitors` SeaORM entity — that widens the base `SELECT` and breaks every
  monitor query on pre-migration databases.
* The clip-path contract above.

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
