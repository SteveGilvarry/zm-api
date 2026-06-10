# Review Fixes Implementation Plan

Source: full API review (2026-06-10) — three parallel code reviews (streaming, HTTP layer,
service/repo) plus a WebRTC startup-latency trace. Every finding below was verified against
the code before inclusion. Line numbers were correct at plan time; re-locate by symbol if
they have drifted.

## How to work this plan

- Follow the repo TDD workflow (CLAUDE.md): write/adjust the failing test first, then the
  smallest fix.
- Quality gates after every phase, all must pass:
  ```bash
  cargo fmt --all
  cargo clippy --all-targets --all-features -- -D warnings
  cargo test --all-features
  ```
- If a phase touches DB-facing code, also run:
  `APP_PROFILE=test-db cargo test --test '*' -- --include-ignored`
  (start DB first: `./scripts/db-manager.sh start && ./scripts/db-manager.sh mysql`)
- Commit per phase (or per numbered item for Phase 1), directly to `master`.
- When all phases are done, run `cargo build --release` once.
- Guardrail: small targeted changes only. Do NOT refactor streaming code beyond what each
  item specifies. Use Context7 MCP for authoritative Axum/SeaORM/webrtc-rs API details
  rather than guessing.

## Verified NON-issues — do not "fix" these

These were review claims that turned out to be false positives. Leave them alone:

1. **HLS live endpoints are NOT unauthenticated.** `add_live_routes` is wrapped by
   `protect(Feature::Stream)` plus `monitor_path_guard` in `src/routes/mod.rs:298-304`.
   Only the utoipa `security` annotations are missing (fixed in Phase 5).
2. **LL-HLS `wait_for_segment` has NO off-by-one.** `segmenter.sequence()` returns the
   *next* segment number (incremented after completion, `segmenter.rs:983-990`), so the
   `sequence() > target_sequence` check at `src/streaming/hls/session.rs:375` is correct
   and consistent with `seq >= target_sequence` at line 391.
3. **User listing is NOT open to all authenticated users.** User routes are wrapped in
   `protect(Feature::System)` at `src/routes/mod.rs:321`.
4. **`repo::users::update` does not handle passwords** (only email/enabled), so only the
   create path needs hashing (Phase 1.1).
5. **`src/streaming/webrtc/{engine,session}.rs`** ("native WebRTC Phase 2") are wired into
   `AppState` but not used by the live WS path. Leave them in place — planned future work,
   protected by the CLAUDE.md streaming guardrail.

---

## Phase 1 — Critical security & correctness (do first)

### 1.1 Hash passwords on user creation (CRITICAL)

**Bug:** `POST /api/v3/users` stores the plaintext password. `repo::users::create`
(`src/repo/users.rs:80`) does `password: Set(req.password.clone())` and nothing in the
chain (`handlers/users.rs::create_user` → `service::users::create` at
`src/service/users.rs:48`) hashes it. The bcrypt login verifier then always rejects these
users, so they can never log in. Both a security hole and a functional bug.

**Fix:** In `service::users::create`, hash before calling the repo. The util already
exists: `pub async fn hash(password: String) -> AppResult<String>` in
`src/util/password.rs:7` (it spawn_blocks bcrypt internally — check and keep that
behavior).

```rust
// service/users.rs create():
let hashed = crate::util::password::hash(req.password.clone()).await?;
let req = CreateUserRequest { password: hashed, ..req };
let model = repo::users::create(state.db(), &req).await?;
```

**Tests:** Unit test in `service/users.rs` (mock DB tests already exist there — follow
their pattern) asserting the value passed to the repo / stored model starts with `$2`
(bcrypt prefix) and is not the plaintext. If there is an `#[ignore]` DB integration test
for user creation, extend it: create user → login with that password succeeds.

### 1.2 Fix HTTP status-code mappings (one-liners)

File: `src/error/mod.rs`.

- Line ~274: `ConflictError` → change `StatusCode::INTERNAL_SERVER_ERROR` to
  `StatusCode::CONFLICT` (409). This variant is used by `start_live_stream` for
  duplicate sessions; the endpoint's OpenAPI already documents 409.
- Line ~268: `InvalidSessionError` → change `StatusCode::BAD_REQUEST` to
  `StatusCode::UNAUTHORIZED` (401) so clients know to re-authenticate.
- Line ~338-343: `TypeHeaderError` (malformed Authorization header) → change
  `StatusCode::INTERNAL_SERVER_ERROR` to `StatusCode::BAD_REQUEST` (400).

**Tests:** Add/extend a unit test in `error/mod.rs` that maps each variant via
`response_tuple()`/`IntoResponse` and asserts the status codes. Check whether any
existing test asserts the old wrong codes and update it.

### 1.3 Row-level ACL on event playback endpoints (CRITICAL)

**Bug:** The playback handlers in `src/handlers/events_playback.rs` (playlist, video,
thumbnail, info, segments — every handler that resolves an event) fetch via
`get_event_entity` (`events_playback.rs:186`) with no `MonitorScope` check. Routes get
feature-level `protect(Feature::Events)` only (`src/routes/mod.rs:235-238`). A user
restricted to specific monitors can stream any event's video by ID.

**Fix:** Mirror what `get_event` / `service::events::get_by_id` does (read it first and
copy its exact semantics — it returns NotFound for out-of-scope rows to avoid an
existence oracle):

1. Add `scope: MonitorScope` as an extractor parameter to each playback handler.
   `MonitorScope` lives in `src/service/monitor_acl.rs` (API: `allows(monitor_id,
   Level)`, `is_restricted()`). Confirm it implements `FromRequestParts` and works on
   these routes — the playback routes use `media_auth_middleware` (token may arrive via
   `?token=`). Verify `media_auth_middleware` inserts the same request extension
   (claims) that the `MonitorScope` extractor reads; if it does not, fix the extractor
   path so it does, do not weaken the middleware.
2. Change `get_event_entity` to take the scope and enforce it after the fetch:
   `if !scope.allows(event.monitor_id, Level::View) { return Err(NotFoundError(...)) }`
   (same NotFound shape it already builds). Since all handlers go through
   `get_event_entity`, this covers every endpoint in the file. Make the scope parameter
   non-optional so future handlers can't forget it.

**Tests:** Unit/integration test: user with restricted scope (e.g. allowed monitor 5)
requesting playback info for an event on monitor 20 gets 404; same request with
unrestricted scope succeeds. Follow the pattern of existing ACL tests from commit
f5f599e (look in `tests/` and `handlers/events.rs` tests for scope-test scaffolding).

### 1.4 Row-level ACL on `create_monitor`

**Bug:** `create_monitor` (`src/handlers/monitor.rs:101`) is the only mutating monitor
handler without a `MonitorScope` extractor. Feature-level `Monitors:Edit` still gates it,
but a row-restricted user can create monitors.

**Fix:** Add `scope: MonitorScope` and reject restricted users:
`if scope.is_restricted() { return Err(AppError::ForbiddenError(...)) }` (use whatever
forbidden variant the other monitor handlers use — check `update_monitor` for the
convention). Rationale: monitor creation is a whole-system operation; restricted users
shouldn't do it.

**Tests:** restricted scope → 403; unrestricted + `Monitors:Edit` → 201.

---

## Phase 2 — WebRTC startup latency

User-reported symptom: stream sometimes takes many seconds to start. Verified causes, in
payoff order. The WS handler flow lives in `src/handlers/live.rs`
(`webrtc_websocket_handler` → `handle_webrtc_websocket`, ~line 599 onward); the per-
connection manager is `WebRtcLiveManager` in `src/streaming/live/webrtc.rs`.

### 2.1 Re-read the keyframe cache at DTLS-connected time (biggest win, smallest change)

**Bug:** `handle_webrtc_websocket` reads `source.cached_keyframe()` ONCE before signaling
starts (`live.rs:655`, variable `cached`). Seconds later, when the peer connection
reaches Connected (`live.rs:821-836` and the second injection site at ~`live.rs:902-918`),
it injects that stale snapshot. On a cold start it captured `None`, even though the
reader task has almost certainly cached a keyframe during signaling
(`src/streaming/source/router.rs:457-484`). Downstream, `AccessUnitAssembler` drops all
access units until a keyframe (`webrtc.rs:172-190`), so the viewer waits a full GOP
(1–8s+ depending on camera).

**Fix:** At both Connected-branch injection sites, call `source.cached_keyframe()` fresh
instead of using the captured variable. If still `None`, subscribe to
`source.subscribe_keyframe_cache()` (watch channel, `router.rs:161`) and inject on the
first `Some` — bounded by a short timeout (~10s) so a dead source doesn't hang the task.

**Tests:** The streaming code has unit tests with synthetic sources — add one that
populates the keyframe cache *after* session setup but *before* the connected event and
asserts the keyframe is injected. If the existing test harness can't express that
cheaply, test the extracted helper function (e.g. a
`resolve_startup_keyframe(source, timeout)` helper) in isolation.

### 2.2 Trickle server ICE candidates; stop blocking on full gathering; honor configured STUN servers

**Bug A:** `create_session` awaits `gathering_complete_promise()` before returning the
offer (`src/streaming/live/webrtc.rs:644`). webrtc-ice 0.17 has a hard 5s STUN gather
timeout, so on hosts with no internet/filtered UDP every connection stalls ~5s before
the offer is even sent. This is the main "sometimes fast, sometimes slow" cause.

**Bug B:** The handler builds `WebRtcLiveConfig::default()` (`live.rs:641`) whose STUN
list is hardcoded to Google STUN (`webrtc.rs:357-360`). The existing
`[streaming.webrtc] stun_servers` setting (`src/configure/streaming.rs:118`) is ignored
on this path.

**Fix:**
1. Plumb config: in `handle_webrtc_websocket`, build `WebRtcLiveConfig` from
   `state.config().streaming.webrtc` (stun_servers at minimum). Empty list must be valid
   (LAN-only: host candidates suffice and gathering completes in ms).
2. Trickle: remove the `gathering_complete_promise().await`; send the offer immediately
   after `set_local_description`. Register `peer_connection.on_ice_candidate` and forward
   each candidate to the client as a WS message. The WS protocol already documents an
   `icecandidate` message type (see protocol doc comment near `live.rs:576`) and the
   client→server direction already works (`live.rs:862-867`) — implement the
   server→client direction with the same JSON shape. Mechanics: the callback can't write
   to the WS sink directly; create a `tokio::sync::mpsc::unbounded_channel`, send
   candidates (serialized `RTCIceCandidateInit` from `candidate.to_json()`) from the
   callback, and add the receiver as a branch in the handler's existing `select!`/message
   loop, forwarding to the WS sink. A `None` candidate from the callback means gathering
   finished — optionally forward as `{"type":"icecandidate","candidate":null}`.
3. Check the web client (static assets / whatever consumes this WS protocol in this repo,
   if present) handles server `icecandidate` messages; if a bundled JS client exists and
   ignores them, update it to call `pc.addIceCandidate`.

Use Context7 for exact webrtc-rs (`webrtc` crate) callback signatures — do not guess.

**Tests:** Unit-test the config plumbing (WebRtcLiveConfig built from settings reflects
stun_servers, including empty). For trickle, test whatever pure pieces exist (message
serialization of the icecandidate WS frame). Full ICE integration is not unit-testable
here; verify manually with `cargo run` + a browser if available.

**STATUS (implemented):** Part 1 (config plumbing, Bug B) is **done** —
`WebRtcLiveConfig::from_webrtc_config` maps `[streaming.webrtc]` stun_servers/turn/
max_connections, and the live WS handler uses it instead of `default()`. An empty
`stun_servers` list is honored, so a LAN deployment can set `stun_servers = []` to make
ICE gathering complete in milliseconds and eliminate the ~5s STUN-gather stall. Unit
tests cover empty-list, enabled-TURN, and disabled-TURN mapping.

**DEFERRED:** Part 2 (server-side trickle ICE — removing `gathering_complete_promise`,
`on_ice_candidate` → WS `icecandidate`) and Part 3 (JS client). These change the
SDP/ICE flow and the client protocol and can silently break *all* WebRTC if wrong; they
require browser verification that the current environment can't provide. With the config
plumbing in place, LAN operators already have a zero-risk path to remove the stall
(empty STUN). Revisit trickle when a browser test loop is available. Bug A's
`gathering_complete` wait is intentionally left as-is: for remote (non-LAN) peers it is
load-bearing (srflx candidates), and bounding it without trickle would regress them.

### 2.3 Take SPS/profile detection off the critical path

**Bug:** Cold path sleeps 100ms then waits up to 5s for an SPS NAL (`live.rs:671-711`)
*before* peer-connection setup — yet the SDP offer advertises fixed H.264 profiles
regardless (`webrtc.rs:401-403`, `42e01f`/`640c1f`). The detected profile is effectively
unused. This serializes a GOP-wait in front of ICE.

**Fix:** Remove the SPS-wait block from the startup path. The only real decision it
feeds is H.264 vs H.265 — derive that from the FIFO codec instead: ZoneMinder FIFOs are
named `/run/zm/video_fifo_{id}.{codec}` and the source layer already knows the codec
(check `MonitorSource`/`SourceRouter` for an existing codec accessor; `fifo.rs` parses
the extension). If a codec accessor genuinely doesn't exist, add a small one to
`MonitorSource` populated from the FIFO path. Keep the fixed-profile offer.

**Tests:** Update/remove tests that exercised the SPS-wait; add a test for codec
derivation from FIFO extension if you add the accessor.

### 2.4 (Optional, only if 2.1–2.3 land cleanly) Pre-warm readers

Skip unless trivial: on startup, optionally start FIFO readers for monitors whose FIFO
exists, so first viewers never hit the cold path. Behind a config flag, default off.
If in doubt, leave for a future PR.

---

## Phase 3 — HLS session lifecycle

### 3.1 Actually start the cleanup task, and make it stoppable

**Bug:** `HlsSessionManager::start_cleanup_task` (`src/streaming/hls/session.rs:441`,
delegating to `HlsStorage::start_cleanup_task` at `storage.rs:488`) has ZERO callers —
the time-based segment eviction loop never runs. Only the inline cap-eviction inside
`store_segment` runs (and only while packets flow).

**Fix:** Start it where the manager is created (`src/server/state.rs:65-74`, inside the
`if config.streaming.hls.enabled` branch). Store the returned `JoinHandle` inside
`HlsSessionManager` (e.g. `cleanup_handle: std::sync::Mutex<Option<JoinHandle<()>>>`)
and abort it in a `Drop` impl so tests and reloads don't leak the task.

### 3.2 Idle-session timeout (sessions currently live forever if the client walks away)

**Bug:** HLS sessions end only via explicit `DELETE /stop`
(`LiveStreamCoordinator::stop_session`, `src/streaming/live/coordinator.rs:394`). A
viewer who closes the tab leaves the FIFO reader + segmenter running indefinitely.

**Design (adapt details to the code, keep the shape):**
1. `HlsSessionManager` records last access per monitor: an
   `Instant` updated in `get_playlist`, `wait_for_segment`, and segment/init fetch paths.
   Expose `pub async fn idle_sessions(&self, idle_for: Duration) -> Vec<u32>`.
2. Add `idle_timeout_secs: u64` to the HLS config struct in
   `src/configure/streaming.rs` (serde default; pick 90 — it MUST comfortably exceed the
   LL-HLS blocking-request hold time plus one segment duration, or active LL-HLS players
   could be reaped; check the wait timeout in `wait_for_segment` and the configured
   segment duration, then document the constraint next to the field).
3. The cleanup loop from 3.1 is storage-level; idle reaping needs the coordinator (it
   owns full teardown: reader refcount + segmenter + storage). Add a watchdog task
   spawned wherever the coordinator is created in `state.rs`: every ~15s ask the HLS
   manager for idle monitors and call `coordinator.stop_session(id)` for each. Log at
   info level when reaping. Store/abort its JoinHandle like 3.1.
4. `0` disables idle reaping.

**Tests:** Unit test `idle_sessions` (touch → not idle; manipulate the stored Instant or
use a short timeout + `tokio::time::pause`/`advance` to make it deterministic). Test
that a playlist fetch refreshes the timestamp.

### 3.3 Make segment-cap eviction atomic with insertion

**Bug:** `store_segment` drops the monitors write lock, then `cleanup_monitor` re-acquires
locks (`src/streaming/hls/storage.rs:177-184` and `319-375`) — a TOCTOU window where
concurrent inserts cause over/under-eviction at the `max_segments_per_stream` boundary.

**Fix:** Restructure so the cap-eviction decision (which sequences to remove from the
in-memory map) happens under the same write lock as the insert; only the filesystem
deletions happen after the lock is released (collect paths under the lock, delete after).
Keep `cleanup_monitor` for the background/time-based path.

**Tests:** Existing storage tests live in `storage.rs` (see `cleanup_interval` test
around line 541) — add one inserting `max+N` segments and asserting exactly `max` remain.

### 3.4 Snapshot temp-source prompt teardown

**Bug:** In `src/streaming/snapshot.rs:113-164` (temporary-source path of
`capture_keyframe`), the local `Arc<MonitorSource>` is still alive when `remove_source`
is called, so the broadcast channel stays open longer than intended.

**Fix:** Explicitly `drop(source)` (and any subscriber/receiver) before calling
`remove_source(...)` in both the success and timeout paths. Two-line change; add a
comment stating why the drop order matters.

---

## Phase 4 — Daemon & service correctness

### 4.1 Unify zmc daemon-ID construction (breaks Local/V4L2 monitors today)

**Bug:** `start_all_daemons` starts Local-type monitors as `zmc -d <device>`
(`src/daemon/manager.rs:826`) but `start_monitor`/`stop_monitor`/restart/status always
use `zmc -m <id>` (lines 1630, 1696, 1775, 1791). Stop/status can't find the `-d`
daemon and the reconciliation loop churn-restarts it.

**Fix:**
1. Add one helper, e.g.
   `fn zmc_daemon_id(monitor_type: &MonitorType, device: &str, monitor_id: u32) -> String`
   returning `zmc -d {device}` for Local with non-empty device, else `zmc -m {id}`.
   Reject/fall back to `-m` if the device contains whitespace (whitespace would be
   re-split by `parse_daemon_command` and fail validation downstream) — log a warning.
2. Use it at ALL five sites. `start_monitor`/`stop_monitor` etc. currently only have the
   monitor ID — they must look up the monitor's type/device the same way
   `start_all_daemons` does (check what repo call it uses and reuse it).
3. Keep `validate_daemon_spec` as-is (it already whitelists `/dev/...` paths).

**Tests:** manager.rs has unit tests around `parse_daemon_command` (~line 2057). Add
tests for `zmc_daemon_id`: Local+device → `-d`; Local+empty device → `-m`; non-Local →
`-m`; device with space → `-m` + warning. If start/stop functions are testable with the
existing mock scaffolding, add a round-trip test (start as Local then stop finds it).

### 4.2 Make `apply_state` transactional and batch the deactivation

**Bug:** `service::daemon::apply_state` (`src/service/daemon.rs:278-395`) updates N
monitors in a loop, then flips `IsActive` flags, all as separate statements — partial
failure leaves a half-applied run state. Also deactivates other states with a per-row
loop (~lines 375-382) — N+1.

**Fix:** Wrap the whole sequence in SeaORM `db.transaction(|txn| ...)` (use Context7 for
the exact closure signature on `DatabaseConnection`; all inner repo calls must take the
`txn` connection — if repo functions take `&DatabaseConnection`, generalize the ones
involved to `&impl ConnectionTrait` as the codebase convention allows). Replace the
deactivation loop with one `update_many().filter(Name.ne(state_name))` — the same file
already uses `update_many` (~line 497) as a pattern.

**Tests:** This is DB-facing; add/extend an `#[ignore]` integration test: apply a state,
assert all monitor functions changed and exactly one state has `IsActive=1`. For the
failure path, at minimum assert via code review that no statement executes outside the
transaction (a unit test forcing a mid-transaction error needs a mock — only do it if
the mock scaffolding in service tests supports it cheaply).

### 4.3 Wrap blocking shared-memory alarm calls

**Bug:** `zm_shm::trigger_alarm` / `cancel_alarm` are synchronous mmap operations called
directly from async code (`src/service/monitor.rs:852,866`).

**Fix:** Wrap each in `tokio::task::spawn_blocking(move || ...)` and map the JoinError
into the existing `ServiceUnavailableError` mapping. Note the args are `&str` — convert
to owned `String` before moving into the closure.

### 4.4 Forbid unbounded `frames::find_all`

**Bug:** `repo::frames::find_all` with `event_id = None` does
`FrameEntity::find().all(db)` (`src/repo/frames.rs:25-41`) — unbounded SELECT on what is
the largest table in real ZoneMinder installs.

**Fix:** Find the handler(s)/service calling it with `None` (grep callers). Preferred:
make `event_id` required at the API boundary (400 with a clear message if absent) and
change the repo signature to take `event_id: u64` (non-optional). If a legitimate
no-event listing use case exists, route it through the paginated variant with the
existing max-page-size clamp instead. Update the OpenAPI annotation accordingly.

**Tests:** handler test: missing event_id → 400; with event_id → 200.

---

## Phase 5 — Housekeeping (single commit is fine)

### 5.1 Delete dead WebRTC files

- Delete `src/webrtc_signaling.rs` and `src/webrtc_signaling_clean.rs` — not declared in
  any module tree (orphaned by commit dec2d29).
- Delete `src/webrtc_ffi.rs` and remove `pub mod webrtc_ffi;` from `src/lib.rs:27` — its
  only references were the two orphaned files. Before deleting, run
  `grep -rn 'webrtc_ffi' src/ tests/` to confirm nothing else references it; also check
  `Cargo.toml` for now-unused deps (`libloading`?) and remove them only if nothing else
  uses them (`cargo build` will confirm).
- Do NOT touch `src/zm_shm.rs` (used by `service/monitor.rs`) or
  `src/streaming/webrtc/` (planned native engine).

### 5.2 Initialize ffmpeg once at startup

`ffmpeg_next::init()` is only called in tests. Call `ffmpeg_next::init().ok();` once
during server startup (in `main.rs` or wherever `AppState::new` runs, before any
streaming/snapshot work). It is idempotent.

### 5.3 Percent-encode DB credentials in the connection URL

`src/configure/db.rs:176` interpolates raw username/password into
`mysql://{u}:{p}@{host}...` — breaks with `@ / ? #` in passwords (Debian-packaged ZM
auto-generates such passwords, and the `/etc/zm/zm.conf` fallback path feeds this).
Use the `percent-encoding` crate (`utf8_percent_encode` with `NON_ALPHANUMERIC` or a
USERINFO set) on username and password. Add it to Cargo.toml if not already a direct
dep. Unit-test with a password containing `@` and `/` — assert the URL parses back to
the right host/password (the `url` crate can verify).

### 5.4 Replace hand-rolled percent_decode

`src/util/middleware.rs:131-156` decodes `%XX` as `byte as char` — wrong for multi-byte
UTF-8. Replace with `percent_encoding::percent_decode_str(s).decode_utf8()` (reject on
error → treat token as invalid). Keep existing call sites' behavior for plain ASCII
tokens. Existing tests for this function (if any) should still pass; add one with
`%C3%A9` → `é`.

### 5.5 Add missing utoipa security annotations

Add `security(("jwt" = []))` to the four HLS live media handlers
(`get_live_master_playlist`, `get_live_media_playlist`, `get_live_init_segment`,
`get_live_segment`, ~`src/handlers/live.rs:301-460`) and `get_monitor_snapshot`
(~`live.rs:1048`) — they ARE protected at runtime (see non-issue #1); the spec just
lies. Also do the same for the event playback handlers if their annotations lack it.

### 5.6 (Optional / verify-first) Remove redundant inner auth middleware

PTZ (`src/routes/ptz.rs:129`), monitors, snapshots, snapshots_events route files apply
`auth_middleware` inside routers that `routes/mod.rs` already wraps with `protect(...)`,
decoding the JWT twice per request.

**Before removing anything:** read `util/authz.rs` (`protect`/`enforce`) and
`auth_middleware` and confirm `protect` inserts the SAME request extensions (user
claims) that downstream extractors (`MonitorScope`, any `UserClaims` extractor) rely
on. If yes: remove the inner `.layer(middleware::from_fn(auth_middleware))` from those
route files and run the full test suite. If no: SKIP this item and instead leave a
short comment in `routes/mod.rs` explaining the double-decode is load-bearing.

---

## Suggested commit sequence

1. `fix(security): hash passwords on user creation` (1.1)
2. `fix(api): correct HTTP status codes for Conflict/InvalidSession/TypedHeader errors` (1.2)
3. `fix(security): enforce row-level monitor ACL on event playback endpoints` (1.3)
4. `fix(security): require unrestricted scope for monitor creation` (1.4)
5. `fix(webrtc): re-read keyframe cache at connection time` (2.1)
6. `feat(webrtc): trickle server ICE candidates, honor configured STUN servers` (2.2)
7. `perf(webrtc): drop SPS wait from startup critical path` (2.3)
8. `fix(hls): start segment cleanup task and reap idle sessions` (3.1 + 3.2)
9. `fix(hls): atomic segment-cap eviction; prompt snapshot source teardown` (3.3 + 3.4)
10. `fix(daemon): unify zmc daemon id for Local monitors; transactional apply_state` (4.1 + 4.2)
11. `fix(service): spawn_blocking for shm alarms; bound frames queries` (4.3 + 4.4)
12. `chore: dead code removal, ffmpeg init, URL encoding, OpenAPI security annotations` (Phase 5)

Each commit message body should reference this plan section. End commit messages with the
project's Co-Authored-By convention.
