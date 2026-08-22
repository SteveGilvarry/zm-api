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

`<img>` and `<video>` elements cannot set headers, so the snapshot route also
accepts the token as a query parameter:

```
GET /api/v3/monitors/1/snapshot?token=<JWT>
```

This is the only place that is accepted. It puts a credential in a URL, where it
can land in proxy logs and browser history — prefer the header wherever the
client controls the request.

## Rate limiting

The authentication endpoints have their own limiter, on by default at roughly
one request per two seconds with a burst of 10. A login retry loop will start
getting 429s. The `prod` profile additionally enables a global per-IP limiter
that is off in `base.toml`, so a client that fans out many parallel requests on
page load can behave differently in production than in development.

Behind a reverse proxy, set `APP_SERVER__MIDDLEWARE__TRUST_PROXY_HEADERS=true`
or every client shares the proxy's single bucket. Leave it `false` anywhere
zm-api is reachable directly — the headers are attacker-controlled there.
