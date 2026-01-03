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

## Recommended deployment flow
1. Create config in `/etc/zm_api` (start from `settings/base.toml` + `settings/prod.toml`).
2. Place key material in `/var/lib/zm_api/keys` and update `secret.*_key` paths.
3. Install the package (Deb or RPM) and enable the systemd unit.
4. Configure TLS/ACME if needed (see `docs/tls.md`).
5. Validate health with `/swagger-ui` and run a smoke test against `/api-docs/openapi.json`.

## Packaging layout (Deb/RPM)
- Binary: `/usr/bin/zm_api`
- Config: `/etc/zm_api/base.toml`, `/etc/zm_api/prod.toml`, `/etc/zm_api/zm_api.env`
- Static assets: `/usr/share/zm_api/static`
- Systemd unit: `/lib/systemd/system/zm_api.service` (Deb) or `/usr/lib/systemd/system/zm_api.service` (RPM)

## Building packages
- Install tooling: `cargo install cargo-deb cargo-rpm`
- Build both: `./scripts/package.sh`
- Deb only: `./scripts/package.sh deb`
- RPM only: `./scripts/package.sh rpm`
- Update `Cargo.toml` package metadata (maintainer/license) before release.

## Testing and release checklist
- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-features`
- Integration tests: `APP_PROFILE=test-db cargo test --test '*' -- --include-ignored`
- Build packages in CI or a clean builder container

## Notes for end users
- Only edit `prod.toml` or override via environment variables.
- Keep `base.toml` in the package as a stable defaults layer.
