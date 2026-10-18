# Authentication

zm-api uses bearer JWTs. **No cookies are involved anywhere.**

## Getting tokens

```http
POST /api/v3/auth/login
Content-Type: application/json

{ "username": "admin", "password": "…" }
```

```json
{
  "token_type": "Bearer",
  "access_token": "eyJ…",
  "refresh_token": "eyJ…",
  "expire_in": 600
}
```

Send the access token on every other request:

```
Authorization: Bearer eyJ…
```

## Lifetimes and refresh

Access tokens last **10 minutes**, refresh tokens **1 hour**. They are signed
with *separate* RSA key pairs, deliberately — a leaked access key cannot mint
refresh tokens. Tokens also carry a `typ` claim, so a refresh token cannot be
presented as an access token or vice versa.

```http
POST /api/v3/auth/refresh
Content-Type: application/json

{ "token": "<refresh_token>" }
```

## Knowing when a token expires

Don't decode the JWT client-side. `GET /api/v3/me` returns the user plus
`issued_at`, `expires_at`, and `token_type`:

```json
{
  "user": { "username": "operator", "system": "None", "monitors": "Edit", … },
  "token_type": "access",
  "issued_at": 1755820000,
  "expires_at": 1755820600
}
```

## Logging out

```http
GET /api/v3/auth/logout
Authorization: Bearer <access_token>
```

This is a real server-side revocation, not a client-side token discard: it
raises the user's `TokenMinExpiry` floor, so **every** outstanding token for
that account stops working immediately, including ones issued to other devices.

## Media URLs

`<img>` and `<video>` elements, and players such as FFmpeg, VLC and Home
Assistant, cannot set headers, so media routes accept a credential in the URL
too. There are two kinds.

**Signed links (preferred).** A signed link opens one monitor's live media or
one event's recording, read-only, and nothing else. Mint one with a header
token:

```
POST /api/v3/live/{monitor_id}/hls/link     {"ttl_seconds": 2592000}
POST /api/v3/events/{id}/stream/link        {}
```

The response's `playlist_url` is ready for a player. The playlist then signs
every URI it lists with a signature that lasts minutes, renewed each time a
live playlist is refetched, so a segment URL that leaks into a log is soon
useless. Links last 30 days by default, up to 365. See
[Live streaming](streaming.md#players-that-cannot-send-a-header).

Nothing is stored for a link, so a single link cannot be revoked on its own.
Every link a user minted stops working when that user is logged out
everywhere, changes password or is disabled. Give each integration its own
user, with `Stream: View` (or `Events: View`) on the monitors it needs, so
revoking one integration leaves the others alone.

**The access token (`?token=<JWT>`).** Still accepted on media routes for
existing clients. It is the full access token, able to do anything the user
can, and it puts that in a URL where it can land in proxy logs and browser
history. Prefer the header wherever the client controls the request, and a
signed link where it doesn't. A playlist fetched this way lists signed URIs,
not the token.

Responses to signed requests are sent `Cache-Control: private` and
`Referrer-Policy: no-referrer`.

## Rate limiting

The authentication endpoints have their own limiter, on by default at roughly
one request per two seconds with a burst of 10. A login retry loop will start
getting 429s. The `prod` profile additionally enables a global per-IP limiter
that is off in `base.toml`, so a client that fans out many parallel requests on
page load can behave differently in production than in development.

Behind a reverse proxy, set `APP_SERVER__MIDDLEWARE__TRUST_PROXY_HEADERS=true`
or every client shares the proxy's single bucket. Leave it `false` anywhere
zm-api is reachable directly — the headers are attacker-controlled there.
