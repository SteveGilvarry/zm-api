# Changelog

All notable changes to zm-api are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versioning is
[SemVer](https://semver.org/) — carrying the `v3` major from ZoneMinder's API
lineage, so a client written against the legacy `/api/v3` surface has a
recognisable path forward.

## [Unreleased]

### Added

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
