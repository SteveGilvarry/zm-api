# Serving a dashboard

There are three ways to do this. The first needs no reverse proxy at all.

Note that `APP_STATIC_DIR`, despite its name, is unrelated: it locates JWT keys
and a couple of image constants, and is never served over HTTP.

## Let zm-api serve it (simplest)

zm-api can serve zm-web's built `dist/` itself:

```toml
[web]
enabled = true
root = "/usr/share/zm-web"
```

or in `/etc/zm-api/zm-api.env`:

```
APP_WEB__ENABLED=true
APP_WEB__ROOT=/usr/share/zm-web
```

One process, one port, one certificate. The UI and the API share an origin by
construction, so **CORS does not apply and `allowed_origins` needs nothing**.
TLS is already handled by `[server.tls]` / `[server.acme]`.

What you get:

- **SPA fallback** — `/events/123` serves `index.html`, so a browser refresh on
  a client-side route works.
- **API paths are never shadowed.** `/api/`, `/swagger-ui`, `/api-docs` and
  `/.well-known/` keep their JSON 404 envelope. A mistyped endpoint still fails
  loudly instead of quietly returning an HTML page with status 200.
- **Cache headers that match how the UI is built** — hashed assets are
  `immutable` for a year, `index.html` is always `no-cache` because it names the
  current asset hashes.
- **A Content-Security-Policy** on UI responses only, configurable via
  `web.content_security_policy` (empty disables it).

Off by default: a deployment fronted by a CDN, or one already running a proxy,
should keep serving the files there. If `web.enabled` is true but the directory
has no `index.html`, zm-api logs a warning and serves the API anyway rather than
refusing to start.

## Same origin behind a reverse proxy

Use this when something else already terminates TLS, or when you want a CDN,
caching, or other sites on the same host.

```nginx
server {
    listen 443 ssl;
    server_name zm.example.com;

    # Dashboard: static build output. The SPA fallback lives here, because
    # zm-api has none — see above.
    root /var/www/zm-web;
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

The browser makes no cross-origin request here either, so **CORS still does not
apply**.

Since the proxy is trusted, also set
`APP_SERVER__MIDDLEWARE__TRUST_PROXY_HEADERS=true` so rate limits key on the
real client IP rather than the proxy's — otherwise every client shares one
bucket. Leave it `false` on any host where zm-api is reachable directly: the
headers are attacker-controlled there, and trusting them lets a client mint a
fresh bucket per request.

## Separate origins

The dashboard on `https://dash.example.com`, the API on `https://api.example.com`.
Then CORS **is** in play and you must set:

```
APP_SERVER__ALLOWED_ORIGINS=https://dash.example.com
```

Unset, zm-api allows localhost only. The failure mode is quiet — the dashboard
loads and every request fails, reported only in the browser console — so this is
the single most likely day-one problem. zm-api logs its effective origin list at
startup and warns when it fell back to the default; check there first.

A bare `*` is not accepted. The API sends credentials, and the CORS spec forbids
that pairing. Entries are exact origins, or `scheme://host:*` to match any port
on a host (intended for development).

See also [Permissions](permissions.md) for what a client can discover about
its own account, and [Architecture](architecture.md) for the ports involved.
