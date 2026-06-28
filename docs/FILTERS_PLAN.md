# Filters — Refactor / Retirement Plan

**Status:** Active (design) — 2026-06-28. No code yet.

The `Filters` table conflates five unrelated concerns under one row format.
The plan: split each concern out into a dedicated subsystem, retire the
user-facing concept of "filter", and remove the `zmfilter.pl --filter_id=N
--daemon` supervision path.

> Related: [STORAGE_PLAN.md](STORAGE_PLAN.md) covers (D) housekeeping; this
> doc covers (A)-(C) and (E). Built-in filters → storage manager; user
> filters → the new subsystems below.

## Inventory (the headline finding)

A fresh ZoneMinder install seeds **exactly two** filters
(`db/zm_create.sql.in`):

| Filter | Purpose | Auto-action |
|---|---|---|
| `PurgeWhenFull` | Delete oldest 100 non-archived completed events when disk ≥95% full | `AutoDelete` |
| `Update DiskSpace` | Backfill `Events.DiskSpace` for completed events where it's NULL | `UpdateDiskSpace` |

That's it. Exhaustive search of `INSERT INTO Filters` across the entire
ZoneMinder repo (master, all `db/zm_update-*.sql`) returns only those two +
their migration scaffolding. There is no third built-in filter anywhere.

User-defined filters exist, but they all reuse the same `AutoX` action
vocabulary.

## Auto-action taxonomy (5 categories)

| Category | Auto-actions | New home |
|---|---|---|
| **(A) Per-event mutation** | `AutoArchive`, `AutoUnarchive`, `AutoDelete` | **Saved searches + bulk event ops in zm-api.** |
| **(B) Notification** | `AutoEmail`, `AutoMessage` | **Notification rules engine.** |
| **(C) Media side-effect** | `AutoVideo`, `AutoUpload`, `AutoMove`, `AutoCopy` | **Media job queue.** |
| **(D) Disk housekeeping** | `UpdateDiskSpace`, `PurgeWhenFull` (`AutoDelete` on the seeded filter only) | **Storage manager** — see [STORAGE_PLAN.md](STORAGE_PLAN.md). |
| **(E) Arbitrary exec** | `AutoExecuteCmd` | **Webhooks** (replace; do not port). |

Coverage in zm-api today (as of 2026-06-28, `master @ 766c1a7`):

| Auto-action | Today | Gap |
|---|---|---|
| `AutoArchive` / `AutoUnarchive` | `PATCH /events/{id} {archived: true/false}` — `src/service/events.rs:230-232`. Covered. | Bulk endpoint missing. |
| `AutoDelete` | `DELETE /events/{id}` — `src/repo/events.rs:180`. Leaky: drops DB row but leaves on-disk dir/frames/snapshot/mp4. | Two-stage soft-delete + reap (see STORAGE_PLAN P0+P1) **before** bulk-delete is safe. |
| `AutoVideo` | None. `videoed` not even on `EventUpdateRequest`. | New media job kind. |
| `AutoUpload` | Flag-only: `PATCH {uploaded: true}` (`src/service/events.rs:246-248`). No FTP/SFTP/cloud client. | Move to media job queue (cold-tier handoff covers most real use cases anyway). |
| `AutoEmail` | Flag-only: `PATCH {emailed: true}` (`src/service/events.rs:238-240`). No SMTP sender. | Notification rules engine. |
| `AutoMessage` | Flag-only: `PATCH {messaged: true}`. Despite the name, this is an email-to-SMS pathway upstream — not Jabber/XMPP. | Notification rules engine, with explicit channel selection. |
| `AutoExecute` / `AutoExecuteCmd` | Flag-only. `auto_execute_cmd` is stored on the filter row but never executed (no `Command::new` in zm-api outside ffmpeg/snapshot). | **Drop entirely.** Webhooks cover every legitimate use and are auditable. |
| `AutoMove` / `AutoMoveTo` | None. PATCH does not even expose `storage_id`. | Storage tier_mode + media job queue. |
| `AutoCopy` / `AutoCopyTo` | None. `secondary_storage_id` only settable at create time. | Storage tier_mode + media job queue. |
| `UpdateDiskSpace` | None. Column is read-only on responses. | Storage manager. |

## Subsystem 1: Saved searches + bulk event ops

The AST translator already exists (`src/service/filter_translate.rs`,
`filter_query.rs`, `filter_build.rs`) and the preview compiler emits a
parameterised SeaORM `Condition` (`src/service/filter_build.rs:1-8`).

What's needed:

- **`saved_searches` table**: `(id, user_id, name, ast_json, monitor_scope?,
  created_at)`. No auto-action columns. No execution columns. No
  `Background` / `Concurrent` / `LockRows`.
- **`POST /api/v3/events:bulk`** body `{ saved_search_id | ast, action:
  archive | unarchive | delete, dry_run: bool }`. Cursor-paginated execution
  with a debounce. Returns a job id; status polled via `GET
  /api/v3/jobs/{id}`.
- **`GET /api/v3/events:search`** wraps the existing preview path; trivial.

Critically: `DELETE /events/{id}` must be fixed first (see STORAGE_PLAN P0+P1
— soft-delete + reap). Bulk-delete on the current implementation silently
leaks storage at scale.

## Subsystem 2: Notification rules engine

A dedicated, channel-aware rules engine. Notifications are an
**independent concern** from filters; they need to fire on event-start /
event-end / score-threshold regardless of any saved search.

Shape:

| Field | Type |
|---|---|
| `id` | u64 |
| `user_id` | FK Users |
| `name` | string |
| `predicate_ast` | JSON | reuses the same AST as saved_searches |
| `trigger` | enum `EventStart \| EventEnd \| ScoreThreshold \| StorageHealth` |
| `channel` | enum `Smtp \| Webhook \| Fcm \| Apns \| Mqtt` |
| `template` | JSON | per-channel render template |
| `dedupe_window_seconds` | u32 |

The `emailed` / `messaged` columns on `Events` become **per-rule delivery
state** (new table `notification_deliveries`, FK to rule + event) rather
than per-event flags. Lossy single-flag semantics in upstream were a bug
anyway — a rule may fire on Telegram but not on email; the current model
can't represent that.

Channels are pluggable. SMTP is the obvious first (covers `AutoEmail` +
`AutoMessage` via the SMS-gateway pattern). Webhook is the catch-all that
replaces `AutoExecuteCmd`.

## Subsystem 3: Media job queue

`POST /api/v3/events/{id}/jobs` body `{ kind: transcode | export | move |
copy, target_storage_id?, destination? }`. Workers consume the queue with
bounded concurrency. The `videoed` / `uploaded` columns become job-completion
markers.

`AutoMove` / `AutoCopy` use cases mostly fold into the storage tier_mode
(see STORAGE_PLAN.md) — operator declares a cold-tier handoff once, all
clips migrate automatically. The job queue is the manual escape hatch for
"export this clip to that S3 bucket" or "transcode this event to mobile-
friendly MP4".

## Subsystem 4: Webhooks (replaces AutoExecuteCmd)

`qx($command)` with `%TAG%` substitution and no escaping (zmfilter.pl's
implementation) is a stored-RCE primitive. Existing rows in production are
already a security liability.

Replacement:

- Notification channel `Webhook` (see Subsystem 2).
- HMAC-signed POST body with event JSON.
- Per-rule URL + secret.
- Audit log of every invocation.

`AutoExecute=1` on filter create/update returns 400 with a deprecation
message linking to webhooks. Existing rows are warned about at startup.
Two-release deprecation window, then drop.

## Migration of existing user filter rows

A one-shot importer routes each existing `Filters` row by which `Auto*`
flags are set:

| Has flags | Routed to |
|---|---|
| Only `AutoArchive` / `AutoUnarchive` / `AutoDelete` | Saved search; can be invoked via bulk endpoint. |
| Only `AutoEmail` / `AutoMessage` | Notification rule (Smtp channel). |
| Only `AutoUpload` / `AutoMove` / `AutoCopy` / `AutoVideo` | Media job template attached to a saved search (queued by scheduler — see open Q). |
| `AutoExecute=1` | Hard-rejected on import; warned at startup; operator must migrate to webhook. |
| Mixed across categories | Split into N rows (one per category) — emit a migration report listing the splits for operator review. |

The seeded `PurgeWhenFull` and `Update DiskSpace` rows are deleted on
import (their job moves to the storage manager).

## What "scheduler" actually meant (for the original GitHub issue)

ZoneMinder upstream issue #4957 asks for a "full-fledged scheduler" running
Perl/PHP/Bash/binary scripts on cron-like schedules with a UI.

Reframed against this plan:
- **Time-of-day arming** → already solvable via `Monitors.Function` +
  scheduled API calls (host cron + a couple of REST calls). Not a backend
  feature, frontend at most.
- **Periodic housekeeping** → storage manager (no cron).
- **Scheduled exports/reports** → notification rules with `EventEnd` trigger
  + daily-digest channel (open question 5 in STORAGE_PLAN.md style).
- **Maintenance windows** → notification rule with a time-of-day predicate
  + a state-change action.
- **Long-running PHP/worker** → daemon supervisor, not a scheduler. zm-api's
  `DaemonManager` already supervises ZoneMinder daemons; could grow a
  user-defined-daemons table if there's real demand. Out of scope for this
  plan; tracked separately if requested.

There is no general cron + script-runner to build.

## Implementation phases

| Phase | Scope | Acceptance |
|---|---|---|
| **P0 — Saved searches** | Schema + CRUD. AST shape reuses existing `filter_translate`. No `Auto*` columns. | `GET /events:search` works; saved searches list/CRUD. |
| **P1 — Bulk event ops** | `POST /events:bulk` for archive/unarchive/delete. Depends on STORAGE_PLAN P0+P1 (soft-delete + reap) for delete to be safe. | Bulk archive 10k events in one call; delete leaks no storage. |
| **P2 — Notification rules** | Rules table, deliveries table, SMTP + Webhook channels. Trigger on event-end. | "Email me when person detected on driveway" works end-to-end. |
| **P3 — Media job queue** | Jobs table, worker pool, transcode + export + move + copy kinds. | Manual export to S3 works; storage tier_mode auto-tiering covers `AutoMove`. |
| **P4 — Importer** | One-shot migration of existing `Filters` rows. Migration report. | All seeded + user filters migrated; zmfilter.pl supervision can be turned off per Storage. |
| **P5 — Retire zmfilter.pl** | Remove `src/daemon/manager.rs:1031-1065` background-filter spawn loop. `AutoExecute=1` hard-rejected on create/update. | No `zmfilter.pl --daemon` processes; `Filters` table marked legacy. |
| **P6 — (Future) Daemon supervisor for user workers** | Only if real demand for long-running PHP/Python workers. `DaemonManager` extension. | Out of scope unless requested. |

## Open questions

1. **Rule predicate vs event API filter.** Both share the same AST. Do they
   share the same SQL translator (single source of truth), or do rules use a
   real-time event matcher (no SQL) for low latency? Lean: real-time matcher
   for rules (events flow through it as they're created), AST shared.
2. **`AutoExecute` deprecation path.** Hard-error on existing rows immediately,
   or warn-and-ignore for one release? Security argues immediate; ops continuity
   argues warning. Lean: warn-and-ignore for one release, hard-error in the
   next.
3. **Mixed-category rows on import.** Split silently, or require operator
   intervention? Lean: split silently, emit a report; operator can revert.
4. **`Filters` table retention.** Keep around after P5 as legacy storage (in
   case upstream web UI still writes to it), or drop the table entirely? Lean:
   keep + mark read-only on the API for one release after P5.
5. **Notification delivery-state retention.** Per-rule per-event deliveries
   could grow large. Retention TTL on the deliveries table — 30 days?

## References

- Inventory + behavior of every `Auto*` action (the long form): the
  filter-inventory workflow output from 2026-06-28. Key sources: ZoneMinder
  `db/zm_create.sql.in`, `scripts/zmfilter.pl.in`,
  `docs/userguide/filterevents.rst`.
- Current zm-api filter code: `src/handlers/filters.rs`,
  `src/service/filters.rs`, `src/repo/filters.rs`, `src/entity/filters.rs`,
  `src/service/filter_{ast,translate,query,build,field}.rs`.
- Current zm-api supervision of zmfilter.pl:
  `src/daemon/manager.rs:1031-1065`, `src/daemon/daemons.rs:28-39`.
- The "delete leaks files" issue: `src/repo/events.rs:180-181` (`Events::delete_by_id`).
- ZoneMinder upstream issue #4957: the scheduler ask.
