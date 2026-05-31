# Deployment and Packaging Guide

## Environments
- dev: local developer runs with `APP_PROFILE=dev`, uses local `settings/`.
- staging: production-like config, separate database, limited access.
- prod: managed service with systemd, config in `/etc/zm_api`, assets in `/usr/share/zm_api`.

## Configuration strategy
- Base config is `settings/base.toml` plus `settings/{APP_PROFILE}.toml`.
- Environment variables override config values using the `APP_` prefix and `__` separators.
  - Example: `APP_DB__HOST=10.0.0.5`
- Use `APP_CONFIG_DIR` to point the app at `/etc/zm_api` in packaged installs.
- Use `APP_STATIC_DIR` to point the app at `/usr/share/zm_api/static` in packaged installs.
- Secrets should live outside the repo. Store keys under `/var/lib/zm_api/keys` and update
  the `secret.*_key` paths in the profile config.
- Generate per-install JWT keys with `scripts/generate-jwt-keys.sh /var/lib/zm_api/keys`
  (or `JWT_KEY_DIR=/var/lib/zm_api/keys`).

## Passive vs. active (daemon control)
zm_api ships **passive** by default (`daemon.enabled = false`): it serves only the
REST API and does not create the daemon manager, bind `/run/zm/zmdc.sock`, or run
`kill_orphan_daemons()`. This makes it safe to install **alongside a running stock
ZoneMinder** — the package never fights the existing `zmdc.pl`/`zmc`/`zmfilter`
processes.

When you are ready to let zm_api supervise the ZoneMinder daemons itself:

```bash
sudo zm_api-takeover          # stops+disables zoneminder.service, flips the flag, restarts zm_api
sudo zm_api-takeover --revert # hand control back to ZoneMinder
```

Equivalent manual steps: `systemctl disable --now zoneminder`, set
`APP_DAEMON__ENABLED=true` in `/etc/zm_api/zm_api.env`, `systemctl restart zm_api`.

## Recommended deployment flow
1. Install the package (see matrix below). It creates the `zoneminder` user (if
   absent), provisions `/var/lib/zm_api/keys` with generated JWT keys, installs
   the unit, and starts zm_api in **passive** mode.
2. Confirm DB connectivity — zm_api falls back to `/etc/zm/zm.conf` when the
   `[db]` placeholders are unchanged; otherwise set `APP_DB__*` in `zm_api.env`.
3. Configure TLS/ACME if needed (see `docs/tls.md`).
4. Validate health with `/swagger-ui` and a smoke test against `/api-docs/openapi.json`.
5. When ready, run `sudo zm_api-takeover` to assume daemon supervision.

JWT keys are generated automatically on install by `setup-instance.sh`. For a
manual/source install, run `scripts/generate-jwt-keys.sh /var/lib/zm_api/keys`
and point `secret.*_key` (or `APP_SECRET__*`) at that directory.

## Packaging layout
- Binary: `/usr/bin/zm_api`; takeover helper: `/usr/bin/zm_api-takeover`
- Config: `/etc/zm_api/base.toml`, `/etc/zm_api/prod.toml`, `/etc/zm_api/zm_api.env`
- State (JWT keys): `/var/lib/zm_api/keys` (generated per-install; never packaged)
- Setup helper: `/usr/share/zm_api/setup-instance.sh`
- Systemd unit: `/lib/systemd/system/zm_api.service` (Deb) or `/usr/lib/systemd/system/zm_api.service` (RPM/Arch)

## Distro matrix
| Family | Definition | Build / publish |
| --- | --- | --- |
| Debian/Ubuntu (`.deb`) | `Cargo.toml [package.metadata.deb]` + `packaging/debian/{postinst,prerm,postrm}` | `cargo deb` / `./scripts/package.sh deb` |
| Fedora/RHEL/Rocky/Alma (`.rpm`) | `packaging/rpm/zm_api.spec` | `rpmbuild` / COPR / `./scripts/package.sh rpm` |
| openSUSE (`.rpm`) | same spec (has `%if 0%{?suse_version}` branches) | OBS / `rpmbuild` |
| Arch (`PKGBUILD`) | `packaging/arch/PKGBUILD` + `zm_api.install` | `makepkg` / AUR |

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
  - RPM (`packaging/rpm/zm_api.spec`): `Release: 0.1.alpha1%{?dist}` (set back to
    `1%{?dist}` for stable).
  - Arch (`packaging/arch/PKGBUILD`): `pkgver=3.0.0~alpha1`, git tag in `_pkgtag`.
- **Bumping the version** touches four spots, kept in sync by hand: `Cargo.toml`,
  the spec `Version`/`Release` + changelog, and the PKGBUILD `pkgver`/`_pkgtag`.

## Testing and release checklist
- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-features`
- Integration tests: `APP_PROFILE=test-db cargo test --test '*' -- --include-ignored`
- Build packages in CI or a clean builder container

## Notes for end users
- Only edit `prod.toml` or override via environment variables.
- Keep `base.toml` in the package as a stable defaults layer.
