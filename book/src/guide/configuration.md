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
