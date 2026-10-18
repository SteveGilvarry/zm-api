# Deployment and Packaging Guide

This document covers **packaging internals and the release process**.

For installing, configuring, and deploying zm-api, see the documentation site:
<https://stevegilvarry.github.io/zm-api/> — in particular
[Architecture](https://stevegilvarry.github.io/zm-api/guide/architecture.html)
for how ZoneMinder, zm-api, and a dashboard fit together.

## Environments
- dev: local developer runs with `APP_PROFILE=dev`, uses local `settings/`.
- staging: production-like config, separate database, limited access.
- prod: managed service with systemd, config in `/etc/zm-api`, assets in `/usr/share/zm-api`.

## Configuration strategy
- Base config is `settings/base.toml` plus `settings/{APP_PROFILE}.toml`.
- Environment variables override config values using the `APP_` prefix and `__` separators.
  - Example: `APP_DB__HOST=10.0.0.5`
- Use `APP_CONFIG_DIR` to point the app at `/etc/zm-api` in packaged installs.
- Use `APP_STATIC_DIR` to point the app at `/usr/share/zm-api/static` in packaged installs.
- Secrets should live outside the repo. Store keys under `/var/lib/zm-api/keys` and update
  the `secret.*_key` paths in the profile config.
- Generate per-install JWT keys with `scripts/generate-jwt-keys.sh /var/lib/zm-api/keys`
  (or `JWT_KEY_DIR=/var/lib/zm-api/keys`).

## Upgrading an existing ZoneMinder database

**This is the step most likely to be skipped, and it fails quietly.** zm-api runs
pending migrations at startup but only *warns* if they fail, so a database left in
the wrong state yields a service that starts, answers requests, and has features
silently missing.

There are two distinct cases and they need different commands:

| Database | Command |
| --- | --- |
| Existing ZoneMinder (1.26.0+) | `zm-api-db bridge -u mysql://zmuser:zmpass@localhost/zm` |
| Fresh / empty | `zm-api-db up -u mysql://...` |

`bridge` walks the embedded `zm_update` chain, converges triggers, stamps the
baseline migration as applied, then runs the rest. `up` assumes it is *creating*
the schema — never run it against a database that already has ZoneMinder tables.

Back up first (`bridge` rewrites schema across many ZoneMinder versions in one
pass and has no undo), and check the result with `zm-api-db status`. Full details
in `man 8 zm-api-db`.

> The tool is built by cargo as `migrator` (the name `scripts/*-parity.sh` use)
> and installed as `/usr/bin/zm-api-db`.

## Passive vs. active (daemon control)
zm-api ships **passive** by default (`daemon.enabled = false`): it serves only the
REST API and does not create the daemon manager, bind `/run/zm/zmdc.sock`, or run
`kill_orphan_daemons()`. This makes it safe to install **alongside a running stock
ZoneMinder** — the package never fights the existing `zmdc.pl`/`zmc`/`zmfilter`
processes.

Passive is the install-time default, not the intended end state. Takeover
replaces both `zmdc.pl` and `zmwatch.pl` with one native supervisor, and is
where a zm-api host is meant to end up; passive exists so the operator picks the
moment. When you are ready:

```bash
sudo zm-api-takeover          # stops+disables zoneminder.service, flips the flag, restarts zm-api
sudo zm-api-takeover --revert # hand control back to ZoneMinder
```

Equivalent manual steps: `systemctl disable --now zoneminder`, set
`APP_DAEMON__ENABLED=true` in `/etc/zm-api/zm-api.env`, `systemctl restart zm-api`.

## Service account
zm-api runs as the account ZoneMinder's daemons run as. ZoneMinder has no
account of its own: `zmdc.pl`, `zmc` and the rest run as the web server's user,
which upstream's packaging picks per distribution and records in `zm.conf`.

| Distribution | `ZM_WEB_USER`:`ZM_WEB_GROUP` | `zm.conf` | Sockets (`ZM_PATH_SOCKS`) |
| --- | --- | --- | --- |
| Debian, Ubuntu | `www-data:www-data` | `www-data:root` 0640 | `/run/zm` |
| Fedora, RHEL family | `apache:apache` (`nginx:nginx` variant) | `root:apache` 0640 | `/var/lib/zoneminder/sock` |
| openSUSE | `wwwrun:www` | not checked | not checked |
| Arch (AUR) | `http:http` | not checked | not checked |

The Debian row is what upstream's master package produces on Ubuntu 24.04,
observed in the install test. The Red Hat row is read from upstream's
`distros/redhat/zoneminder.spec`. The openSUSE and Arch accounts are upstream's
CMake defaults and the Arch `http` convention.

Earlier packages ran zm-api as a fixed `zoneminder` account and joined it to
the web group. That fails against upstream's own packages: on Debian `zm.conf`
is readable by `www-data` alone, so zm-api could not read the database password
and crash-looped on install; and after takeover `zmc` could not write the
events tree (0755, owned by the web user). A second account also makes revert
lossy, because recordings written under zm-api would belong to an account
ZoneMinder cannot delete as.

`packaging/setup-instance.sh` therefore resolves the account on every install
and upgrade and writes it to a systemd drop-in,
`/etc/systemd/system/zm-api.service.d/10-account.conf`:

1. `ZM_WEB_USER`/`ZM_WEB_GROUP` from `/etc/zm/zm.conf` and `conf.d`, when that
   user exists.
2. Otherwise the account a previous run settled on.
3. Otherwise a dedicated `zoneminder` account, created here. This is the host
   with no ZoneMinder (zm-next only, or zm-api installed first).

Rules the script keeps:

- It never modifies an account it did not create. Extra groups (`video`,
  `ssl-cert`, `ZM_STREAM_SOCKET_GROUP`) go in the drop-in as
  `SupplementaryGroups=`, and only groups that exist are named, because systemd
  refuses to start a unit that lists a missing one.
- The state and log directories are `StateDirectory=`/`LogsDirectory=`, so
  systemd re-owns them when the account changes.
- An admin's own drop-in (`systemctl edit zm-api`) sorts after `10-account.conf`
  and wins.
- When the account changes on upgrade, the script says so and prints the
  `chown --from=` command for recordings made under the old account. It does
  not chown the events tree itself.

Socket and binary directories follow `zm.conf` too: zm-api reads
`ZM_PATH_SOCKS` and `ZM_PATH_BIN` when `daemon.socket_path`,
`streaming.zoneminder.socks_path` and `daemon.bin_path` are at their defaults.

## Recommended deployment flow
1. Install the package (see matrix below). It resolves the service account
   (above), provisions `/var/lib/zm-api/keys` with generated JWT keys, installs
   the unit, and starts zm-api in **passive** mode.
2. Confirm DB connectivity — zm-api falls back to `/etc/zm/zm.conf` when the
   `[db]` placeholders are unchanged; otherwise set `APP_DB__*` in `zm-api.env`.
3. Configure TLS/ACME if needed (see `docs/tls.md`).
4. If a dashboard will be served from a different origin than the API, set
   `APP_SERVER__ALLOWED_ORIGINS` to that origin. Skipping this is the most
   common day-one failure: the dashboard loads and every request is
   CORS-blocked, visible only in the browser console. Not needed when both sit
   behind one hostname — see the docs site.
5. If ZoneMinder is installed, or moved to a different web server, after
   zm-api: `sudo /usr/share/zm-api/setup-instance.sh && sudo systemctl restart zm-api`.
6. Validate health with `/swagger-ui` and a smoke test against `/api-docs/openapi.json`.
7. When ready, run `sudo zm-api-takeover` to assume daemon supervision. It
   checks the host first (`--check` runs only the checks) and rolls back if
   zm-api does not stay up or capture does not resume
   (`man 8 zm-api-takeover`).

JWT keys are generated automatically on install by `setup-instance.sh`. For a
manual/source install, run `scripts/generate-jwt-keys.sh /var/lib/zm-api/keys`
and point `secret.*_key` (or `APP_SECRET__*`) at that directory.

## Packaging layout
- Binary: `/usr/bin/zm-api`; takeover helper: `/usr/bin/zm-api-takeover`;
  migration tool: `/usr/bin/zm-api-db`
- Man pages: `zm-api(8)`, `zm-api-takeover(8)`, `zm-api-db(8)`, `zm-api.env(5)`
- Config: `/etc/zm-api/base.toml`, `/etc/zm-api/prod.toml`, `/etc/zm-api/zm-api.env`
- State (JWT keys): `/var/lib/zm-api/keys` (generated per-install; never packaged)
- Setup helper: `/usr/share/zm-api/setup-instance.sh`
- Systemd unit: `/lib/systemd/system/zm-api.service` (Deb) or `/usr/lib/systemd/system/zm-api.service` (RPM/Arch)

## Distro matrix
| Family | Definition | Build / publish |
| --- | --- | --- |
| Debian/Ubuntu (`.deb`) | `Cargo.toml [package.metadata.deb]` + `packaging/debian/{postinst,prerm,postrm}` | `cargo deb` / `./scripts/package.sh deb` |
| Fedora (`.rpm`) — **CI-tested on 41** | `packaging/rpm/zm-api.spec` | `rpmbuild` / COPR / `./scripts/package.sh rpm` |
| RHEL/Rocky/Alma (`.rpm`) — spec supports, **not CI-tested** | same spec | `rpmbuild` on the target |
| openSUSE (`.rpm`) | same spec (has `%if 0%{?suse_version}` branches) | OBS / `rpmbuild` |
| Arch (`PKGBUILD`) | `packaging/arch/PKGBUILD` + `zm-api.install` | `makepkg` / AUR |

## Building packages
- Debian/Ubuntu: `cargo install cargo-deb` then `./scripts/package.sh deb`.
- RPM (Fedora/EL/openSUSE): build on the target family (or COPR/OBS) with
  `./scripts/package.sh rpm`, which tarballs `HEAD` and runs `rpmbuild` on the spec.
- Arch: `./scripts/package.sh arch` (runs `makepkg`) on an Arch host.
- Update `Cargo.toml` / spec / PKGBUILD version before a release.

## Versioning and releases
This is the first Rust version; it keeps the `v3` major from ZoneMinder's API
lineage and uses SemVer pre-releases while it stabilises:

`3.0.0-alpha.N` → `3.0.0-beta.N` → `3.0.0-rc.N` → `3.0.0`

- **Source of truth:** `Cargo.toml` `version` (currently `3.0.0-alpha.1`).
- **Cutting a release:** push a matching tag, e.g. `git tag v3.0.0-alpha.1 && git
  push origin v3.0.0-alpha.1`. The `Release` workflow builds every package and,
  for tags, publishes a GitHub Release — automatically flagged **pre-release**
  when the tag contains a hyphen.
- **Dry run without publishing:** trigger the workflow via *Actions → Release →
  Run workflow* (`workflow_dispatch`); it builds and uploads artifacts but does
  not create a Release.
- **Per-distro pre-release ordering** (so the eventual stable upgrades cleanly):
  - Debian: cargo-deb maps `-alpha.1` → `~alpha.1` automatically.
  - RPM (`packaging/rpm/zm-api.spec`): `Release: 0.1.alpha1%{?dist}` (set back to
    `1%{?dist}` for stable).
  - Arch (`packaging/arch/PKGBUILD`): `pkgver=3.0.0~alpha1`, git tag in `_pkgtag`.
- **Bumping the version** touches four spots, kept in sync by hand: `Cargo.toml`,
  the spec `Version`/`Release` + changelog, and the PKGBUILD `pkgver`/`_pkgtag`.

## Install testing (CI)
`packaging/e2e/` installs the built package on a container running systemd and
walks the whole process. `.github/workflows/packaging.yml` runs it on pull
requests that touch packaging, on `master`, and nightly. The `Release` workflow
runs it on the exact artifacts it is about to publish, and a package that fails
is held back while the rest go out.

| Distro | ZoneMinder under test | Flow |
| --- | --- | --- |
| Ubuntu 24.04 (amd64, arm64), Debian 13 | upstream master, from zmrepo | database → zoneminder → install → passive → takeover → revert → upgrade → remove |
| Rocky 9 | upstream master, from zmrepo (amd64 only) | same; not yet proven, reported without failing the workflow |
| Fedora 41, openSUSE Tumbleweed, Arch | none published | database → layout → install → passive → upgrade → remove → standalone |

With ZoneMinder present, a synthetic RTSP camera feeds a monitor that records
continuously, and each stage checks that capture and recording carried on:
passive leaves ZoneMinder's processes and sockets untouched, takeover moves
`zmc` under zm-api with the same owner on the recordings, revert moves it back,
and removing the package in takeover mode hands control back first. The
nightly run is the early warning for upstream changes to the schema or the
stream socket.

```bash
packaging/e2e/build.sh ubuntu-24.04    # package built inside the target distro
packaging/e2e/run.sh ubuntu-24.04      # logs in target/e2e/ubuntu-24.04/logs
```

See `packaging/e2e/README.md`.

## Testing and release checklist
- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-features`
- Integration tests: `APP_PROFILE=test-db cargo test --test '*' -- --include-ignored`
- Build packages in CI or a clean builder container
- Packaging change: `packaging/e2e/run.sh <distro>` for at least one distro of
  the family you touched

## Notes for end users
- Only edit `prod.toml` or override via environment variables.
- Keep `base.toml` in the package as a stable defaults layer.
