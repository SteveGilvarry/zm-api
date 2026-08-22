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
