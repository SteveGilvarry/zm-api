# Recording retention

Automatic cleanup that deletes whole events — media *and* database rows —
oldest first, per Storage, when a limit is breached. It replaces ZoneMinder's
`PurgeWhenFull` filter.

**Off by default.** Enable it in `prod.toml`:

```toml
[retention]
enabled = true
```

## What it will not delete

- Archived events
- Events still being recorded
- The newest event for each monitor

That last one means a monitor never ends up with no footage at all, however
tight the limits.

## Limits

Configured per Storage. Any breach triggers reaping:

| Limit | Meaning |
| --- | --- |
| Free-space floor | Keep at least this much free on the volume |
| Age | Delete events older than this |
| Quota | Cap total bytes for this Storage |

## Media deletion

The reaper and the `DELETE /api/v3/events/{id}` endpoint share one code path,
so both remove the same things: the event's directory on disk plus its `Frames`,
`Events_Hour`/`Day`/`Week`/`Month`, and `Events_Archived` rows, transactionally.
No foreign key cascade covers those, which is why it is centralised.

On-disk paths honour the per-event `Storage` row and ZoneMinder's Deep, Medium,
and Shallow layout schemes.

## Watching it

```bash
journalctl -u zm_api -f | grep -i reap
```

Every deletion is logged with the event id and the limit that triggered it.
Start with generous limits and read the log for a cycle or two before tightening
them — deletion is not reversible.
