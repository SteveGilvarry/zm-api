# Serving a dashboard

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

See also [Permissions](permissions.md) for what a client can discover about
its own account, and [Architecture](architecture.md) for the ports involved.
