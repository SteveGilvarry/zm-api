# Changelog

All notable changes to zm-api are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versioning is
[SemVer](https://semver.org/) — carrying the `v3` major from ZoneMinder's API
lineage, so a client written against the legacy `/api/v3` surface has a
recognisable path forward.

## [Unreleased]

### Added

- **Fresh databases start at ZoneMinder 1.39.1 and are upgraded by one portable
  migration per upstream `zm_update-1.39.x`** (docs/DB_VERSIONING_PLAN.md).
  `migrator up` on an empty MySQL/MariaDB *or* Postgres produces the 1.39.1
  create script, then every upgrade upstream has shipped since (currently
  through 1.39.33), the way `zmupdate.pl` does for MySQL alone. Stored
  procedures in the upstream updates (zone-coordinate conversions) are Rust
  loops; MySQL-only trigger rewrites are skipped on Postgres and logged. The
  `schema-parity` CI job now proves the 32 migrations reproduce upstream's
  create script exactly; a new `postgres-schema` job proves the Postgres
  schema has every table and column MySQL has. Existing MySQL installs are
  unchanged: `migrator bridge` walks the raw chain to the latest vendored
  version and records the migrations that version embodies (#48).

- `Monitors.DeviceClass`, `AudioDetection`, `AudioThreshold` and
  `AudioAlarmScore` (upstream 1.39.30/31) on the monitor create/update
  requests and response; `Controls` gains its light and audio capability
  columns (#48).

- `Reports.CreatedBy` is read and written (#29). The column has existed since
  1.37 but was never modelled, so it was neither stored nor returned.
  Attribution comes from the authenticated token rather than the request body —
  letting a client name the creator is forging authorship. (`description`,
  the other half of that issue, is not possible: there is no such column.)

- **Native replacements for three Perl maintenance daemons** — `zmstats.pl`,
  `zmaudit.pl` (database side) and `zmtelemetry.pl` — each independently
  switchable under `[maintenance]` and all off by default, so an existing
  install keeps running the Perl until the operator moves over. Only one of a
  pair may run: in takeover mode an enabled native job stops the supervisor
  starting its Perl counterpart; in passive mode disable the Perl daemon in
  ZoneMinder yourself.
  <br>**Stats** samples CPU and memory into `Server_Stats`, evicts stale
  `Monitor_Status` heartbeats, ages events out of the `Events_Hour/Day/Week/Month`
  windows and resyncs the counters they feed, and prunes `Logs` and `Sessions`
  under ZoneMinder's own retention settings.
  <br>**Audit** removes `Frames`/`Stats` rows whose event is gone, deletes events
  that never recorded a frame, closes events left open by a capture daemon that
  died, and recomputes `Event_Summaries` and `Storage.DiskSpace` from the rows
  they summarise.
  <br>**Telemetry** posts the same anonymous report on the same schedule.
  <br>Three deliberate differences from the Perl, each because the original is
  wrong rather than because this is simpler: archived events are genuinely
  skipped when deleting frameless events (zmaudit tests a column it never
  selects, so its guard never fires); `dry_run` writes nothing at all (zmaudit's
  `--report` still performs updates, log pruning and counter resyncs); and
  telemetry performs **no geolocation lookup** — zmtelemetry calls `ipinfo.io` on
  every collection, disclosing the server's public IP to a third party under the
  heading of anonymous statistics. Those fields are still sent, always
  `"Unknown"`.
  <br>Retention limits and the telemetry interval are read from ZoneMinder's
  `Config` table but **parsed** rather than interpolated: the Perl splices
  `ZM_LOG_DATABASE_LIMIT` straight into SQL and `eval`s `ZM_TELEMETRY_INTERVAL`
  as code, so a `Config` row is an injection surface in both.
  <br>**The filesystem half** reconciles event directories against `Events`
  rows, and is off even when the audit is on. It deliberately does not work the
  way `zmaudit.pl` does: zmaudit computes a directory from `StartDateTime` and
  `rm -rf`s the result, so a timezone difference, corrupted timestamp, changed
  `Scheme` or wrong `StorageId` makes it remove an unrelated directory with no
  error. Here nothing destructive acts on a computed path — directories are
  found by walking, identified from evidence inside them, and only an enumerated
  path is ever moved. A directory nothing identifies is reported and left alone.
  <br>Orphans are moved to a quarantine directory rather than deleted (an atomic
  rename within the storage, swept after `quarantine_retention_days`); a pass
  refuses if the storage is missing or has no monitor directories, so an
  unmounted volume cannot orphan the whole database; an orphan must be seen in
  two consecutive passes before anything happens; and a derivation canary
  compares computed against actual paths for events found by both routes,
  disabling the filesystem half on sustained disagreement. That last one turns
  the timezone class of bug from silent data loss into an alarm.

- **zm-api can serve the zm-web browser UI itself** (`[web] enabled = true`,
  `APP_WEB__ENABLED`). One process instead of a reverse proxy in front of two:
  the UI and the API share an origin by construction, so CORS stops applying,
  and TLS is already handled by `[server.tls]` / `[server.acme]`. Includes the
  SPA fallback so a refresh on `/events/123` works, `immutable` caching for
  hashed assets with `no-cache` on `index.html`, and a configurable
  Content-Security-Policy applied to UI responses only.
  <br>API paths keep their JSON 404 envelope — the SPA fallback never shadows
  `/api/`, `/swagger-ui`, `/api-docs` or `/.well-known/`, so a mistyped endpoint
  still fails loudly instead of returning an HTML page with status 200.
  <br>Off by default; enabling it with no `index.html` present logs a warning
  and serves the API anyway rather than refusing to start.
- **Still images are rotated to match the monitor's `Orientation`.**
  `/events/{id}/thumbnail` and `/monitors/{id}/snapshot` now return an upright
  JPEG, as ZoneMinder's own image view does — previously a `ROTATE_90` camera
  produced sideways thumbnails, and the failure was silent because nothing
  errored. Stills only: rotating live video would mean re-encoding, and a client
  can do it in CSS for free. `ROTATE_0` keeps the existing zero-copy path.
- Man pages: `zm-api(8)`, `zm-api.env(5)`, `zm-api-takeover(8)`, `zm-api-db(8)`.
- `zm-api --help`, `--version`, and `--openapi` (writes the OpenAPI spec to
  stdout, so the API surface can be diffed or fed to a client generator without
  running a server). All three work before configuration is loaded, so they
  still answer on a host whose config is broken.
- The migration tool ships as `/usr/bin/zm-api-db` in all three packages. It was
  previously built but packaged nowhere, leaving no upgrade path for an existing
  ZoneMinder database.
- `server.allowed_origins` (`APP_SERVER__ALLOWED_ORIGINS`) as a documented,
  `APP_`-prefixed setting, accepting a TOML array or a comma-separated string.
- A documentation site (mdBook) published to GitHub Pages, covering install,
  configuration, deployment architecture, TLS, passive/takeover mode, and
  permissions — plus a browsable API reference rendered from the OpenAPI spec,
  which CI exports from the freshly built binary so it cannot drift from the
  code. `docs/` is now the contributor-facing plan tree only.
- The OpenAPI spec is exported in CI and attached to each release, alongside a
  `SHA256SUMS` file covering every artifact.
- Release notes are taken from this file's entry for the tag, falling back to
  GitHub's generated commit list only when there isn't one.
- `scripts/check-version-consistency.sh`, run in CI before any package is built,
  so a half-finished version bump fails fast.
- `openapi.json` is committed as a reviewed baseline, and CI fails a pull
  request whose spec change could break a deployed client — a removed endpoint,
  a replaced response shape, a response field no longer guaranteed, a dropped
  enum value, or a request that gained a required field. New endpoints and new
  optional fields pass with a notice. This is the guard that would have caught
  the `/me` change above in the pull request that made it.

### Fixed

- **Over-long request fields are a 400 naming the field, not a 500** (#55).
  About forty request structs had no length rule for columns that are
  `varchar(N)` or `tinytext`, so anything past the width reached MySQL and
  came back as a truncation error. Every bounded field on those structs now
  carries the column's cap, the handlers that never called `validate()` do,
  and the caps count *characters* for `varchar` (a 64-character non-ASCII name
  is legal) and *bytes* for `tinytext` (which is 255 bytes). The monitor
  requests' 26 `#[garde(skip)]` fields are bounded the same way. The 1406→400
  safety net stays for columns nobody has enumerated.
- **The watchdog judges `zmc` by its capture heartbeat, not CPU time** (#123).
  `zmwatch.pl` reads the heartbeat in the monitor's shared memory; the CPU
  heuristic could not see a capture loop that was alive and spinning but no
  longer capturing. The health loop now reads the heartbeat (from
  `ZM_PATH_MAP`) for each running `zmc` past its startup grace, and falls back
  to CPU time only when there is no segment to read — an unreadable segment is
  not treated as hung, so a wrong path cannot restart every camera each tick.
- **The hung-daemon watchdog could never fire** (#73). `check_activity` stamped
  a timestamp on every sample and `appears_hung` then asked whether that stamp
  was older than `watch_max_delay_seconds` — microseconds later it never was.
  A `zmc` blocked on a stalled RTSP read stayed dead until an operator noticed,
  while the docs said `zmwatch.pl` was replaced. The stamp now moves only when
  CPU time advances, so its age is the stall.

- **A second `startup` on a running supervisor SIGKILLed every daemon** (#74).
  The `pkill -9` orphan sweep ran inside `start_all_daemons`, which the legacy
  socket `startup`/`pkg_start` commands and `POST /system/startup` also call.
  Because the entries still read Running nothing respawned them until the
  health tick and first backoff — a 10–20s capture gap on every monitor. The
  sweep now runs once per manager. It also uses `pkill -x`: the unanchored
  default matched `zmaudit.pl` for "zma" and `zmcontrol.pl` for "zmc".

- **A daemon stopped and started again was never crash-supervised** (#78).
  `stop_daemon` clears `auto_restart` so the health loop does not resurrect a
  deliberate stop, but nothing re-armed it on the next spawn (monitor restart,
  reconcile). Supervision is re-armed on every spawn.

- **`zm-api.service` deleted ZoneMinder's `/run/zm` on every stop/restart**
  (#82). It was declared as the unit's `RuntimeDirectory`, so systemd chowned
  it on start and removed it on stop — in passive mode, from under a running
  `zmdc.pl` and every `zmc` stream socket. The unit now only creates the
  directory when it is missing and never touches an existing one.

- **Takeover started the Perl maintenance daemon alongside its native
  replacement** (#87). `base.toml` said to enable the Rust job and disable the
  Perl daemon together, but nothing in zm-api did the second half: the
  singleton gates read only ZoneMinder's `Config`/`Servers` rows. With
  `[maintenance.stats]` on, `zmstats.pl` still ran and both wrote the same
  rows. Each enabled native job now suppresses its Perl counterpart's automatic
  start (an explicit `start` over the socket or REST is still honoured).

- **The daemon manager never knew which server it was** (#75) and the stats
  job read `ZM_SERVER_ID` from an environment variable nothing sets (#100).
  On a multi-server install every host started every monitor's `zmc`, the
  per-server `Servers.zm*` gates were dead, and `Server_Stats` rows landed
  under `ServerId = 0`. The id is now resolved the way ZoneMinder's own daemons
  do it — `ZM_SERVER_ID` from `zm.conf`, else `ZM_SERVER_HOST` against
  `Servers`, else the machine hostname — and shared by both.

- **Three documented `[daemon]` keys were never read** (#88).
  `enable_watchdog` now gates the hung-process check (exit detection and crash
  restarts are never optional), `stats_update_interval_seconds` drives the
  `Servers` status loop instead of a hard-coded 60s, and
  `enable_rest_api = false` leaves the daemon/system routes unregistered.

- **The retention reaper could empty a storage in one pass** (#105). With
  `Events.DiskSpace` NULL — the normal state of a freshly recorded event until
  something backfills it — each deletion credited zero bytes, the free-space
  model never moved, and the loop ran to the end of the table. NULL sizes are
  now measured on disk, free space is re-read from the filesystem every 25
  deletions, and a new `max_deletes_per_pass` (default 500, `0` = unlimited)
  bounds one pass. Alongside it: an unmounted storage (missing or empty
  directory) is skipped rather than reaped against the parent filesystem's
  free space, and the first pass waits 60s after start (#104); a failed media
  removal stops the pass instead of deleting row after row while the disk
  stays full (#106); an event archived after the candidate list was taken is
  left alone (#109); and event deletion now removes the event's `Stats` rows,
  which have no foreign key and were left behind (#110).

- **Audit and stats SQL** — a batch from the same review. The
  `Storage.DiskSpace` resync never ran: its SELECT could not decode the
  unsigned `Id` and the DECIMAL `SUM` into signed Rust integers, and the error
  was swallowed (#90); the columns are cast and a failed job is now part of the
  audit report, so a test asserting a clean pass fails (#101 for the stats
  job likewise). Every "older than N" comparison used a session forced to UTC
  against columns ZoneMinder writes in local time (#98); the connection now
  sets `time_zone = SYSTEM`. The window-counter and total-counter resyncs took
  locking reads inside an UPDATE in the reverse order of ZoneMinder's Events
  triggers (#99); they read with a plain SELECT and write by primary key. The
  orphan-frame sweep full-scanned `Frames` under next-key locks (#94); it
  selects the orphan ids first and deletes by index in batches.
  `max_deletes_per_pass` capped which rows were *looked at* rather than
  deleted (#91). Two `Storage` rows on one path confirmed an orphan in a
  single pass (#93). `min_age_seconds` never applied to directory quarantine
  (#95). `close_unclosed_events` closed live events still receiving frames
  (#96). A Deep-scheme day directory holding one `.{id}` symlink was recorded
  as the event itself (#92); the link is followed to the leaf. A storage whose
  mount dropped, then got a fresh monitor directory from zmc, passed the
  preconditions and had every older row deleted as "no media" (#89); a pass
  now refuses when more rows are missing than events were found. ONVIF
  event-listener rows, which never have frames, were deleted as empty (#97).

- **Supervisor lifecycle** — reconcile only diffed the database against the
  process map in one direction, so a deleted monitor's daemons ran on, and a
  hard-deleted one crash-looped forever (#76). `restart_monitor` slept 500ms
  and spawned over a process still shutting down, orphaning it (#77); starts
  are refused over a `Stopping` entry and restarts wait for the exit. A
  daemon restart latched the process-wide shutdown flag, so the ONVIF event
  listeners exited and never came back (#79); they now watch a separate
  process-exit flag. Background loops that missed the shutdown wake-up
  mid-tick ran on beside their replacements (#80); every startup retires the
  previous generation. Stop/kill on a crashed entry signalled a dead, possibly
  reused pid (#81). Two Local monitors on one device had their shared `zmc -d`
  stopped and restarted every tick (#113). A database still coming up at boot
  left the singleton daemons unstarted for the life of the process (#114); it
  retries. Reconcile respawned a crash-looping daemon every 60s, capping the
  documented backoff (#115). The HTTP drain ran unbounded and before the daemon
  SIGTERM wave, so systemd could SIGKILL zmc mid-stop (#116); both run
  concurrently and bounded, and `TimeoutStopSec` leaves margin. `POST
  /system/state` returned a bare 500 after committing when the restart failed
  (#117).

- **zmdc.sock compatibility** — `status`/`check` ignored the daemon and
  never emitted zmdc.pl's per-daemon lines or words, which the web console
  string-matches (#83). Daemons were keyed by name alone, so `zmdc.pl stop zmc
  -m 1` missed the tracked `zmc -m 1` and `start` spawned a duplicate (#84);
  one canonical `"<command> <args>"` key everywhere. The socket was 0660 in
  the service user's own group, so ZoneMinder's web user could not connect
  (#85); it is chgrp'd to `ZM_WEB_GROUP` (or `daemon.socket_group`) and the
  install script adds the service account to that group. `logrot` sent SIGHUP
  — reload, which drops zmc's camera — to every daemon, so nightly logrotate
  interrupted capture (#86); it sends SIGWINCH like zmdc.pl. The orphan sweep
  missed four Perl daemons and `zm-core`, and never checked for a live
  zmdc.pl before pkilling (#118); the list is derived from the spawn table,
  anchored on the full command line, and takeover refuses while something
  answers on the socket. Command reads had no size or time bound (#119).

- **Smaller ones** — telemetry kept the query string, so `?user=&password=`
  camera credentials were sent (#102); it ignored `ZM_TELEMETRY_DATA` and
  silently minted a fresh uuid every pass on a database without the Config
  row (#103). Deep-scheme deletion removed a timestamp-derived directory with
  no check it held the event, and unlinked the `.{id}` marker from the wrong
  directory (#107); it follows the marker and refuses a directory naming
  another event. `Storage.DoDelete` was never read (#108). The byte quota
  counted only deletable events (#110). The retention guide's grep matched
  nothing and the promised per-deletion reason was never logged (#112). The
  ONVIF listener's reconnect backoff never reset, its renew cancelled an
  in-flight pull whose response the device had already dequeued, and
  listeners were spawned for deleted monitors (#120). `POST
  /monitors/{id}/zmnext` returned 200 with zm-next disabled (#121). Non-DB
  unit tests now cover the reaper's free-space loop, NULL `DiskSpace`, age
  cutoff, quota and cap (#111).

- **The rate limiter made the API unusable for any browser client** (#70). A
  burst of `0` with the limiter enabled was clamped silently to `1`, so one
  request succeeded and everything after it returned 429 — no page in any
  single-page app could load, and nothing said why. A burst that small is never
  intentional, so it is now treated as the misconfiguration it is: the server
  substitutes a usable value and logs an error naming the setting.
  <br>The setting is renamed `rate_limit_period_secs`, because
  `rate_limit_per_second` read as a rate while meaning a *period* — setting it
  to `4` expecting four requests a second gave one request every four seconds.
  The old name is still accepted, so existing configuration keeps working.
  <br>The production defaults are retuned from one token per 25 seconds with a
  burst of 50 to one per second with a burst of 120. The old values let the
  first screen through and then throttled everything after it to one request
  every 25 seconds. Credential brute-forcing is handled separately and far more
  tightly by the auth limiter, which is why the global one can afford to be
  generous.
- **Six duplicate `operationId`s** (#32) — the five AI-model routes collided
  with the camera-model routes, and two unrelated `update_state` handlers with
  each other. A generator silently emits one method and drops the other.
- **Four routes that enforce authentication did not say so in the spec** (#32):
  `/daemons`, `/daemons/{id}`, `/system/status` and the WebRTC signalling
  socket. A generated client reads a missing `security` key as "no token
  needed".
- **`NaiveDateTimeWrapper` claimed `format: date-time`** (#32) while emitting
  `2025-04-24T12:34:56`, which has no offset and is therefore not RFC 3339.
  Generators built an offset-aware parser that rejected every value the API
  sends. It is now described as what it is — local wall-clock time as
  ZoneMinder stores it.
- **Zone `Area` was never computed** (#43). Every zone created through the API
  had `Area = 0` hardcoded, and changing a zone's coordinates never recomputed
  it. That is not cosmetic: when a zone's units are `Percent`, the alarm
  thresholds are stored *relative to* `Area`, so those zones had thresholds that
  silently did not mean what they said. Area is now derived from `Coords` on
  both create and update, matching ZoneMinder's own `getPolyArea` (plain
  shoelace — upstream keeps an inclusive-pixel variant but no longer calls it).
  Coordinates that do not describe a polygon are rejected with a 400 rather than
  stored with a zero.
- **`SaveJPEGs` rejected its own default** (#39). It is a two-bit mask whose
  column default is 3, but the bound was `-1..=1`, so the API refused the value
  ZoneMinder ships with — the same class of bug as #19. The two neighbouring
  fields had the same copy-pasted bound and were also wrong: `VideoWriter` is
  0–2 (disabled / encode / camera passthrough) and `RecordAudio` is 0–1. All
  three confirmed against the upstream monitor form rather than inferred.
- **Storage created through the API could never be reclaimed** (#44).
  `DoDelete` was hardcoded to 0 while the column defaults to 1, so neither the
  retention reaper nor `DELETE /events/{id}` could remove media from a storage
  the API created — the disk fills and nothing says why. Now defaults to 1,
  matching the column, and is settable on create.
- **Over-long values return 400 instead of 500** (#55). Roughly forty request
  fields write to fixed-width columns with no length rule of their own, and each
  turned an over-long value into `DATABASE_ERROR` with no indication of what was
  wrong. A "data too long" error is now mapped to `VALUE_TOO_LONG` / 400 naming
  the offending column. Only the column name is surfaced — the driver's message
  can carry the rejected value and surrounding SQL, and that redaction is
  tested. Per-DTO rules remain better where they exist, since they reject before
  the round trip; this is the net under everything else.
- **Bridged installs kept a legacy collation** (#40), failing upgrade-parity on
  every pull request since #14. The bridge normalised `EncoderTemplates` *to*
  `utf8mb4_unicode_ci`, which is the value the legacy chain creates it with —
  converting toward the old collation guaranteed the mismatch against a fresh
  baseline instead of removing it. It now converges on the database's own
  default, and any other table that drifts is logged by name.

### Removed

- `frame_skip` from the monitor create/update requests and response:
  upstream dropped `Monitors.FrameSkip` in 1.39.24. `motion_frame_skip`
  remains. `User_Preferences.Name` is required on create (NOT NULL and unique
  per user since 1.39.19) (#48).

- **Config blocks nothing implemented** (#53). `[streaming.rtsp_proxy]` declared
  a port and an RTP range that nothing bound, and `[streaming.go2rtc]` a base
  URL that nothing called — an operator could configure either, restart, and get
  no behaviour change and no warning. Both are gone, along with an unused
  request DTO. `Monitors.Go2RTCEnabled` stays: that is ZoneMinder's own column
  and the response passes it through. Removing them is upgrade-safe — no config
  struct denies unknown fields, so a stale block in an existing file is ignored
  rather than refusing to start, and there is a test for that.

- **Enums emitted Rust variant names instead of the values ZoneMinder stores.**
  `#[sea_orm(string_value = …)]` governs only the database mapping, so serde
  fell back to the variant name: `/monitors` reported `Rotate90` where the
  column holds `ROTATE_90`, and `Curl` where it holds `cURL`. Nine enums were
  affected — `Orientation`, `MonitorType`, `DefaultCodec`, `EventCloseMode`,
  `Rtsp2WebType`, `Decoding`, `OutputContainer`, `StorageType`,
  `SynopsisStatus`. It was self-consistent, and therefore invisible: requests
  accepted the same wrong spelling responses emitted, so a client that only ever
  talked to this API round-tripped fine while anything that knows ZoneMinder's
  real values silently mismatched.
  <br>Responses and the OpenAPI schema now carry the DB values. The previous
  spelling is still **accepted** on input via a serde alias, so clients keep
  working while they migrate. A test walks `ActiveEnum::values()` for every
  affected enum, so a newly generated one is covered without anyone remembering.
- **A configured TURN server had no effect.** `AppState` built the WebRTC engine
  from defaults rather than the loaded `[streaming.webrtc]` config, so
  `stun_servers` and `turn` were parsed, validated, and discarded — viewers
  behind symmetric NAT could not connect, with nothing in the log to explain it.
- **CORS was undiscoverable.** The allowed-origin list came from a bare,
  un-prefixed `ALLOWED_ORIGINS` variable that appeared in no config file and no
  documentation, defaulting to localhost. A dashboard on any other origin was
  silently blocked with no string in the repo to grep for. The variable is still
  honoured (with a deprecation warning) so existing deployments keep working; the
  effective list and its source are now logged at startup.
- **`APP_DAEMON__SCRIPT_PATH` shipped a value wrong for Debian and Ubuntu.** One
  env file serves all three package formats, and no single value suits every
  distribution, so daemon paths are now resolved by searching the standard
  locations, with the setting as an override.
- **`Requires=mariadb.service` failed the unit** on hosts using `mysql.service`
  or a remote database. The unit is still ordered `After=` both, and
  `Restart=on-failure` covers a slow database.
- A stream socket the service user cannot open now reports that
  `ZM_STREAM_SOCKET_GROUP` membership is missing, rather than a bare "permission
  denied" naming nothing actionable.
- `docs/tls.md` claimed the systemd unit uses `DynamicUser`; it runs as
  `User=zoneminder`.

### Changed

- **BREAKING: `frame_skip` is gone from monitor requests and responses.**
  ZoneMinder 1.39.24 drops `Monitors.FrameSkip`, and zm-api now mirrors that
  update, so `MonitorResponse` no longer carries the field and create/update
  ignore it. `motion_frame_skip` is unchanged.
- **BREAKING: six `operationId`s renamed** (#32). They were duplicated, which
  meant a generated client silently got one method and lost the other, so this
  had to change — but it renames methods for anyone already generating against
  the spec.
  <br>`list_models` → `list_ai_models`, `create_model` → `create_ai_model`,
  `get_model` → `get_ai_model`, `update_model` → `update_ai_model`,
  `delete_model` → `delete_ai_model` (the AI registry; the camera-model routes
  keep the plain names). `update_state` on `/monitors/{id}/state` →
  `update_monitor_state`, and on `/states/{id}` → `update_state_preset`.
  <br>The compatibility gate did not catch this on its first run — it compared
  paths, response shapes and schemas but not operation ids. It does now, which
  is how the list above was produced.
- **BREAKING: `rate_limit_per_second` renamed to `rate_limit_period_secs`**
  (#70). The old name read as a rate and meant a period. It is still accepted
  as an alias, so no configuration needs changing, but the old spelling is
  misleading enough that it should not be used in new files.

- **BREAKING: `GET /api/v3/me` returns a wrapper, not a bare user.** As of
  `5ce04e5` the response is `MeResponse` — `{ user, issued_at, expires_at,
  token_type }` — where it was previously `UserResponse` with the eight
  permission columns at the top level. This shipped without a changelog entry
  and broke zm-web's permission gating: reading the wrapper as a user finds no
  permission columns, and absent columns fail closed to `None`, so the camera
  wall and every edit control disappeared.
  <br>Clients should read `response.user`. Accepting both shapes is worth it
  while older backends are still deployed.
- **BREAKING: the project is named `zm-api` throughout, including on disk.**
  The binary is `/usr/bin/zm-api`, config lives in `/etc/zm-api/`, state in
  `/var/lib/zm-api/`, logs in `/var/log/zm-api/`, and the unit is
  `zm-api.service`; the helpers are `zm-api-db` and `zm-api-takeover`, and the
  man pages match. The distribution packages were already called `zm-api` — only
  what they installed disagreed. Nothing has been released, so there is no
  upgrade path to migrate; a pre-release install should be removed and
  reinstalled. The Rust crate is still imported as `zm_api`, which is the normal
  Cargo mapping for a hyphenated package name.
- `packaging/install.sh` rewritten for source installs: it now matches the
  package layout, installs the man pages and `zm-api-db`, and runs
  `setup-instance.sh` to generate JWT keys. Previously it installed the unit to a
  different directory than the packages, never generated keys, and left a
  freshly "installed" service unable to sign a token.
- Removed the dead `[package.metadata.rpm]` block from `Cargo.toml`; nothing
  invoked cargo-rpm, and it duplicated `packaging/rpm/zm-api.spec`.

## [3.0.0-alpha.1]

First Rust release, replacing ZoneMinder's Perl/PHP/CGI API surface with a
single native service. It talks directly to an existing ZoneMinder
MySQL/MariaDB database and ships in **passive mode**, serving only the REST API
so it can be installed alongside a running ZoneMinder without touching its
daemons.

### Added
- **REST API** under `/api/v3` for monitors, events, frames, zones, groups,
  users, storage, controls, PTZ presets, and configuration — with an
  auto-generated OpenAPI 3 spec at `/api-docs/openapi.json` and Swagger UI at
  `/swagger-ui`.
- **Live streaming** over WebRTC and HLS, sourced from zmc's per-monitor stream
  socket (video and audio on one connection with a HELLO codec handshake).
- **Event playback** — VOD, fragmented-MP4 streaming, thumbnails, and motion
  synopsis.
- **Authentication and authorisation** — RS256 JWTs with separate access and
  refresh key pairs, server-side revocation, feature-level RBAC, and row-level
  monitor ACLs.
- **Daemon supervision (takeover mode)** — one native supervisor replacing both
  `zmdc.pl` and `zmwatch.pl`, with exponential backoff, database
  reconciliation, REST daemon control, and a `zmdc.sock` compatibility shim.
  `zm-api-takeover` switches a host between modes in one command, either way.
- **Retention** — automatic recording cleanup bounded by free-space floor, age,
  and per-storage quota, deleting media and database rows together.
- **ONVIF** device discovery, media profiles, PTZ, and event pull-point.
- **Natural-language event search** over MariaDB 11.8 native vectors.
- **Object-detection registry** — CRUD over ZoneMinder 1.39's `AI_Datasets`,
  `AI_Models`, and `AI_Object_Classes`, plus the per-monitor detection columns.
- **Packaging** for Debian/Ubuntu (`.deb`), Fedora/RHEL/openSUSE (`.rpm`), and
  Arch (`PKGBUILD`), with a systemd unit, per-install JWT key generation, and
  built-in TLS/ACME.

### Notes
- Upgrading an existing ZoneMinder database requires
  `zm-api-db bridge -u mysql://...` before first start. Only a fresh, empty
  database should use `zm-api-db up`.
- Passive mode is the install-time default so the package cannot disturb a
  running ZoneMinder. Takeover is the intended destination — one native
  supervisor replacing `zmdc.pl` and `zmwatch.pl` — and `zm-api-takeover`
  switches either way in one command.
