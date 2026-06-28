# Monitor Events — Capture-Fault & State Push (Spec + Tasks)

> **Status (2026-06-28):** Active. zm-api protocol side largely landed (EVENT
> 0x06 + Monitor stream parsing are in `src/streaming/source/protocol.rs`,
> `stream_socket.rs`, `router.rs` — shared with the zm-next ingest path). The
> `GET /api/v3/monitors/{id}/events` SSE endpoint (Phase 4) is **not yet
> implemented**. Gating dependency: Phase 1 zmc-side EVENT emission lives in
> a separate repo.

Adds a **server-push event stream** to zm-api, fed by a new event frame on
zmc's existing per-monitor stream socket. This is the API-first slice of
PR #4830's capabilities: the capture-fault channel (only zmc can produce it)
plus meaningful monitor state transitions, folded into **one** SSE stream.

Deliberately **not** in scope (see the #4830 comparison):
- A generic subscribe/topic/command multiplexer — REST is the command path,
  SSE is the push path. The socket never becomes a request/response bus.
- Raw RGBA/YUV image channel — deferred until a WebCodecs client or snapshot
  load justifies it.
- The *poll* half of status (shm fields on `GET /monitor-status`) — a separate
  trivial DTO-widening task, tracked at the end; it needs no protocol change.

## Design principles

1. **One push channel.** Capture faults *and* state transitions ride the same
   SSE stream. No second mechanism.
2. **Same socket, new frame.** Events ride the existing
   `{ZM_PATH_SOCKS}/stream_{id}.sock`, not a new socket. The connection stays
   up across camera reconnects, so `connection_failed` is observable precisely
   when *media* has stopped — the socket is still alive.
3. **Additive, no version bump.** The v1 framing is length-prefixed, so a new
   message type is forward-compatible: any consumer skips an unknown type by
   its `length`. A pre-event zmc simply never emits one; a pre-event consumer
   skips it. (zm-api's reader already skips unknown message types.)
4. **Snapshot on connect.** zmc sends each new consumer a `SNAPSHOT` event
   carrying current health + state — the events analogue of the media
   `KEYFRAME`-on-connect — so a late subscriber learns current status without
   waiting for the next transition.

## Wire format — EVENT frame (stream-socket protocol v1)

Reuses the 24-byte little-endian header from `docs/stream_socket.rst` /
`src/streaming/source/protocol.rs`. New assignments:

```
MessageType  0x06  EVENT
StreamId     0x02  MONITOR        (events are neither video=0 nor audio=1)
```

Header field meaning for EVENT frames:

| field        | meaning for EVENT                                                  |
|--------------|--------------------------------------------------------------------|
| `type`       | `0x06`                                                             |
| `stream`     | `0x02` (MONITOR)                                                   |
| `flags`      | reserved, 0                                                        |
| `sequence`   | per-monitor event sequence; counts every event **produced**, so a slow consumer's dropped events show as gaps (same semantics as media streams) |
| `generation` | the media stream generation in effect at emission (lets a client correlate a fault with the stream epoch; 0 if no media has started) |
| `pts_us`     | media shared-clock value at emission, or 0 if no media is flowing. Wall-clock time travels in a TLV (below), since that is what the API surfaces |

### EVENT payload

A fixed `u16` event code, then a TLV tail (`u8 tag, u16 len, value`; unknown
tags skipped — same rule as HELLO):

```
u16  event_code        (LE)
[ TLV list ]
```

Event codes:

| code     | name                      | group     |
|----------|---------------------------|-----------|
| `0x0001` | `snapshot`                | lifecycle — current health+state, sent to each consumer on connect |
| `0x0101` | `connection_failed`       | health    |
| `0x0102` | `connection_restored`     | health    |
| `0x0103` | `prime_capture_failed`    | health    |
| `0x0104` | `prime_capture_restored`  | health    |
| `0x0105` | `capture_failed`          | health    |
| `0x0106` | `capture_resumed`         | health    |
| `0x0201` | `state_changed`           | state     |

TLV tags (all optional except where a code needs one):

| tag    | name             | type  | notes                                            |
|--------|------------------|-------|--------------------------------------------------|
| `0x01` | `wall_clock_us`  | u64   | unix-epoch microseconds; the API timestamp       |
| `0x02` | `message`        | utf8  | human-readable cause/detail                      |
| `0x03` | `state_id`       | u32   | current state (required for `state_changed`/`snapshot`) |
| `0x04` | `prev_state_id`  | u32   | for `state_changed`                              |
| `0x05` | `detail`         | u32   | errno / ffmpeg error code for `*_failed`         |
| `0x06` | `state_name`     | utf8  | "Idle"/"PreAlarm"/"Alarm"/"Alert" (API may also map from id) |

`snapshot` carries at minimum `state_id` + `wall_clock_us`, plus the latest
health condition if the monitor is currently faulted (e.g. code accompanied by
a `message`). It is the consumer's authoritative "current status" on connect
and after any gap.

## zm-api integration

### `src/streaming/source/protocol.rs`
- `MessageType::Event = 0x06`, `StreamId::Monitor = 0x02`.
- `event_code` constants + `MonitorEvent` struct
  (`code`, `name`, `message`, `state_id`, `prev_state_id`, `detail`,
  `wall_clock_us`, `generation`).
- `parse_event(payload) -> Result<MonitorEvent, ProtocolError>` (u16 + TLV).
- Unit tests on the encode/decode round-trip (extend `test_encode`).

### `src/streaming/source/stream_socket.rs`
- New `SocketEvent::Event(MonitorEvent)`.
- `handle_message`: route `MessageType::Event` → `parse_event` → push.
  No pts normalization (events use wall-clock from TLV).
- Cache the last `snapshot`/health event per connection for connect-time
  replay to new SSE subscribers (parallel to the keyframe cache).
- Fake-zmc test: HELLO → snapshot event → media → capture_failed → restored.

### `src/streaming/source/router.rs`
- `MonitorSource` gains `event_tx: broadcast::Sender<MonitorEvent>` and a
  `watch<Option<MonitorEvent>>` holding the latest health snapshot
  (mirrors `keyframe_cache`). `subscribe_events()` + `current_event()`.
- Reader task publishes `SocketEvent::Event` to `event_tx` and updates the
  snapshot watch.
- **Reaper integration:** a source with live event subscribers must not be
  reaped. The idle watchdog currently keys off HLS `last_access`; extend the
  "is idle" test so an active event subscriber (or a non-zero event
  `receiver_count`) keeps the reader connected even with no media session.
  This is what makes faults observable while nobody is watching video.

### SSE endpoint — `GET /api/v3/monitors/{monitor_id}/events`
- **Auth:** `monitor_path_guard` (View) + `media_auth_middleware` — accept the
  JWT in the `Authorization` header *or* `?token=` query, because browser
  `EventSource` cannot set headers (same pattern as HLS media routes).
- **Acquire:** `source_router.get_source(monitor_id)` (starts/keeps the reader
  connected) + `subscribe_events()`. Holds the source for the connection
  lifetime so the reader stays up while ≥1 subscriber is connected.
- **Response:** `axum::response::sse::Sse` over the broadcast receiver.
  - First frame: the cached `snapshot` (current status) so the client syncs
    immediately.
  - Per event → SSE frame:
    ```
    event: connection_failed
    id: 42
    data: {"monitor_id":3,"code":"connection_failed","message":"…",
           "generation":4,"wall_clock":"2026-06-14T09:11:43.501Z",
           "state_id":null,"state_name":null}
    ```
    `event:` = event name (so `addEventListener('connection_failed', …)`
    works); `id:` = the header `sequence` (enables `Last-Event-ID`); `data:` =
    JSON.
  - **Keep-alive:** `KeepAlive` comment every 15s (liveness to the browser;
    independent of zmc's 5s STATS).
  - **Lag:** on `broadcast::RecvError::Lagged`, emit a fresh `snapshot`
    (re-sync current state) and continue — gaps in `id` already signal loss.
- **Resume:** the broadcast is ephemeral (no replay), but `snapshot`-on-connect
  re-establishes current state after any reconnect or gap. `Last-Event-ID` is
  accepted and logged; not used for replay in v1.

### Out of scope here (cross-refs)
- **Firehose** `GET /api/v3/events/stream` (all permitted monitors, multiplexed,
  ACL-filtered) — a follow-up once the per-monitor stream is proven.
- **Poll-half of status** — widen `GET /api/v3/monitor-status/{id}` with the shm
  fields already read in `src/zm_shm.rs` (`analysing`/`capturing`/`recording`,
  `last_frame_score`, `signal`, `heartbeat_time`). No protocol change; do
  anytime.

## Open questions for the zmc side

1. **Same socket** for events (recommended) vs a separate control socket?
   Same socket keeps it one connection and lets faults flow while media is
   stalled.
2. Can zmc emit a **`snapshot` event to each new consumer on connect**, like it
   already does for the cached `KEYFRAME`?
3. The #4830 branch already tracks these transitions
   (`connection_failed`/`prime_capture_*`/`capture_*`). Can they be emitted on
   `stream-socket-core`'s socket as EVENT frames with minimal change, rather
   than re-deriving them?
4. Confirm **event `sequence` is per-monitor, monotonic, gaps-observable** —
   same contract as the media streams.
5. **Wall-clock source** for `wall_clock_us` (CLOCK_REALTIME at emission).

## Phased tasks

- **Phase 1 — zmc (separate repo/agent).** Add `EVENT` message type; emit the
  six health transitions + `state_changed`; send `snapshot` on consumer
  connect. No protocol version bump.
- **Phase 2 — zm-api protocol.** `protocol.rs`: types, `MonitorEvent`,
  `parse_event`, encode/decode tests.
- **Phase 3 — zm-api source.** `stream_socket.rs` `SocketEvent::Event` +
  snapshot cache; `router.rs` event broadcast + snapshot watch +
  `subscribe_events`; reaper keeps event-subscribed sources alive. Fake-zmc
  integration test.
- **Phase 4 — zm-api endpoint.** `GET /monitors/{id}/events` SSE: auth (header
  or `?token=`), connect-snapshot, keep-alive, lag re-sync. Handler + route
  tests.
- **Phase 5 (optional).** Firehose `GET /events/stream`.
- **Side task (independent).** Widen `monitor-status` DTO with shm fields.
