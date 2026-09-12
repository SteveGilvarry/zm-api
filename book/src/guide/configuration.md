# Configuration

Sources apply in order, each overriding the previous:

1. **ZoneMinder's `/etc/zm/zm.conf`** and `/etc/zm/conf.d/*.conf` — database
   settings only. This is what lets a packaged install work against an existing
   ZoneMinder without being told anything.
2. **`/etc/zm-api/base.toml`** — the packaged defaults layer. Replaced on
   upgrade; don't edit it.
3. **`/etc/zm-api/prod.toml`** — your configuration.
4. **`APP_*` environment variables**, from `/etc/zm-api/zm-api.env`.

The environment file wins over everything. That makes it the right place for
host-specific settings, and a trap: a stale value there silently overrides a
corrected default shipped in a later package upgrade.

## Naming

`APP_`, then the TOML path in upper case with `__` between levels:

| TOML | Environment |
| --- | --- |
| `db.host` | `APP_DB__HOST` |
| `server.allowed_origins` | `APP_SERVER__ALLOWED_ORIGINS` |
| `server.middleware.body_limit_bytes` | `APP_SERVER__MIDDLEWARE__BODY_LIMIT_BYTES` |

List values are indexed: `APP_SERVER__ACME__DOMAINS__0=api.example.com`.

Restart the service after editing.

## The settings you are most likely to need

| Variable | Notes |
| --- | --- |
| `APP_SERVER__ALLOWED_ORIGINS` | CORS. Required for a cross-origin dashboard — see [Serving a dashboard](dashboard.md) |
| `APP_DB__HOST` etc. | Only if the database isn't where `zm.conf` says |
| `APP_DAEMON__ENABLED` | `false` = passive (install default), `true` = zm-api supervises the daemons. Prefer `zm-api-takeover` over setting this by hand |
| `APP_SERVER__PORT` | Default 8080 |
| `RUST_LOG` | `info`, or per-module: `info,zm-api::streaming=debug` |
| `APP_SERVER__MIDDLEWARE__TRUST_PROXY_HEADERS` | `true` only behind a trusted proxy |

`man 5 zm-api.env` documents every variable.

## Daemon control

The `[daemon]` table configures the supervisor that replaces `zmdc.pl` and
`zmwatch.pl`. The whole table is inert while `enabled = false` — in passive mode
no manager is constructed, so none of these values are read. See
[Passive and takeover mode](takeover.md).

| Key | Default | Notes |
| --- | --- | --- |
| `enabled` | `false` | Passive vs. takeover. Prefer `zm-api-takeover` over setting it by hand |
| `socket_path` | `/run/zm` | Directory holding the legacy IPC socket |
| `socket_name` | `zmdc.sock` | Speaks the `zmdc.pl` wire protocol |
| `enable_socket_ipc` | `true` | Bind that socket at all |
| `enable_rest_api` | `true` | Expose `/api/v3/daemons*` and `/api/v3/system/*` |
| `bin_path` | `/usr/bin` | Where `zmc` and `zma` live |
| `script_path` | `/usr/bin` | Where the Perl daemons live |
| `min_backoff_seconds` | `5` | First restart delay after a crash |
| `max_backoff_seconds` | `900` | Backoff ceiling, and the stability threshold |
| `shutdown_timeout_seconds` | `30` | SIGTERM, then SIGKILL after this |
| `stats_update_interval_seconds` | `60` | How often process stats are written to the database |

`bin_path` and `script_path` are hints, not requirements. If the command is not
found where you point, the standard per-distribution locations are searched:
`/usr/bin`, `/usr/share/zoneminder/scripts` (Fedora, openSUSE, RHEL) and
`/usr/local/bin` for Perl scripts; `/usr/bin` and `/usr/local/bin` for `zmc` and
`zma`. Set them only when ZoneMinder is installed somewhere unusual.

Restart backoff is `min_backoff_seconds × 2^attempt`, capped at
`max_backoff_seconds` — the first retry waits 10s, then 20s, 40s, up to 15
minutes by default. A daemon
that stays up longer than the cap is considered stable and its counter resets,
which is what stops a camera that was briefly unreachable from being stuck at a
15-minute retry forever.

### Watchdog

Three further keys have no entry in `base.toml` and exist only as built-in
defaults, so you will not find them by reading the shipped config. Add them to
your own TOML to change them:

| Key | Default | Notes |
| --- | --- | --- |
| `enable_watchdog` | `true` | Health-check loop; this is the part that replaces `zmwatch.pl` |
| `watch_check_interval_seconds` | `10` | Matches ZoneMinder's `ZM_WATCH_CHECK_INTERVAL` |
| `watch_max_delay_seconds` | `30` | Seconds of unchanged CPU time before a restart. ZoneMinder's `ZM_WATCH_MAX_DELAY` default is 45 |

Separately from the watchdog, a reconciliation loop runs every 60 seconds (after
a 45-second startup delay) and brings running daemons back in line with the
`Monitors` table. Neither interval is configurable.

### Maintenance jobs

The `[maintenance.*]` tables are a different thing: native replacements for
ZoneMinder's Perl *maintenance* daemons, which run inside zm-api rather than as
supervised processes, and are independently switchable from `[daemon]`. They are
documented in [Replacing the Perl maintenance daemons](maintenance.md).

## Profiles

`APP_PROFILE` selects which TOML loads alongside `base.toml`: `dev`, `test`,
`test-db`, or `prod`. Packaged installs use `prod`.

`prod` differs from the defaults in ways worth knowing: it enables the global
per-IP rate limiter (off in `base.toml`) and trusts proxy headers.

## Secrets

JWT signing keys are generated per install into `/var/lib/zm-api/keys` and are
never packaged. To regenerate:

```bash
sudo /usr/share/zm-api/setup-instance.sh   # idempotent; won't overwrite
```

Existing tokens stop working if you delete the old keys first.
