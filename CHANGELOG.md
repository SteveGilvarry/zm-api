# Changelog

All notable changes to zm_api are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versioning is
[SemVer](https://semver.org/) — carrying the `v3` major from ZoneMinder's API
lineage, so a client written against the legacy `/api/v3` surface has a
recognisable path forward.

## [Unreleased]

### Added
- Man pages: `zm_api(8)`, `zm_api.env(5)`, `zm_api-takeover(8)`, `zm_api-db(8)`.
- `zm_api --help`, `--version`, and `--openapi` (writes the OpenAPI spec to
  stdout, so the API surface can be diffed or fed to a client generator without
  running a server). All three work before configuration is loaded, so they
  still answer on a host whose config is broken.
- The migration tool ships as `/usr/bin/zm_api-db` in all three packages. It was
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

### Fixed
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
- `packaging/install.sh` rewritten for source installs: it now matches the
  package layout, installs the man pages and `zm_api-db`, and runs
  `setup-instance.sh` to generate JWT keys. Previously it installed the unit to a
  different directory than the packages, never generated keys, and left a
  freshly "installed" service unable to sign a token.
- Removed the dead `[package.metadata.rpm]` block from `Cargo.toml`; nothing
  invoked cargo-rpm, and it duplicated `packaging/rpm/zm_api.spec`.

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
- **Daemon supervision (takeover mode)** — an opt-in replacement for `zmdc.pl`
  and `zmwatch.pl`, with `zm_api-takeover` to switch a host between modes
  safely.
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
  `zm_api-db bridge -u mysql://...` before first start. Only a fresh, empty
  database should use `zm_api-db up`.
- Passive mode is the default and the recommended arrangement; takeover is
  opt-in.
