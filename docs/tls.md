# TLS and ACME

The API server can terminate TLS directly. The `secret.*_key` files are used for
JWT signing, not for TLS certificates.

## Enable TLS in the API
1. Set `server.port` to `443` (or another HTTPS port).
2. Enable TLS in config:
   - `server.tls.enabled = true`
   - `server.tls.cert_path = "/etc/letsencrypt/live/example.com/fullchain.pem"`
   - `server.tls.key_path = "/etc/letsencrypt/live/example.com/privkey.pem"`
3. Restart the service.

Environment variable equivalents:
- `APP_SERVER__TLS__ENABLED=true`
- `APP_SERVER__TLS__CERT_PATH=/etc/letsencrypt/live/example.com/fullchain.pem`
- `APP_SERVER__TLS__KEY_PATH=/etc/letsencrypt/live/example.com/privkey.pem`

## Built-in ACMEv2 (rustls-acme)
`rustls-acme` (built on instant-acme) can request and renew certs without
external tooling.

Example config:
- `server.acme.enabled = true`
- `server.acme.domains = ["api.example.com"]`
- `server.acme.contact_emails = ["ops@example.com"]`
- `server.acme.cache_dir = "/var/lib/zm_api/acme"`
- `server.acme.production = true`
- `server.acme.challenge = "tls-alpn-01"` (default)

If you need HTTP-01 (behind a TLS-terminating proxy), set:
- `server.acme.challenge = "http-01"`
- `server.acme.http_port = 80`

Environment variable equivalents:
- `APP_SERVER__ACME__ENABLED=true`
- `APP_SERVER__ACME__DOMAINS__0=api.example.com`
- `APP_SERVER__ACME__CONTACT_EMAILS__0=ops@example.com`
- `APP_SERVER__ACME__CACHE_DIR=/var/lib/zm_api/acme`
- `APP_SERVER__ACME__PRODUCTION=true`
- `APP_SERVER__ACME__CHALLENGE=tls-alpn-01`

## External ACMEv2 automation (Let's Encrypt)

### certbot (standalone)
Use a pre/post hook so certbot can bind port 80/443 during issuance:
```
certbot certonly --standalone -d api.example.com \
  --agree-tos -m ops@example.com \
  --pre-hook "systemctl stop zm_api" \
  --post-hook "systemctl start zm_api"
```

Renewals are handled by the certbot timer:
```
systemctl enable --now certbot.timer
```

### lego (standalone)
```
lego --email ops@example.com --domains api.example.com \
  --path /etc/letsencrypt --accept-tos run
```

On renew, restart the API so it reloads the new certs:
```
systemctl restart zm_api
```

## Notes
- Ensure ports 80/443 are reachable from the internet.
- Static TLS (`server.tls.enabled`) loads certs at startup; restart after renewals.
- Built-in ACME keeps certs fresh without restarts.
- `server.acme.enabled` and `server.tls.enabled` are mutually exclusive.
- HTTP-01 mode starts a separate listener on `server.acme.http_port` for challenges.
- `server.acme.cache_dir` must be writable by the service user.
- The systemd unit uses `DynamicUser` with `SupplementaryGroups=ssl-cert`. Ensure the
  cert/key are readable by the `ssl-cert` group (for example with `setfacl`) or
  switch the unit to a static `User=`/`Group=` if you prefer fixed ownership.
