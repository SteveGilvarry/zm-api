# Troubleshooting

## The dashboard loads but every request fails

CORS. The browser reports it in its own console and the API logs nothing
unusual, which is what makes it hard to spot.

Set the dashboard's origin:

```
APP_SERVER__ALLOWED_ORIGINS=https://zm.example.com
```

zm-api logs its effective origin list at startup and warns when it fell back to
the localhost-only default — check there first:

```bash
journalctl -u zm-api | grep -i cors
```

A bare `*` is not accepted. The API sends credentials and the CORS spec forbids
that pairing. Not needed at all when the dashboard and API share a hostname —
see [Serving a dashboard](../guide/dashboard.md).

## `permission denied opening stream socket`

The service user is not in ZoneMinder's `ZM_STREAM_SOCKET_GROUP`. The
per-monitor sockets are mode 0660.

```bash
sudo systemctl edit zm-api
```

```ini
[Service]
SupplementaryGroups=<value of ZM_STREAM_SOCKET_GROUP>
```

Do **not** add it to the shipped unit file. A group that does not exist makes
systemd refuse to start the service entirely (`status=216/GROUP`).

## The service starts but features are missing

Migrations probably failed. zm-api only *warns* on migration failure at startup,
so the service looks healthy.

```bash
journalctl -u zm-api | grep -i migrat
zm-api-db status -u mysql://zmuser:zmpass@localhost/zm
```

See [Upgrading](../getting-started/upgrading.md).

## The unit won't start at all

```bash
systemctl status zm-api
journalctl -u zm-api -n 100 --no-pager
```

Common causes:

| Symptom | Cause |
| --- | --- |
| `status=216/GROUP` | A `SupplementaryGroups=` entry names a group that doesn't exist |
| Cannot connect to database | `APP_DB__*` set to something wrong, overriding the working `zm.conf` fallback |
| Missing JWT keys | `/var/lib/zm-api/keys` not provisioned — run `/usr/share/zm-api/setup-instance.sh` |
| Port in use | Something else on 8080; set `APP_SERVER__PORT` |

## Streams stall behind a reverse proxy

`proxy_buffering off;` on the live and playback routes, and the
`Upgrade`/`Connection` headers with `proxy_http_version 1.1` for the WebRTC
signalling WebSocket. Full config in [Serving a dashboard](../guide/dashboard.md).

## WebRTC connects then shows nothing

Usually NAT. STUN cannot traverse symmetric NAT; configure a TURN server. Note
the media ports are OS-assigned ephemeral UDP with no configurable range. See
[Live streaming](../guide/streaming.md).

## Getting 429s

Rate limiting. The auth endpoints allow roughly one request per two seconds
with a burst of 10; `prod` also enables a global per-IP limiter.

Behind a proxy, set `APP_SERVER__MIDDLEWARE__TRUST_PROXY_HEADERS=true` — without
it every client shares the proxy's single bucket.

## Turning up the logs

```bash
sudo systemctl edit zm-api      # [Service] Environment=RUST_LOG=debug
# or per-module:
#   RUST_LOG=info,zm-api::streaming=debug
#   RUST_LOG=info,zm-api::daemon=debug
sudo systemctl restart zm-api
journalctl -u zm-api -f
```
