<div align="center">

# 🎥 zm-api

### A modern, fast, type-safe REST API for [ZoneMinder](https://zoneminder.com)

*Rebuilding ZoneMinder's aging Perl/PHP API surface as a single, well-tested Rust service —
with live streaming, fine-grained access control, and OpenAPI docs baked in.*

[![Tests](https://github.com/SteveGilvarry/zm-api/actions/workflows/test.yml/badge.svg)](https://github.com/SteveGilvarry/zm-api/actions/workflows/test.yml)
![Coverage](https://img.shields.io/badge/coverage-62%25-green)
![Rust](https://img.shields.io/badge/Rust-2021-orange?logo=rust&logoColor=white)
![Axum](https://img.shields.io/badge/Axum-0.8-blue)
![SeaORM](https://img.shields.io/badge/SeaORM-1.1-9cf)
[![License](https://img.shields.io/badge/license-AGPL--3.0-blue)](LICENSE)
![Status](https://img.shields.io/badge/status-active%20development-yellow)

</div>

---

## 📦 Install

Install the prebuilt package — it starts in **passive mode** (REST API only), so it is safe
to drop onto a live ZoneMinder box without changing how anything records:

```bash
sudo dpkg -i zm-api_*.deb        # Ubuntu 24.04 family (amd64, arm64)
sudo dnf install zm-api-*.rpm    # Fedora 41  (zypper on openSUSE Tumbleweed)
```

Those are the targets built and tested by CI. The packages may work on nearby
releases — Debian bookworm, Ubuntu 22.04, RHEL-family — but nothing verifies it,
so the dependency versions cargo-deb pins are not guaranteed to resolve. Build
from source there, or ask for a target to be added.

**Already running ZoneMinder?** Migrate its database before the first start —
`zm-api-db bridge -u mysql://zmuser:zmpass@localhost/zm`. Only a fresh, empty database
should use `zm-api-db up`. See `man 8 zm-api-db`.

📖 **[Full documentation](https://stevegilvarry.github.io/zm-api/)** — install, configuration,
deployment architecture, and a browsable
**[API reference](https://stevegilvarry.github.io/zm-api/api/)** generated from the OpenAPI spec.

Want to build from source or run a local dev setup instead? See **[Quick Start](#-quick-start)**.
Packaging internals and the release process: **[`docs/deployment.md`](docs/deployment.md)**.
What changed: **[`CHANGELOG.md`](CHANGELOG.md)**.

---

## ✨ Why zm-api?

ZoneMinder is a rock-solid surveillance platform, but its API grew organically across Perl,
PHP, and CGI over two decades. **zm-api** replaces that surface with one cohesive service:

- 🦀 **One binary, one language** — no PHP-FPM, no CGI, no Perl runtime to babysit.
- ⚡ **Fast & async** — built on Axum + Tokio; streaming endpoints don't block the API.
- 🔒 **Secure by default** — JWT auth, per-feature RBAC, *and* row-level monitor ACLs.
- 📖 **Self-documenting** — every endpoint is in an auto-generated OpenAPI spec + Swagger UI.
- 🧪 **Actually tested** — 1,100+ unit + integration tests, with a coverage gate in CI.
- 🗄️ **Drop-in schema** — talks directly to an existing ZoneMinder MySQL/MariaDB database.

---

## 🚀 Features

### 📹 Monitors & Events
Full CRUD for monitors, events, frames, zones, and event metadata — paginated, filterable,
and searchable. Monitor state & alarm control. Per-monitor snapshots straight from the live
stream.

### 🎬 Live Streaming
Two delivery paths from one API:
- **HLS** — fragmented-MP4 playlists for any HTML5 `<video>` element.
- **WebRTC** — native peer connection with ICE/SDP signaling.

Plus recorded-event playback (video, byte-range seeking, thumbnails).

### 🕹️ PTZ Control
Pan/tilt/zoom with native protocol drivers and a Perl-bridge fallback — ONVIF and
vendor protocols, presets, and continuous control.

### 🔐 Access Control
- **JWT** access + refresh tokens.
- **Feature RBAC** — ZoneMinder's 8 permission columns (`Stream`, `Events`, `Control`,
  `Monitors`, `Groups`, `Devices`, `Snapshots`, `System`) enforced on every route.
- **Row-level ACLs** — `Monitors_Permissions` / `Groups_Permissions` resolved per request,
  so a user only ever sees the monitors they're granted (default-allow, fully backward
  compatible).

### 🛠️ Operations
Optional daemon supervision (zmc/zma) with a Unix-socket IPC shim for legacy `zmdc.pl`
compatibility, storage & server management, configs, logs, montage layouts, and system
control — all behind graceful SIGTERM/SIGINT shutdown.
See **[Daemon Control](#️-daemon-control)** for how supervision runs and how to turn it on.

### 🧱 Production hardening
TLS with optional ACME/Let's Encrypt, security headers, gzip/brotli compression
(streaming routes excluded), request-body limits, and opt-in per-IP rate limiting.

---

## 🏗️ Architecture

A clean, one-way layered flow — handlers never touch the database directly.

```mermaid
flowchart TD
    Client["🌐 Client / Browser / ZM UI"] -->|HTTPS · JWT| Stack

    subgraph Stack["Middleware stack"]
        direction LR
        T[trace] --> C[compression] --> H[security headers] --> R[rate limit] --> O[CORS]
    end

    Stack --> Auth{"auth · RBAC · monitor ACL"}
    Auth -->|allowed| Handlers["📥 Handlers — extraction & validation"]
    Auth -->|denied| Reject["401 / 403 / 404"]

    Handlers --> Services["⚙️ Services — business logic"]
    Services --> Repos["🗃️ Repositories — SeaORM queries"]
    Repos --> DB[("🛢️ MariaDB / MySQL<br/>ZoneMinder schema")]

    Handlers -.live & playback.-> Streaming["🎬 HLS · WebRTC"]
    Streaming -.-> Cameras[("📷 ZoneMinder capture daemons")]
```

| Layer | Path | Responsibility |
|------|------|----------------|
| **Routes** | `src/routes/` | Endpoint wiring & middleware layering |
| **Handlers** | `src/handlers/` | Request extraction, validation, response mapping |
| **Services** | `src/service/` | Business logic |
| **Repositories** | `src/repo/` | Database queries |
| **Entities** | `src/entity/` | SeaORM models (generated from the ZM schema) |
| **DTOs** | `src/dto/` | Request/response types (OpenAPI schemas) |

---

## 🛠️ Daemon Control

zm-api can supervise ZoneMinder's daemons itself, replacing `zmdc.pl` and `zmwatch.pl`.
It is **off by default** — a starting point, not the destination. Takeover is where a zm-api
host is meant to end up; passive exists so you pick the moment, with a one-command way back.

### Passive vs. active

| | **Passive** (default) | **Active** (takeover) |
|---|---|---|
| `daemon.enabled` | `false` | `true` |
| Supervises `zmc`/`zma` | No — ZoneMinder does | Yes |
| Binds `zmdc.sock` | No | Yes |
| `zoneminder.service` | Keeps running | Must be stopped & disabled |

In passive mode no manager is constructed at all: nothing binds the socket, no orphan sweep
runs, no process is spawned. zm-api is purely a REST layer over the database ZoneMinder is
already using — which is why the packages are safe to drop onto a live box. The daemon-control
endpoints stay registered but return **503** until you switch.

> ⚠️ **Exactly one supervisor may run.** Starting in takeover mode `pkill -9`s any `zmc`,
> `zma` and `zmfilter.pl` left behind before starting its own. Leaving `zoneminder.service`
> enabled means two supervisors killing and restarting each other's processes — daemons
> restarting in a loop, events recording erratically.

Switching is scripted, and sequences both services in the right order either way. Prefer it
over editing `APP_DAEMON__ENABLED` by hand:

```bash
sudo zm-api-takeover              # hand supervision to zm-api
sudo zm-api-takeover --revert     # hand it back to ZoneMinder
```

### What it supervises

Per-monitor capture and analysis, plus ZoneMinder's Perl singletons, started in priority
order:

| Priority | Daemon | Scope |
|---|---|---|
| 5 / 6 | `zmc` / `zma` | one per monitor |
| 10 | `zmfilter.pl --daemon` | singleton |
| 20 | `zmaudit.pl --continuous` | singleton |
| 30 | `zmtrigger.pl` | singleton |
| 40 / 50 | `zmcontrol.pl` / `zmtrack.pl` | one per controllable / tracking monitor |
| 70 – 90 | `zmstats.pl`, `zmtelemetry.pl`, `zmeventnotification.pl` | singleton |

`zmwatch.pl` has no entry — watching capture daemons and restarting them is the supervisor's
own health-check loop.

### How supervision behaves

- **Health checks** — every 10s; a daemon whose heartbeat is more than 30s stale is restarted
  (defaults match `ZM_WATCH_CHECK_INTERVAL` / `ZM_WATCH_MAX_DELAY`).
- **Restart backoff** — exponential: `min_backoff × 2^attempt`, capped at `max_backoff`
  (5s → 15min by default). A process that stayed up longer than the cap resets its counter.
- **Reconciliation** — every 60s, after a 45s startup delay, the monitors in the database are
  diffed against what is actually running and the difference is corrected. That is what
  self-heals an external `kill`, a crash between a DB write and the daemon call, or a reboot.
- **Graceful shutdown** — SIGTERM, then SIGKILL after `shutdown_timeout_seconds`. Supervised
  daemons are drained whether the server exited cleanly or not.
- **Orphan sweep** — daemons left behind by a previous run are killed at startup, before
  anything new is spawned.

### Three ways to drive it

1. **REST** — `/api/v3/daemons` for per-daemon list/start/stop/restart/reload, and
   `/api/v3/system/*` for status, startup, shutdown, restart, logrot, and run state. The whole
   group is JWT-gated.
2. **Legacy socket** — `/run/zm/zmdc.sock` speaks the `zmdc.pl` wire protocol, so existing
   scripts and tooling keep working unchanged.
3. **Run states** — `POST /api/v3/system/state` applies a named row from the `States` table.
   The monitor changes and the state activation commit as one transaction; the daemon restart
   deliberately happens outside it.

### Configuration

```toml
[daemon]
enabled = false            # passive; true = takeover
socket_path = "/run/zm"
socket_name = "zmdc.sock"
bin_path = "/usr/bin"      # zmc, zma
script_path = "/usr/bin"   # Perl daemons — searched per-distro if no match here
min_backoff_seconds = 5
max_backoff_seconds = 900
shutdown_timeout_seconds = 30
stats_update_interval_seconds = 60
enable_socket_ipc = true   # bind zmdc.sock
enable_rest_api = true     # expose the daemon routes
```

Every key takes an env override — `APP_DAEMON__ENABLED=true` is the one the takeover script
writes.

### Native replacements for the Perl maintenance daemons

Separately from supervision, several Perl daemons have Rust equivalents that run *inside*
zm-api rather than as supervised processes. Each is independently switchable and **all
default off**, so an existing install keeps running the Perl until you move over deliberately.

| Config | Replaces | What it does |
|---|---|---|
| `[maintenance.stats]` | `zmstats.pl` | CPU/memory into `Server_Stats`, stale `Monitor_Status` eviction, event-window counters, expired session pruning |
| `[maintenance.audit]` | `zmaudit.pl` (database side) | orphaned `Frames`/`Stats` rows, events that never recorded a frame, events left open by a dead capture daemon, counter resync |
| `[maintenance.audit.filesystem]` | `zmaudit.pl` (disk side) | quarantines orphaned event directories — discovered by walking and identified from evidence inside them, never by deriving a path from a timestamp |
| `[maintenance.telemetry]` | `zmtelemetry.pl` | anonymous usage report, with no geolocation lookup |

> ⚠️ Enable a Rust job and disable its Perl counterpart **together** — running both has them
> competing over the same rows. The audit ships with `dry_run = true`; read a pass or two in
> the log before turning that off.

Going deeper: **[the takeover guide](book/src/guide/takeover.md)** covers prerequisites,
verification and rollback; **[the maintenance guide](book/src/guide/maintenance.md)** covers
each Perl replacement in detail; and systemd units, permissions and the distro matrix are in
[`docs/deployment.md`](docs/deployment.md).

---

## 🧰 Tech Stack

| | |
|---|---|
| **Language** | Rust (edition 2021) |
| **Web framework** | [Axum](https://github.com/tokio-rs/axum) 0.8 |
| **Async runtime** | [Tokio](https://tokio.rs) |
| **ORM** | [SeaORM](https://www.sea-ql.org/SeaORM/) 1.1 (MySQL/MariaDB) |
| **API docs** | [utoipa](https://github.com/juhaku/utoipa) + Swagger UI |
| **Auth** | JSON Web Tokens (`jsonwebtoken`) |
| **Streaming** | `webrtc`, fMP4, HLS, `retina` (RTSP) |
| **Media** | FFmpeg (`ffmpeg-next`) for H.264 → JPEG |

---

## 🚀 Quick Start

### Install as a service (packages)

The fastest path. Packages install zm-api as a systemd service in **passive mode** — it serves
the REST API alongside a running ZoneMinder without touching its daemons, so it's safe on an
existing box. That's the on-ramp, not the end state.

```bash
sudo dpkg -i zm-api_*.deb        # Ubuntu 24.04 family (amd64, arm64)
sudo dnf install zm-api-*.rpm    # Fedora 41  (zypper on openSUSE Tumbleweed)
```

When you're ready, hand daemon supervision to zm-api — one native supervisor replacing
`zmdc.pl` and `zmwatch.pl`, with restart backoff, database reconciliation, and daemon control
over REST (this stops & disables `zoneminder.service`):

```bash
sudo zm-api-takeover             # --revert hands control back to ZoneMinder
```

What that changes, and everything the supervisor does once it is on:
**[Daemon Control](#️-daemon-control)**.

Build packages yourself with `./scripts/package.sh [deb|rpm|arch|all]`. Full distro matrix,
config, and TLS setup: [`docs/deployment.md`](docs/deployment.md).

### Build from source

For development or platforms without a prebuilt package.

**Prerequisites**

- **Rust** (current stable) — install via [rustup](https://rustup.rs)
- **MariaDB / MySQL** with a ZoneMinder schema
- **FFmpeg dev libraries** — `libavutil-dev libavcodec-dev libavformat-dev libavfilter-dev
  libavdevice-dev libswscale-dev libswresample-dev` (Debian/Ubuntu) or `brew install ffmpeg`

```bash
# 1. Clone
git clone https://github.com/SteveGilvarry/zm-api.git
cd zm-api

# 2. Spin up a local test database (Docker)
./scripts/db-manager.sh start
./scripts/db-manager.sh mysql      # load the ZoneMinder schema

# 3. Build & run
cargo run                          # uses settings/base.toml
APP_PROFILE=prod cargo run         # production profile
```

The API comes up on the address/port from your active profile (see `settings/`).

### Explore the API

Once running, open the interactive docs:

| | |
|---|---|
| 🧭 **Swagger UI** | `http://<host>:<port>/swagger-ui` |
| 📄 **OpenAPI spec** | `http://<host>:<port>/api-docs/openapi.json` |

---

## ⚙️ Configuration

Settings are layered, last one wins:

1. `settings/base.toml` — defaults
2. `settings/{APP_PROFILE}.toml` — `dev` · `test` · `test-db` · `prod`
3. **Environment variables** — prefix `APP_`, nested keys use `__`

```bash
APP_PROFILE=prod
APP_DB__HOST=10.0.0.5            # overrides db.host
APP_CONFIG_DIR=/etc/zm-api       # alternate config directory
```

> 💡 Local profiles like `settings/dev.toml` are gitignored — keep secrets out of version control.

---

## 🧪 Testing

```bash
# Unit + non-DB tests
cargo test --all-features

# Full integration suite (needs the test database)
./scripts/db-manager.sh start && ./scripts/db-manager.sh mysql
APP_PROFILE=test-db cargo test --test '*' -- --include-ignored

# Coverage report
cargo llvm-cov --all-features --ignore-filename-regex '/(entity|migration)/' \
  -- --include-ignored
```

CI runs the suite on every push and **gates line coverage** — it can't regress below the
floor. Currently **~62%** and climbing (the CI floor is 60%), across 1,100+ unit and per-domain
integration tests — roughly 850 run without a database, the rest need one.

---

## 📁 Project Layout

```
src/
├── routes/      Axum routers & middleware wiring
├── handlers/    HTTP handlers
├── service/     Business logic
├── repo/        Database query layer
├── entity/      SeaORM entities (generated from the ZM schema)
├── dto/         Request/response DTOs
├── streaming/   HLS / WebRTC pipelines
├── ptz/         PTZ control drivers
├── daemon/      ZoneMinder daemon supervision
├── configure/   Config loading
└── error/       AppError + HTTP mapping
scripts/         DB management, JWT key generation, CI helpers
settings/        Layered TOML configuration
docs/            Deployment, TLS, PTZ & streaming design notes
```

---

## 🤝 Contributing

Before opening a PR, make sure the local quality gates pass:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

Work tests-first, keep changes focused, and treat `src/entity/` as generated artifacts.
See [`CLAUDE.md`](CLAUDE.md) for the full development workflow and conventions.

---

## 📄 License

zm-api is **dual-licensed**:

- 🆓 **Open source — [AGPL-3.0](LICENSE).** Free to use, modify, and self-host. If you
  run a modified version as a network service, the AGPL requires you to publish your
  changes.
- 💼 **Commercial license.** For embedding zm-api in a closed-source product, or running a
  modified version as a hosted service without the AGPL's source-sharing obligation, a
  commercial license is available. Contact the maintainer to enquire.

Contributions are accepted under a [Contributor License Agreement](CLA.md) so the project
can be offered under both licenses — see [`CONTRIBUTING.md`](CONTRIBUTING.md).

> The `db/*.sql` schema files are from ZoneMinder and remain under its GPL-2.0 license.

---

<div align="center">
<sub>Built with 🦀 for the ZoneMinder community.</sub>
</div>
