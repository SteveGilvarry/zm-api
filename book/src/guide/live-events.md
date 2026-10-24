# Live events

`GET /api/v3/events/stream` pushes what happens on your cameras as it happens:
events starting and ending, their notes changing, and each camera's alarm
state. Home Assistant, dashboards and detectors can react within a fraction of
a second, without ZoneMinder's MQTT and without polling `GET /events`.

It is a [Server-Sent Events](https://html.spec.whatwg.org/multipage/server-sent-events.html)
stream: plain HTTP that stays open. Send a header token:

```
curl -N -H "Authorization: Bearer $TOKEN" https://zm.example/api/v3/events/stream
event: alarm_state
data: {"type":"alarm_state","monitor_id":1,"state":"idle","score":0}

event: event_start
id: 34735
data: {"type":"event_start","event_id":34735,"monitor_id":1,"name":"Event-34735","cause":"Motion",...}
```

The stream covers every monitor you may see. Add `?monitor_id=3` for one.

| Message | When |
|---|---|
| `alarm_state` | For every monitor when you connect, then whenever one changes: `idle`, `prealarm`, `alarm`, `alert`, or `unknown` (capture not running). Carries the last frame's `score`. |
| `event_start` | An event opens. |
| `event_end` | It closes: `end_date_time`, `frames`, `alarm_frames`, `max_score` and `length` are final. |
| `event_updated` | Its `notes` changed. Object detection (zmeventnotification, pyzm) writes results such as `detected:person:97%` there. |
| `lagged` | You fell behind and missed messages; reconnect with `Last-Event-ID`. |

Event messages carry `event_id`, `monitor_id`, `name`, `cause`, `notes`,
`start_date_time`, `end_date_time` (null while open), `frames`,
`alarm_frames`, `max_score` and `length` (seconds).

## Reconnecting

Each event message's SSE `id` is its event id. Reconnect with
`Last-Event-ID: <id>` (browsers' `EventSource` does this for you) or
`?since=<id>`, and the events after it, up to 500, are replayed before the
live ones. Alarm states can't be replayed, but every monitor's current state is
sent on connect. Expect the same event more than once across a reconnect; key
on `(event_id, type)`.

The stream ends when your access token expires and when you lose access
(logout, a disabled account, losing Events or the monitor; checked every
minute). Reconnect with a fresh token and `Last-Event-ID`. A comment line
arrives every 15 seconds, so a silent connection is a dead one.

## How it works

One watcher in zm-api serves every subscriber and runs only while someone is
connected. Alarm states come from each camera's shared memory, four times a
second; a camera's new event wakes the watcher at once. Events come from
ZoneMinder's `Events` table, so cameras on other ZoneMinder servers appear too,
within a second or so. Nothing is written to the database.
