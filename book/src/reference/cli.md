# Command-line tools

Three binaries, each with a man page.

## `zm-api`

The server. It takes no arguments beyond these — everything else is
configuration.

| Option | What |
| --- | --- |
| `-h`, `--help` | Usage summary |
| `-V`, `--version` | Version |
| `--openapi` | Write the OpenAPI 3.1 spec to stdout |

All three are handled before configuration loads, so they still answer on a
host whose config is broken.

```bash
zm-api --openapi > openapi.json          # diff the API between releases
zm-api --openapi | jq '.paths | keys'    # list every route
```

`man 8 zm-api`

## `zm-api-db`

Database migrations. Built by cargo as `migrator`; installed as `zm-api-db`.

| Command | When |
| --- | --- |
| `bridge -u <url>` | Existing ZoneMinder database (1.26.0+) |
| `up -u <url>` | Fresh, empty database **only** |
| `status -u <url>` | List migrations and whether each applied. Read-only |

The URL can come from `DATABASE_URL` instead of `-u`. See
[Upgrading](../getting-started/upgrading.md) — picking the wrong one of
`bridge`/`up` matters.

`up` accepts a Postgres URL as well as MySQL/MariaDB
(`zm-api-db up -u postgres://user:pass@host:5432/zm`). On either backend an
empty database becomes ZoneMinder's 1.39.1 schema followed by every upstream
schema update since, one migration per `zm_update-1.39.x` (see
`docs/DB_VERSIONING_PLAN.md`). Postgres support stops at the schema: the API
itself still runs against MySQL/MariaDB only.

`man 8 zm-api-db`

## `zm-api-takeover`

Switches the host between passive and takeover mode, sequencing
`zoneminder.service` and `zm-api.service` so they are never both supervising.

| Option | What |
| --- | --- |
| *(none)* | Take over |
| `--revert`, `--passive` | Hand control back to ZoneMinder |
| `--yes`, `-y` | Skip the confirmation prompt |

Requires root. See [Passive and takeover mode](../guide/takeover.md).

`man 8 zm-api-takeover`

## Configuration reference

`man 5 zm-api.env` documents every `APP_*` variable.
