# How ZoneMinder, zm_api, and a dashboard fit together

This describes the deployed system: which process owns what, what has to share a
host, and how a browser frontend reaches the API. For install and packaging
steps see [deployment.md](deployment.md); for TLS see [tls.md](tls.md).

## The three components

| Component | What it is | Owns |
| --- | --- | --- |
| **ZoneMinder** | The existing C++/Perl/PHP install | Capture daemons (`zmc`, `zma`), the `zm` database schema, the events directory, the legacy web UI |
| **zm_api** | This project — one native binary | The REST API on port 8080, live streaming (WebRTC + HLS), event playback/VOD, retention |
| **zm-dash** | A browser dashboard (separate project) | Nothing server-side — it is static files calling the API |

zm_api does not replace ZoneMinder's capture pipeline. In the default
configuration it does not manage ZoneMinder's processes at all; it reads and
writes the same database, the same events directory, and the same shared memory
that ZoneMinder itself uses, and adds an HTTP surface over them.

## What must share a host

This is the constraint that shapes every deployment: **zm_api must run on the
same machine as `zmc`.**

| Resource | Path | Access | Same host? |
| --- | --- | --- | --- |
| Database | `mysql://…` | read-write, including DDL at startup | **No** — may be remote |
| `zm.conf` | `/etc/zm/zm.conf`, `/etc/zm/conf.d/*.conf` | read-only | Same host, or copy the file |
| Events directory | `/var/lib/zoneminder/events` (per-event path from the `Storage` table) | read for playback, delete for retention | Same host, or a shared mount |
| Stream sockets | `/run/zm/stream_{id}.sock` | connect + read | **Yes, mandatory** |
| Shared memory | `/dev/shm/zm.mmap.{id}` | read-write | **Yes, mandatory** |
| PTZ sockets | `/run/zm/zmcontrol-{id}.sock` | connect + write | Yes (for the Perl bridge) |

Only the database is genuinely detachable. Live streaming reads zmc's unix
sockets and alarm control writes ZoneMinder's shared memory, neither of which
crosses a machine boundary.

Two consequences worth stating plainly. The stream sockets are mode 0660 owned
by ZoneMinder's `ZM_STREAM_SOCKET_GROUP`, so **the zm_api service user must be a
member of that group** or every live stream fails with a permission error. And
zm_api runs schema migrations against the shared ZoneMinder database at
startup — see the upgrade section of [deployment.md](deployment.md) before
first run on an existing install.

## Passive and takeover mode

zm_api ships **passive** (`daemon.enabled = false`) and most deployments should
stay there.

**Passive.** zm_api serves the REST API. `zoneminder.service` keeps supervising
`zmdc.pl`, `zmc`, `zmfilter` and the rest, exactly as before zm_api was
installed. Installing the package changes nothing about how ZoneMinder records.
The daemon-control endpoints (`/api/v3/daemons*`, `/api/v3/system/*`) are still
registered but return 503.

**Takeover** (`daemon.enabled = true`). zm_api supervises the ZoneMinder daemons
itself, replacing `zmdc.pl` and `zmwatch.pl` with its own manager and health
loop.

Exactly one supervisor may run. On startup in takeover mode zm_api runs
`kill_orphan_daemons()`, which `pkill -9`s `zmc`, `zma`, `zmfilter.pl` and
friends before starting its own — so leaving `zoneminder.service` enabled means
two supervisors killing and restarting each other's processes. Use
`zm_api-takeover`, which sequences both services correctly, rather than editing
the flag by hand. See `man 8 zm_api-takeover`.

## Serving the dashboard

**zm_api serves no static files.** There is no `ServeDir`, no SPA fallback — any
unmatched path returns a JSON 404, including `/index.html`. `APP_STATIC_DIR`
despite its name is not an HTTP-served directory; it only locates JWT keys and a
couple of image constants.

So the dashboard is always served by something else, and there are two shapes:

### Same origin behind a reverse proxy (recommended)

```nginx
server {
    listen 443 ssl;
    server_name zm.example.com;

    # Dashboard: static build output. The SPA fallback lives here, because
    # zm_api has none — see above.
    root /var/www/zm-dash;
    location / {
        try_files $uri $uri/ /index.html;
    }

    location /api/ {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host              $host;
        proxy_set_header X-Real-IP         $remote_addr;
        proxy_set_header X-Forwarded-For   $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # WebRTC signalling (/api/v3/live/{id}/webrtc/ws) is a WebSocket.
        proxy_http_version 1.1;
        proxy_set_header Upgrade    $http_upgrade;
        proxy_set_header Connection $connection_upgrade;

        # Media must stream, not accumulate: buffering stalls HLS and
        # fragmented-MP4 playback.
        proxy_buffering off;
        proxy_read_timeout 3600s;
    }

    location /swagger-ui { proxy_pass http://127.0.0.1:8080; }
    location /api-docs/  { proxy_pass http://127.0.0.1:8080; }
}

# Required for the Upgrade header above.
map $http_upgrade $connection_upgrade {
    default upgrade;
    ''      close;
}
```

The browser makes no cross-origin request, so **CORS does not apply and
`allowed_origins` needs nothing**. This is the deployment to recommend: one
hostname, one certificate, one thing to get wrong.

Since the proxy is trusted here, also set
`APP_SERVER__MIDDLEWARE__TRUST_PROXY_HEADERS=true` so rate limits key on the
real client IP rather than the proxy's — otherwise every client shares one
bucket. Leave it `false` on any host where zm_api is reachable directly: the
headers are attacker-controlled there, and trusting them lets a client mint a
fresh bucket per request.

### Separate origins

The dashboard on `https://dash.example.com`, the API on `https://api.example.com`.
Then CORS **is** in play and you must set:

```
APP_SERVER__ALLOWED_ORIGINS=https://dash.example.com
```

Unset, zm_api allows localhost only. The failure mode is quiet — the dashboard
loads and every request fails, reported only in the browser console — so this is
the single most likely day-one problem. zm_api logs its effective origin list at
startup and warns when it fell back to the default; check there first.

A bare `*` is not accepted. The API sends credentials, and the CORS spec forbids
that pairing. Entries are exact origins, or `scheme://host:*` to match any port
on a host (intended for development).

## Ports

| Port | What | When |
| --- | --- | --- |
| 8080 | The entire HTTP API, HLS, and WebRTC signalling | Always (`server.port`) |
| 80 | ACME HTTP-01 challenge only | Only with `acme.challenge = "http-01"` |
| ephemeral UDP | WebRTC media | Whenever WebRTC is used |

One TCP listener, and it is HTTP **or** HTTPS, never both: enabling
`server.tls` or `server.acme` switches that same port to TLS. Enabling both is a
startup error. The default ACME challenge is `tls-alpn-01`, which opens no
second port.

The WebRTC UDP ports are OS-assigned ephemeral, with no configurable range —
worth knowing before putting the media path through a firewall or NAT.

## API surface

Everything lives under `/api/v3`, plus `/swagger-ui` and
`/api-docs/openapi.json`.

Authentication is a bearer JWT and **no cookies are involved anywhere**:

- `POST /api/v3/auth/login` → `{ token_type, access_token, refresh_token, expire_in }`
- `POST /api/v3/auth/refresh` with `{ "token": "<refresh_token>" }`
- `GET /api/v3/auth/logout` — revokes all of that user's outstanding tokens server-side
- `GET /api/v3/me` — returns the user plus `issued_at`/`expires_at`, so a client never has to decode the JWT

Send `Authorization: Bearer <token>`. Access tokens last 10 minutes and refresh
tokens 1 hour, signed with separate RSA key pairs — a leaked access key cannot
mint refresh tokens.

The one exception to the header rule is media: the snapshot route also accepts
`?token=<JWT>`, because `<img>` and `<video>` elements cannot set headers.

`GET /api/v3/server/health_check` and `GET /api/v3/host/getVersion` are public
and unauthenticated — useful as proxy health checks.

Two things a frontend author should know up front.

**Rate limits.** The authentication endpoints have their own limiter, on by
default at roughly one request per two seconds with a burst of 10 — a login
retry loop will start getting 429s. The `prod` profile additionally enables a
global per-IP limiter that is off in `base.toml`, so a dashboard that fans out
many parallel requests on page load can behave differently in production than in
development. Behind a reverse proxy, set
`APP_SERVER__MIDDLEWARE__TRUST_PROXY_HEADERS=true` or every client shares the
proxy's single bucket.

**Authorisation.** Every non-auth route is behind feature-level RBAC, with reads
requiring View and writes requiring Edit on the relevant feature; daemon, system,
and permission-management routes require the admin-tier `System` feature. On top
of that, monitor and group rows are filtered per user, so two accounts can get
different results from the same endpoint.

## Startup order

Nothing enforces ordering between the two services, and nothing needs to: zm_api
retries, and systemd restarts it on failure. The unit is ordered `After=`
mariadb/mysql but deliberately does **not** `Requires=` them, so a host using a
remote database or `mysql.service` still starts.
