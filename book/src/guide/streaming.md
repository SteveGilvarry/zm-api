# Live streaming

Two delivery paths from one API, both sourced from zmc's per-monitor stream
socket — a single unix socket carrying video and audio with a HELLO codec
handshake.

Everything under `/api/v3/live/{monitor_id}` requires `Stream: View` and passes
the row-level monitor ACL.

## HLS

Works in any HTML5 `<video>` element, including Safari natively.

```
GET /api/v3/live/{monitor_id}/hls/master.m3u8
GET /api/v3/live/{monitor_id}/hls/live.m3u8
GET /api/v3/live/{monitor_id}/hls/init.mp4
GET /api/v3/live/{monitor_id}/hls/{segment}
```

Fragmented MP4, with low-latency HLS available. Higher latency than WebRTC but
far easier to proxy and cache.

## WebRTC

Lower latency, at the cost of a signalling handshake and UDP.

```
GET /api/v3/live/{monitor_id}/webrtc/ws     (WebSocket)
```

The media itself travels over OS-assigned **ephemeral UDP ports** — there is no
configurable range, which matters before putting the media path through a
firewall or NAT.

For viewers behind symmetric NAT, STUN is not enough and you need a TURN server:

```toml
[streaming.webrtc.turn]
enabled = true
server = "turn:turn.example.com:3478"
username = "zm"
password = "…"
```

Media is relayed through TURN, so size it accordingly.

## Session control and snapshots

```
POST   /api/v3/live/{monitor_id}/start
DELETE /api/v3/live/{monitor_id}/stop
GET    /api/v3/live/{monitor_id}/stats
GET    /api/v3/live/sessions
GET    /api/v3/live/sources
GET    /api/v3/monitors/{monitor_id}/snapshot
```

The snapshot route accepts `?token=<JWT>` because `<img>` cannot set headers.

## Still images are rotated for you

`/events/{id}/thumbnail` and `/monitors/{id}/snapshot` apply the monitor's
`Orientation` before returning the JPEG, matching what ZoneMinder's own image
view serves. A client can size and render the result directly.

This is deliberately **stills only**. Rotating live video would mean
re-encoding the stream; a client can do it in CSS for nothing, so WebRTC and HLS
hand back the camera's own frames.

`ROTATE_0` — almost every camera — still streams the bytes straight off disk
with no decode.

## Recorded playback

```
GET /api/v3/events/{id}/video
GET /api/v3/events/{id}/stream/video.mp4
GET /api/v3/events/{id}/stream/playlist.m3u8
GET /api/v3/events/{id}/thumbnail
```

These need `Events: View`, not `Stream`.

## Behind a reverse proxy

Streaming routes are deliberately excluded from zm-api's own compression layer,
and they must not be buffered either. With `proxy_buffering on` an HLS or MP4
route stalls. See [Serving a dashboard](dashboard.md) for a working nginx
config.

## When streams fail

`permission denied opening stream socket …` in the log means the service user
is not in ZoneMinder's `ZM_STREAM_SOCKET_GROUP`. The sockets are mode 0660. Fix
with a systemd drop-in — see [Troubleshooting](../reference/troubleshooting.md).
