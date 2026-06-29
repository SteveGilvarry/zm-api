# Event Retention Plan

Status: **Design — not yet implemented.**
Author: design draft for review.
Context: today the disk-fill incident (root `/` hit 94%) was only caught manually.
zm-next's `store` plugin records continuously and **nothing deletes old events** —
neither the plugin (no DB) nor zm-api. ZoneMinder's answer is *purge filters*
(`Filters` table, `zmfilter`), which the team considers clunky. This proposes a
small, self-contained retention reaper inside zm-api.

## Goals

- Bound disk usage automatically, per storage location, with no operator babysitting.
- Delete *whole events* atomically: media files **and** the DB rows that point at
  them (`Events` + `Frames` + cascades), so the UI never shows 404 playback.
- Be obviously-correct and easy to reason about — a few numeric knobs, not a
  filter-expression DSL.
- Never delete in-progress or archived events.

## Why zm-api (not the store plugin, not ZM filters)

- **store plugin** has no DB handle. If it pruned files it would orphan `Events`
  rows (exactly the mess cleaned up by hand today). Rejected.
- **ZM purge filters** are expressive but opaque: cron-ish `zmfilter`, filter rows,
  `UpdateDiskSpace` flags, action coupling. Hard to predict what gets deleted.
- **zm-api** already owns the DB (SeaORM entities for `events`, `storage`,
  `events_hour/day/week/month`, `events_archived`), already supervises the workers
  (`DaemonManager`), and already resolves each monitor's storage root
  (`manager.rs::zmnext_events_root`). Retention belongs next to that.

## Policy model

Per **Storage** row, evaluated independently (each storage may live on a different
filesystem). Three independent limits; an event is eligible for deletion if **any**
limit is exceeded, deleting oldest-first until **all** limits are satisfied:

| Knob | Meaning | Example |
|---|---|---|
| `min_free_pct` | Keep the filesystem holding this storage at/below `100 - min_free_pct` used. The safety net that actually prevents disk-full. | 10 (keep ≥10% free) |
| `max_bytes` | Hard cap on bytes this storage may hold (sum of `Events.DiskSpace`). 0 = unlimited. | 50 GiB |
| `max_age_days` | Delete events older than N days regardless of space. 0 = disabled. | 30 |

Reaping order within a storage: **oldest `StartDateTime` first**. Always skip:
- `Archived = 1`
- events with no `EndDateTime` (still recording / in-progress)
- the newest event per monitor (never leave a monitor with zero recent footage)

### Config surface

New section in `settings/*.toml`, with safe defaults (age/bytes off, free-floor on):

```toml
[retention]
enabled = true
interval_seconds = 300        # how often the reaper ticks
min_free_pct = 10             # global default; the disk-full safety net
max_bytes = 0                 # 0 = unlimited
max_age_days = 0              # 0 = disabled
dry_run = false              # log what WOULD be deleted, delete nothing
```

Optionally allow per-storage overrides later (a `Storage`-keyed map), but ship the
global default first — it covers the single-box case that bit us today.

## Algorithm (per tick)

```
for storage in Storage.find_all():
    fs   = statvfs(storage.path)                 # free %, total
    used = sum(Events.DiskSpace where StorageId resolves here)   # 0 and NULL → default storage
    candidates = Events
        .where(storage resolves here, Archived = 0, EndDateTime IS NOT NULL)
        .order_by(StartDateTime ASC)             # oldest first
        .exclude(newest event per monitor)

    target_deletions = []
    while over_limit(fs, used, policy) and candidates:
        e = candidates.pop_front()
        target_deletions.push(e)
        used -= e.DiskSpace or measured_size(e)

    if dry_run: log(target_deletions); continue
    for e in target_deletions:
        delete_event(e)     # see below — atomic file+DB
    refresh Storage.DiskSpace
```

`over_limit` = `free% < min_free_pct` OR (`max_bytes>0` AND `used>max_bytes`) OR
(`max_age_days>0` AND oldest candidate older than cutoff).

Note the **`StorageId IN (0, NULL)` → default (lowest-id) storage** rule, identical
to `zmnext_events_root`'s fallback — the reaper must use the same resolution or it
will mis-attribute space (this is why today's cleanup matched on `StorageId IN (0,1)`).

## `delete_event(e)` — the atomic unit

Reusable single-event deletion (also useful for a future manual "delete event" API):

1. Resolve the event's on-disk dir from `storage.path` + `storage.scheme` +
   `MonitorId`/date/`Id` (Medium = `<root>/<mon>/<YYYY-MM-DD>/<id>/`). Validate the
   path is inside `storage.path` (reuse `util::path::contains_traversal`).
2. DB delete in one transaction:
   - `DELETE FROM Frames WHERE EventId = ?`
   - `DELETE FROM Events_Hour/Day/Week/Month/Archived WHERE EventId = ?`
   - `DELETE FROM Events WHERE Id = ?` (cascades `Snapshots_Events`, `Events_Tags`)
3. After commit, `rm -rf` the event dir. (DB first so a crash leaves an orphan file,
   not an orphan row — files are reclaimable by an audit pass; rows are not.)
4. Decrement `Storage.DiskSpace` (and the `event_summaries` rollups if zm-api
   maintains them).

## Where it lives

- `src/service/retention/mod.rs` — policy + `reap_once()` (pure-ish, unit-testable
  against a seeded DB).
- `src/service/retention/delete.rs` — `delete_event()`.
- Spawned from the same place the `DaemonManager`/prewarm tasks start (a
  `tokio::spawn` loop on `interval_seconds`), gated by `[retention].enabled`.
- Config in `src/configure/` (new `RetentionConfig`, mirror existing sections).

## Safety / rollout

- Ship with `dry_run = true` default in `dev`, log a one-line summary per tick
  (`retention: storage=Home free=8% would delete 12 events / 1.4 GiB`). Flip to
  enforce after eyeballing a few ticks.
- Hard guard: never delete an event whose `EndDateTime IS NULL` or `Archived=1`,
  and never delete the newest event per monitor — even if a limit is still breached
  (log a warning instead; a single huge open event shouldn't be force-killed).
- Metrics: expose `retention_events_deleted_total`, `retention_bytes_reclaimed_total`,
  and per-storage free% so the disk-full condition is visible *before* it happens.

## Out of scope (later)

- Per-storage policy overrides / tiered storage (move-then-delete to a cheaper disk).
- A manual delete-event REST endpoint (would reuse `delete_event()`).
- Reconciliation/audit pass for orphan files vs rows (zmaudit equivalent).

## Immediate follow-ups surfaced by today's incident

- Monitors created with `StorageId = 0` fall back to the **lowest-id** storage
  (`zmnext_events_root`), which is `Default → /var/cache/zoneminder/events` on root.
  Either move Storage 1's path off root, or make the `/home` storage the lowest id /
  the default, so new monitors don't silently start filling root again.
- `segment_max_secs = 20` (dev) produces a lot of small events; fine for testing,
  but worth raising in prod to cut event churn.
