# Replacing the Perl maintenance daemons

ZoneMinder runs three periodic housekeeping daemons that zm-api can take over:
`zmstats.pl`, `zmaudit.pl` and `zmtelemetry.pl`. Each is independently
switchable and **all three default to off**, so an existing install keeps
running the Perl until you move over deliberately.

> **Enable the Rust job and disable the matching Perl daemon together.** Running
> both has them competing over the same rows. In takeover mode zm-api supervises
> the Perl daemons, so removing one from ZoneMinder's set is the other half of
> the switch.

## Stats — replaces `zmstats.pl`

```toml
[maintenance.stats]
enabled = true
interval_seconds = 300
```

Six jobs on one timer, none of which touch event media or `Events` rows:

- samples CPU and memory into `Server_Stats`, and trims it to a day
- mirrors the same sample onto this host's `Servers` row (multi-server only)
- evicts `Monitor_Status` rows whose heartbeat has stopped
- ages events out of the `Events_Hour/Day/Week/Month` windows and resyncs the
  counters they feed
- prunes `Logs` under `ZM_LOG_DATABASE_LIMIT` and `ZM_LOG_AUDIT_DATABASE_LIMIT`
- prunes expired `Sessions` under `ZM_COOKIE_LIFETIME`

It reads those retention settings from ZoneMinder's own `Config` table, so the
values you already set still apply. One difference: ZoneMinder splices those
values straight into its SQL, whereas here they are parsed first — a limit that
is neither a row count nor a recognised interval disables that pruning and logs
why, rather than producing a broken statement.

## Audit — replaces `zmaudit.pl` (database side)

```toml
[maintenance.audit]
enabled = true
dry_run = true      # leave this on for a pass or two first
min_age_seconds = 3600
```

Four checks:

- **Orphaned `Frames` and `Stats`** whose event is gone. Pure garbage; nothing
  can reach it and `Frames` is usually the largest table in the database.
- **Empty events** that never recorded a frame and are older than
  `min_age_seconds`.
- **Unclosed events** left by a capture daemon that died mid-recording. End
  time, length, frame count and score totals are recomputed from the frames
  that did land, and the event is marked `Recovered.` so the repair is visible.
  An update, never a delete.
- **Counter drift** in `Event_Summaries` and `Storage.DiskSpace`, recomputed
  from the rows they summarise.

### Two deliberate differences from `zmaudit.pl`

**Archived events are genuinely skipped.** zmaudit intends to skip them when
deleting frameless events, but the column it tests is not in its `SELECT` list,
so the guard never fires and it deletes them. Archiving an event is a user
saying *keep this*.

**`dry_run` means dry.** zmaudit's `--report` suppresses its deletes but still
performs row updates, empty-directory removal, stray-image unlinking, log
pruning and counter resyncs — so "just report" is not what it does. Here
nothing is written at all.

### Not implemented yet: the filesystem half

zmaudit also reconciles event *directories* against `Events` rows in both
directions. That is not here yet, and it is the part worth being careful with:
zmaudit derives its `rm -rf` target from `StartDateTime` formatted in the
process's **local timezone**, so a timezone mismatch between the recording
daemon and the auditor points it at a directory that was never the event's.

Keep running `zmaudit.pl` if you rely on orphaned-directory cleanup. The
[retention reaper](retention.md) already bounds disk usage for events the
database knows about, which is the more common need.

## Telemetry — replaces `zmtelemetry.pl`

```toml
[maintenance.telemetry]
enabled = true
interval_seconds = 1209600  # 14 days
```

Off unless explicitly enabled, and it stays off.

**No geolocation lookup.** `zmtelemetry.pl` calls `ipinfo.io` on *every*
collection — including when you only asked to preview the payload — and reports
city, region, country and latitude/longitude. That discloses the server's public
IP to a third party under the heading of anonymous statistics. Those fields are
still sent so the receiving end sees the shape it expects, but they are always
`Unknown`.

Camera paths are scrubbed before they leave the machine: credentials **and**
hostname are replaced, keeping only the scheme and path shape. A remote path
that is not a parseable URL is dropped entirely rather than forwarded.

The interval comes from `ZM_TELEMETRY_INTERVAL`, which ZoneMinder ships as the
Perl expression `14*24*60*60` and evaluates as code. Here it is parsed as a
number or a product of numbers; anything else falls back to the configured value
and logs a warning.

## Watching it

```bash
journalctl -u zm-api -f | grep -iE 'audit|stats|telemetry'
```

The audit logs a line per pass summarising what it found, and says explicitly
when it is in dry-run mode.
