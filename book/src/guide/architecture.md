# How ZoneMinder, zm-api, and a dashboard fit together

Which process owns what, what has to share a host, and how a browser frontend
reaches the API.

## The three components

| Component | What it is | Owns |
| --- | --- | --- |
| **ZoneMinder** | The existing C++/Perl/PHP install | Capture daemons (`zmc`, `zma`), the `zm` database schema, the events directory, the legacy web UI |
| **zm-api** | This project — one native binary | The REST API on port 8080, live streaming (WebRTC + HLS), event playback/VOD, retention |
| **zm-web** | The browser UI (separate project) — replaces ZoneMinder's PHP `web/` | Nothing server-side: static files calling this API |

zm-api does not replace ZoneMinder's capture pipeline. In the default
configuration it does not manage ZoneMinder's processes at all; it reads and
writes the same database, the same events directory, and the same shared memory
that ZoneMinder itself uses, and adds an HTTP surface over them.

## What must share a host

This is the constraint that shapes every deployment: **zm-api must run on the
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
by ZoneMinder's `ZM_STREAM_SOCKET_GROUP`, so **the zm-api service user must be a
member of that group** or every live stream fails with a permission error. And
zm-api runs schema migrations against the shared ZoneMinder database at
startup — see [Upgrading an existing ZoneMinder](../getting-started/upgrading.md)
before first run on an existing install.

## Passive and takeover mode

zm-api ships **passive** (`daemon.enabled = false`) so installing it cannot
disturb a running ZoneMinder. That is the on-ramp, not the destination —
**takeover is where zm-api is meant to end up**, and passive exists so the
switch happens on your schedule rather than at install time.

**Passive.** zm-api serves the REST API. `zoneminder.service` keeps supervising
`zmdc.pl`, `zmc`, `zmfilter` and the rest, exactly as before zm-api was
installed. Installing the package changes nothing about how ZoneMinder records.
The daemon-control endpoints (`/api/v3/daemons*`, `/api/v3/system/*`) are still
registered but return 503.

**Takeover** (`daemon.enabled = true`). zm-api supervises the ZoneMinder daemons
itself, replacing both `zmdc.pl` and `zmwatch.pl` with one native supervisor:
exponential backoff that resets once a daemon stays up, a reconciliation loop
that keeps running daemons in step with the `Monitors` table, daemon control
over the REST API, and supervision events in the journal rather than in
ZoneMinder's own logs. The legacy `zmdc.sock` IPC shim stays bound, so tooling
that talks to `zmdc.pl` keeps working. See
[Passive and takeover mode](takeover.md).

Exactly one supervisor may run. On startup in takeover mode zm-api runs
`kill_orphan_daemons()`, which `pkill -9`s `zmc`, `zma`, `zmfilter.pl` and
friends before starting its own — so leaving `zoneminder.service` enabled means
two supervisors killing and restarting each other's processes. Use
`zm-api-takeover`, which sequences both services correctly, rather than editing
the flag by hand. See `man 8 zm-api-takeover`.

## Serving a dashboard

zm-api serves no static files, so a dashboard is always served by something
else. See [Serving a dashboard](dashboard.md).

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

Nothing enforces ordering between the two services, and nothing needs to: zm-api
retries, and systemd restarts it on failure. The unit is ordered `After=`
mariadb/mysql but deliberately does **not** `Requires=` them, so a host using a
remote database or `mysql.service` still starts.
