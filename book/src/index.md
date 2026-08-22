<div class="zm-hero">

# zm-api

A modern, fast, type-safe REST API for ZoneMinder — rebuilding a twenty-year-old
Perl, PHP, and CGI surface as one native service.

<div class="zm-badges">
<span class="zm-badge">Rust + Axum</span>
<span class="zm-badge">OpenAPI 3.1</span>
<span class="zm-badge">WebRTC + HLS</span>
<span class="zm-badge">AGPL-3.0</span>
</div>

<div class="zm-cta">
<a class="primary" href="getting-started/install.html">Install</a>
<a href="reference/api.html">API reference</a>
<a href="https://github.com/SteveGilvarry/zm-api">GitHub</a>
</div>

</div>

zm-api talks directly to an existing ZoneMinder MySQL/MariaDB database and ships
in **passive mode** — it serves the REST API and leaves ZoneMinder's own daemons
running exactly as they were. Installing it changes nothing about how your
cameras record, so it is safe to put on a live box and take back off again.

When you are ready, it takes over daemon supervision too, replacing `zmdc.pl`
and `zmwatch.pl` with one native supervisor. Passive is the on-ramp; takeover is
where it is meant to end up, and `zm-api-takeover` moves you either way in one
command.

<div class="zm-cards">

<div class="zm-card">
<h3>One binary, one language</h3>
<p>No PHP-FPM, no CGI, no Perl runtime to babysit. A single native executable and a systemd unit.</p>
</div>

<div class="zm-card">
<h3>Live streaming built in</h3>
<p>WebRTC and HLS from zmc's stream socket, plus recorded-event playback with byte-range seeking.</p>
</div>

<div class="zm-card">
<h3>Real access control</h3>
<p>JWT auth with separate access and refresh keys, per-feature RBAC, and row-level monitor ACLs.</p>
</div>

<div class="zm-card">
<h3>Self-documenting</h3>
<p>Every endpoint is in a generated OpenAPI 3.1 document, served live and published with each release.</p>
</div>

<div class="zm-card">
<h3>A better supervisor</h3>
<p>Takeover replaces zmdc.pl and zmwatch.pl with one native process: exponential backoff, database reconciliation, and daemon control over REST.</p>
</div>

<div class="zm-card">
<h3>Safe to adopt</h3>
<p>Passive by default, so installing changes nothing. Switch to takeover when you choose, and back again with one command.</p>
</div>

<div class="zm-card">
<h3>Actually tested</h3>
<p>1,100+ unit and integration tests with a coverage gate, run against a real ZoneMinder schema in CI.</p>
</div>

</div>

## Where to start

If you have a running ZoneMinder and want zm-api alongside it, go to
[Install](getting-started/install.html) and then
[Upgrading an existing ZoneMinder](getting-started/upgrading.html) — the
database migration step is easy to miss and fails quietly.

If you are building a client against it, start with
[Authentication](guide/authentication.html) and
[Permissions](guide/permissions.html), then browse the
[API reference](reference/api.html).

If you are deciding how to deploy the pieces together,
[Architecture](guide/architecture.html) covers what has to share a host and how
to serve a dashboard.

## Status

zm-api is in active development at `3.0.0-alpha`. It keeps the `v3` major from
ZoneMinder's API lineage, so the URL shape is familiar, but it is not a
drop-in replacement for the CakePHP API — the response envelopes, authentication,
and error format are all different.
