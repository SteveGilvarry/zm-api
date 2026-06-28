# Storage Manager — Design Plan

**Status:** Active (design) — 2026-06-28. No code yet.

Replaces ZoneMinder's two seeded background filters (`PurgeWhenFull`,
`Update DiskSpace`) with a zm-api–owned storage subsystem. Eliminates the
`zmfilter.pl --filter_id=N --daemon` supervision path for the only filters
that ship by default, and gives operators per-camera retention SLAs without
ever editing a `Filters` row.

> Related: [FILTERS_PLAN.md](FILTERS_PLAN.md) (retiring user-facing Filters).
> Without the storage manager, the auto-delete category in FILTERS_PLAN is
> homeless.

## Motivation

1. **`PurgeWhenFull` is fragile UX.** It's a user-editable row in `Filters`
   that, mis-edited, deletes recordings. Upstream has shipped three migrations
   to repair this one filter (`zm_update-1.19.1`, `1.23.x`, `1.35.15` — the
   last one added `EndDateTime IS NOT NULL` to stop it deleting in-progress
   events).
2. **The responsibility is a storage invariant, not user policy.** "Don't let
   the disk fill" belongs on the `Storage` row, not in a query language.
3. **`statvfs()` is the wrong primitive.** Shared storage, NFS, S3-backed
   storage and over-provisioned LVM all lie about "% full". A **declared
   capacity** per Storage decouples retention from filesystem geometry and is
   testable without filling a real disk.
4. **Per-camera quotas don't exist today.** A chatty camera can evict all the
   others; there's no way to say "keep this important camera 30 days, others 7
   days" without writing a filter per camera.

## Data model

### `Storage` (extend the existing entity)

| Field | Type | Purpose |
|---|---|---|
| `kind` | enum `Local \| S3 \| NFS \| SMB \| WebDAV \| Azure \| GCS` | Backing type. Drives reaper/move/copy implementations. |
| `capacity_bytes` | u64 | Declared budget. `DiskPercent = sum(Events.DiskSpace where StorageId=X) / capacity_bytes`. Not `statvfs()`. |
| `high_water_pct` | u8 | Retention triggers above this. Default 90. |
| `low_water_pct` | u8 | Retention runs until back below this. Default 75. |
| `max_age_days` | u32? | Optional unconditional age cap. Null = unlimited. |
| `protect_archived` | bool | Archived events are never auto-deleted. Default true. |
| `protect_locked` | bool | Locked events are never auto-deleted. Default true. |
| `secondary_storage_id` | FK Storage? | Cold tier target (S3, slower disk, etc.). |
| `tier_mode` | enum `Off \| Backup \| Tier \| BackupThenTier` | What to do with the secondary. |
| `tier_age_days` | u32? | Unconditional age-based tier-down regardless of pressure. |
| `encryption` | enum `None \| ServerSide \| ClientSide(key_id)` | At-rest encryption for the backing. |
| `full` | bool (runtime) | Set when archived events fill the quota and recording must stop. Surfaced in API + alerts. |

`tier_mode` semantics:
- **Backup**: copy new clips to secondary; keep originals. Replaces `AutoCopy`.
- **Tier**: when over high-water, move oldest events to secondary, then delete
  from primary. Replaces `AutoMove`.
- **BackupThenTier**: backup new clips immediately, then tier on pressure.
  Data is safe on the secondary before the primary is ever pressured.

### `Monitor` (extend)

| Field | Type | Purpose |
|---|---|---|
| `max_storage_bytes` | u64? | Per-camera quota on its primary Storage. Null = share with others. |
| `max_age_days` | u32? | Per-camera age override (defaults to `Storage.max_age_days`). |
| `protect_archived` | bool? | Per-camera override. |

### Invariants enforced at the API boundary

- **No oversubscription.** When creating/updating a Monitor with
  `max_storage_bytes`, the server validates that
  `sum(Monitor.max_storage_bytes WHERE storage_id=X) ≤ Storage.capacity_bytes`.
  Reject with 400 on overflow. Apologetic but firm — "why was my camera
  evicted" is much harder to explain in an oversubscribed model.
- **Archived events are immortal.** Retention skips them; they count against
  the quota. If they fill it, recording fails closed: `Storage.full = true`,
  `zmc` errors clearly, an operator alert fires. "Silent stop" is the worst
  failure mode and must be impossible.

## Reaper (the delete contract)

PHP/ZoneMinder learned the hard way that synchronous `unlink()` over
thousands of small per-event JPEGs is a foot-gun. The reaper is **two-stage**:
soft-delete + async reap.

1. **API delete** = flip `Events.deleted_at` (new column). The event is
   instantly hidden from list/get; the row is not removed. Cheap, atomic,
   restart-safe — state lives in the DB.
2. **Reaper task per Storage** consumes tombstones and reclaims disk:
   - **Rename-into-trash first.** `rename("events/.../1234",
     "events/.trash/1234")` is one atomic syscall on the same filesystem and
     the event vanishes from the live tree immediately. Long walk happens
     later in `.trash/`, where nothing else is contending.
   - **`tokio::fs::remove_dir_all` with bounded concurrency.** Native
     syscalls; cap at N concurrent event-dir reaps so live recording IO is
     never starved.
   - **Maintenance window.** Hard reaping pauses when any monitor is `Modect`
     and only runs aggressively when retention is pressed.
   - **Per-storage scheduling.** Slow NFS can't stall local-disk cleanup.
   - **Trash lives on the same Storage** (`.trash/` subdir). Cross-fs rename
     would be a copy, defeating the point.

### Cross-kind move/copy

| From → To | Implementation |
|---|---|
| Local → Local (same fs) | `rename` |
| Local → Local (cross fs) | copy + verify + delete |
| Local → S3/Azure/GCS | streamed multipart upload; verify checksum; delete |
| Local → NFS/SMB/WebDAV | streamed copy via existing FS abstraction |
| S3 → Local | streamed download (handles Glacier restore — see open question 2 in `FILTERS_PLAN.md`) |

## Retention runner

A periodic task per `Storage` row:

1. Stat-walk new event dirs; recompute `Events.DiskSpace`. (Replaces
   `UpdateDiskSpace` filter.)
2. Compute per-monitor and per-storage usage from the column.
3. If any monitor is over its `max_storage_bytes`, evict oldest unprotected
   events on that monitor until back under, oldest first.
4. If the Storage is over `high_water_pct`, evict oldest unprotected events
   across all monitors until under `low_water_pct`.
5. If `tier_mode != Off` and a tier-down is configured, move/copy eligible
   events to `secondary_storage_id` per the mode and `tier_age_days`.
6. If the Storage is full of archived events, set `Storage.full = true` and
   emit a health alert (notification rules engine — see `FILTERS_PLAN.md`).

The runner is **idempotent** — re-running it must converge. State lives in
the DB; partial work is fine.

## Storage `kind` adapters (the only kind-specific code)

Each `kind` implements a small trait:

```rust
trait StorageBackend {
    async fn write_event(&self, event_id: u64, src: &Path) -> Result<PathBuf>;
    async fn rename_into_trash(&self, event_id: u64) -> Result<()>;
    async fn reap_trash(&self, event_id: u64) -> Result<()>;
    async fn recompute_disk_space(&self, event_id: u64) -> Result<u64>;
    async fn copy_to(&self, event_id: u64, dst: &dyn StorageBackend) -> Result<()>;
    async fn move_to(&self, event_id: u64, dst: &dyn StorageBackend) -> Result<()>;
}
```

`Local` is the baseline. `S3/Azure/GCS` reuse the same multipart upload/download
path with provider-specific auth. `NFS/SMB/WebDAV` reuse `Local` via the mount.

## API surface

| Endpoint | Purpose |
|---|---|
| `GET /api/v3/storages` | List with computed usage (used/declared, per-monitor breakdown). |
| `PATCH /api/v3/storages/{id}` | Update capacity, water marks, retention, tier config. Validates monitor-sum invariant on capacity changes. |
| `POST /api/v3/storages/{id}:reap` | Force-run the reaper (operator escape hatch). |
| `GET /api/v3/storages/{id}/health` | `{full, used_bytes, capacity_bytes, archived_bytes, oldest_event_ts}`. |
| `PATCH /api/v3/monitors/{id}` | (extend) accept `max_storage_bytes`, `max_age_days`, `protect_archived`. Validates oversubscription. |
| `DELETE /api/v3/events/{id}` | Already exists; change semantics to set `deleted_at` rather than `Events::delete_by_id`. |

`Storage.full=true` surfaces as a health condition on `GET
/monitors/{id}/events` SSE (see `MONITOR_EVENTS_TASKS.md`).

## Migration from PurgeWhenFull + Update DiskSpace

1. Ship the storage manager off by default (feature flag in `[storage]`).
2. For each Storage with the default `PurgeWhenFull` row, write reasonable
   defaults (`high_water_pct=90`, `low_water_pct=75`, `protect_archived=true`)
   and stop supervising the `zmfilter.pl --filter_id=<purge_id> --daemon`
   process for it.
3. After one release where both systems are valid (one of them must be
   disabled per Storage to avoid double-reaping), drop the two seeded filter
   rows from `db/zm_create.sql.in` (zm-api's mirror) and remove the spawn
   path for them in `src/daemon/manager.rs:1031-1065`.
4. Document the upgrade in `deployment.md`.

## Implementation phases

| Phase | Scope | Acceptance |
|---|---|---|
| **P0 — schema + soft-delete** | SeaORM migration: `Events.deleted_at`, Storage extensions, Monitor extensions. Repo + service for soft-delete. List/get hide tombstoned. | `DELETE /events/{id}` is instant + idempotent; tombstoned events vanish from API. |
| **P1 — Local reaper** | Per-Storage reaper task; `rename`-into-trash; bounded concurrency; maintenance window. | Reap throughput benchmarked; recording IO unaffected under load; tests for restart-safety. |
| **P2 — Local retention runner** | Stat-walk → DiskSpace; per-monitor + per-storage eviction; oversubscription validation at API. | Replaces `PurgeWhenFull` + `Update DiskSpace` for the seeded `Local` storage. Side-by-side test against `zmfilter.pl` for one release. |
| **P3 — Health + alerts** | `Storage.full` flag; recording-fails-closed; SSE health event on the Monitor stream. | Operator gets notified before silent stop; integration test for "fill with archived, then attempt new recording". |
| **P4 — S3 backend + tier_mode** | S3 `StorageBackend`; `Backup`/`Tier`/`BackupThenTier`; multipart upload; lifecycle-rule offload (open Q below). | Cold tier verified end-to-end; restore-on-playback path tested. |
| **P5 — Other backends** | Azure, GCS, NFS, SMB, WebDAV adapters as demand justifies. | One per release as needed. |

## Open questions

1. **Reuse S3 lifecycle policies?** If `tier_mode=Tier` and `secondary` is S3,
   we can let AWS reap via lifecycle rules — `max_age_days` becomes a declared
   policy zm-api applies to the bucket once. Cheaper, but couples our
   retention to S3 semantics. Lean: yes.
2. **Glacier/Deep Archive playback latency.** Event tiered to Glacier
   requested for playback — initiate restore + 202 the request, or hard error
   with "cold storage"? Affects UI.
3. **Encryption defaults.** Per-Storage `encryption: None | ServerSide |
   ClientSide(key_id)`? Server-side is the sensible default; client-side adds
   key management.
4. **`deleted_at` retention.** How long do tombstones stick around before the
   row is hard-deleted from `Events`? Long enough for `zmaudit`/operator
   recovery; short enough that the table doesn't bloat. Lean: 30 days.
5. **Frames/Stats FK cascade.** Does the upstream schema already cascade on
   `Events` delete, or do we need explicit cleanup? Verify before P0.
6. **Built-in filter protection on rollback.** If the seeded `PurgeWhenFull`
   row is dropped at P2 completion, do we also block the upstream web UI from
   recreating it (it still seeds the row in `zm_create.sql.in`)?

## References

- ZoneMinder upstream filters: `db/zm_create.sql.in` lines 1085-1168
  (the two seeded rows).
- Current zm-api supervision: `src/daemon/manager.rs:1031-1065`.
- Current (leaky) delete: `src/repo/events.rs:180-181` — `Events::delete_by_id`
  drops the row but leaves the on-disk event dir, frames, snapshot, mp4.
  This is the gap P0+P1 closes.
- Filter context for why this is the right home: see [FILTERS_PLAN.md](FILTERS_PLAN.md).
