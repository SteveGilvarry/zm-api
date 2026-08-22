# First run

## Start it

```bash
sudo systemctl enable --now zm-api
systemctl status zm-api
journalctl -u zm-api -f
```

## Check it responds

```bash
curl -s localhost:8080/api/v3/server/health_check
curl -s localhost:8080/api/v3/host/getVersion
```

Both are public — no token — which makes them useful as reverse-proxy health
checks. Then open `http://localhost:8080/swagger-ui` in a browser.

## Get a token

```bash
curl -s -X POST localhost:8080/api/v3/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"username":"admin","password":"…"}'
```

You get back an access token (10 minutes) and a refresh token (1 hour). Send
`Authorization: Bearer <access_token>` on everything else. See
[Authentication](../guide/authentication.md).

## Two things that commonly need setting

**Database.** zm-api reads `/etc/zm/zm.conf` automatically, so on a normal
single-host install there is nothing to configure. Only set `APP_DB__*` in
`/etc/zm-api/zm-api.env` if the database is somewhere else.

**CORS.** If you are serving a dashboard from a different origin than the API,
set this or the browser blocks every request:

```
APP_SERVER__ALLOWED_ORIGINS=https://zm.example.com
```

The failure is quiet — the dashboard loads and each request fails, reported
only in the browser console. zm-api logs its effective origin list at startup
and warns when it fell back to the localhost-only default, so check there
first. Not needed when both sit behind one hostname; see
[Serving a dashboard](../guide/dashboard.md).

## If live streaming does not work

zmc's per-monitor stream sockets are mode 0660, owned by ZoneMinder's
`ZM_STREAM_SOCKET_GROUP`. If that group is not `zoneminder`, add it:

```bash
sudo systemctl edit zm-api
```

```ini
[Service]
SupplementaryGroups=<value of ZM_STREAM_SOCKET_GROUP>
```

The log says `permission denied opening stream socket …` when this is the
problem. Do not add the group to the shipped unit file directly — naming a
group that does not exist makes systemd refuse to start the service entirely.
