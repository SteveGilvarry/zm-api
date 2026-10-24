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

### Players that cannot send a header

FFmpeg, VLC, Safari's native player and Home Assistant request every URI a
playlist lists exactly as written, so they cannot use a header token. Give
them a signed link instead:

```
curl -X POST -H "Authorization: Bearer $TOKEN" \
  -H 'content-type: application/json' -d '{"ttl_seconds": 2592000}' \
  https://zm.example/api/v3/live/1/hls/link
# {"playlist_url":"/api/v3/live/1/hls/master.m3u8?st=…","token":"…","expires_at":…}

ffplay "https://zm.example/api/v3/live/1/hls/master.m3u8?st=…"
```

The link opens that monitor's live media and nothing else. Every URI the
playlist lists carries a signature valid for a few minutes, refreshed each
time the player refetches the playlist; child playlists carry the link itself.
Revoking it, and why each integration wants its own user, is covered under
[Media URLs](authentication.md#media-urls). Recorded events work the same way
with `POST /api/v3/events/{id}/stream/link`; their segment signatures last four
hours, enough to watch, pause and seek.

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

### WHEP: the viewer makes the offer

`POST /api/v3/live/{monitor_id}/webrtc/whep` is WebRTC the standard way
(WHEP, RFC 9725), for clients whose browser side makes the offer: Home
Assistant's camera WebRTC, OBS, WHEP players. Send the browser's SDP offer with
a header token:

```
curl -X POST -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/sdp' \
  --data-binary @offer.sdp -i https://zm.example/api/v3/live/1/webrtc/whep
# 201 Created
# Location: /api/v3/live/1/webrtc/whep/5b0c…
# Content-Type: application/sdp
# (the answer, with zm-api's ICE candidates in it)
```

- The answer carries zm-api's candidates, so a client that gathers its own
  before sending the offer needs nothing more. A trickling client sends its
  candidates with `PATCH {Location}` (`application/trickle-ice-sdpfrag`).
- `DELETE {Location}` ends the session. It also ends when the connection fails
  or closes, when the camera's stream stops, or when the user loses access
  (checked every minute).
- `406` means the offer can't receive the camera's codec: an H.265 camera
  and a browser without H.265. Fall back to HLS or RTSP.
- Video is the camera's own stream, not re-encoded. AAC audio becomes Opus,
  G.711 passes through.

The media path is the same UDP as above: the viewer's browser must reach
zm-api. Behind 1:1 NAT or in a container, advertise the public address and fix
the port range:

```toml
[streaming.webrtc]
nat_ips = ["203.0.113.10"]
udp_port_min = 50000
udp_port_max = 50099
```

## RTSP

For players that take one URL and want a camera stream: Home Assistant's
stream component, VLC, NVRs. zm-api serves each monitor's live video over RTSP,
opened by a signed link instead of a username and password. It is off until you
turn it on:

```toml
[streaming.rtsp]
enabled = true
port = 8554
# RTSPS, with server.tls's certificate and key:
tls_port = 8322
# Host name in the links (default: the host the link request was sent to):
public_host = "zm.example"
```

ZoneMinder's own `zm_rtsp_server` also defaults to port 8554. Run one or the
other, or move this one.

Mint a link the same way as for HLS:

```
curl -X POST -H "Authorization: Bearer $TOKEN" \
  -H 'content-type: application/json' -d '{"ttl_seconds": 2592000}' \
  https://zm.example/api/v3/live/1/rtsp/link
# {"url":"rtsp://zm.example:8554/live/1?st=…","tls_url":"rtsps://zm.example:8322/live/1?st=…",…}

ffplay -rtsp_transport tcp "rtsp://zm.example:8554/live/1?st=…"
```

- **What a link opens.** That monitor's live video, view only, until it expires.
  A playing stream is checked again every minute, so logging the user out,
  disabling them, or taking away Stream access or the monitor ends it.
- **Transport.** RTP travels over the RTSP connection (interleaved TCP). A
  client that asks for UDP gets `461 Unsupported Transport`, and FFmpeg and VLC
  then retry over TCP.
- **Codecs.** H.264 or H.265, whatever the camera sends. No audio yet.
- **Errors.** A missing or bad link gets `401`, a link for another monitor
  `403`, and a monitor with no stream yet `503`. After three failed links the
  connection is closed.
- **Plain RTSP is unencrypted.** Anyone who can watch the network sees the link
  and the video. Use `rtsps://` where the player supports it (FFmpeg does, and
  doesn't check the certificate by default).

## Session control and snapshots

```
POST   /api/v3/live/{monitor_id}/start
DELETE /api/v3/live/{monitor_id}/stop
GET    /api/v3/live/{monitor_id}/stats
GET    /api/v3/live/sessions
GET    /api/v3/live/sources
GET    /api/v3/monitors/{monitor_id}/snapshot
```

The snapshot route accepts a signed link's `?st=` or `?token=<JWT>`, because
`<img>` cannot set headers.

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

`permission denied opening stream socket …` in the log means zm-api is not
running as ZoneMinder's account and is not in its `ZM_STREAM_SOCKET_GROUP`
either. The sockets are mode 0660. See
[Troubleshooting](../reference/troubleshooting.md).
