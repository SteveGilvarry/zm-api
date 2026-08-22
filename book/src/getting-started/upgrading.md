# Upgrading an existing ZoneMinder

**This is the step most likely to be skipped, and it fails quietly.**

zm_api runs pending migrations at startup, but if they fail it logs a warning
and carries on. The result is a service that starts, answers requests, and has
features silently missing — with one line in the journal to say why.

Run the migration explicitly, and read its output.

## Which command

Two cases, two different commands. Getting this wrong matters.

| Your database | Command |
| --- | --- |
| Already has ZoneMinder in it (1.26.0+) | `zm_api-db bridge -u mysql://…` |
| Fresh and empty | `zm_api-db up -u mysql://…` |

`bridge` walks the embedded `zm_update` chain to bring the schema up to what
zm_api expects, converges triggers, stamps the baseline migration as already
applied, then runs everything after it.

`up` assumes it is *creating* the schema. **Never run it against a database
that already has ZoneMinder tables** — the baseline is not written to adopt an
existing schema.

## Doing it

Back up first. `bridge` rewrites schema across many ZoneMinder versions in one
pass and there is no undo.

```bash
mysqldump --single-transaction --routines --triggers zm > zm-backup.sql

sudo systemctl stop zm_api
zm_api-db bridge -u mysql://zmuser:zmpass@localhost/zm
zm_api-db status -u mysql://zmuser:zmpass@localhost/zm   # confirm
sudo systemctl start zm_api
```

The connection URL can also come from `DATABASE_URL` instead of `-u`.

## Checking it worked

```bash
zm_api-db status -u mysql://zmuser:zmpass@localhost/zm
journalctl -u zm_api -n 50 | grep -i migrat
```

`status` lists every migration and whether it has been applied. Nothing should
be pending after a successful bridge.

## Other subcommands

The underlying SeaORM CLI also accepts `down`, `fresh`, `refresh`, and `reset`.
Those exist for development and **will drop data**. None is part of a supported
upgrade.

Full details: `man 8 zm_api-db`.
