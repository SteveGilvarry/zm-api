# Permissions

Two independent layers apply to every request: **feature-level RBAC** decides
whether you may touch a kind of resource at all, and **row-level ACLs** decide
which specific monitors and groups you see.

## Feature-level RBAC

ZoneMinder accounts carry eight permission columns. zm-api enforces all of them,
deriving the required level from the HTTP method — reads need `View`, writes
need `Edit`.

| Feature | Gates |
| --- | --- |
| `Stream` | Live video (WebRTC, HLS, snapshots) |
| `Events` | Event list, playback, frames, filters, tags, search |
| `Control` | PTZ, control presets, X10 triggers |
| `Monitors` | Monitors, zones, monitor presets, ONVIF discovery |
| `Groups` | Groups and group membership |
| `Devices` | Devices, manufacturers, models |
| `Snapshots` | Snapshots |
| `System` | Config, logs, storage, users, servers, reports, daemon control, AI registry |

`Stream` has no `Edit` tier in ZoneMinder — `View` is the maximum.

Granting permissions is deliberately `System`-tier, not the tier of the thing
being granted, so a `Groups:Edit` user cannot grant themselves more.

## Discovering your own permissions

`GET /api/v3/me` returns all eight columns and is **not** feature-gated, so it
works for any authenticated account including `System: None`.

This matters. The CakePHP API had no way to ask "what am I allowed to do" — a
client had to fetch `/users.json` and find its own row, which was itself gated
on `System != 'None'`. An ordinary operator got a 401 and could infer only that
one column. `System: None` with `Monitors: Edit` is a perfectly legal
ZoneMinder account, and such a user could not discover its own monitor
permissions at all.

So gate your UI on `/me`:

```json
{
  "user": {
    "username": "operator",
    "system": "None",
    "stream": "View",
    "events": "View",
    "control": "None",
    "monitors": "Edit",
    "groups": "View",
    "devices": "None",
    "snapshots": "View"
  }
}
```

## Row-level ACLs

On top of the feature check, `Monitors_Permissions` and `Groups_Permissions`
filter individual rows per user. Two accounts with identical feature
permissions can get different results from the same endpoint.

This is default-allow for backward compatibility: a user with no explicit
row-level entries sees everything their feature permissions allow.

For routes naming a monitor in the path — PTZ, live streaming — the row-level
guard runs *inside* the feature check, so a caller without the feature is
refused before any database query happens.

## What a denial looks like

| Status | Meaning |
| --- | --- |
| `401` | No token, expired token, or a revoked one |
| `403` | Authenticated, but the account lacks the feature or level |
| `404` | May exist, but row-level ACLs hide it from this account |

The 404 is deliberate — a 403 would confirm the resource exists.
