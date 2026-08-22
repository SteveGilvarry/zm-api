#!/usr/bin/env bash
#
# install.sh — install zm_api from a source build.
#
# For when no distribution package is available (or you are testing a local
# build). Lays files out exactly where the .deb/.rpm/PKGBUILD put them, so a
# later package install upgrades cleanly instead of colliding.
#
# If a package IS available for your distribution, use it instead — it also
# handles upgrades, removal, and dependencies. See docs/deployment.md.
#
# Usage:
#   cargo build --release --bins
#   sudo ./packaging/install.sh
#
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CONFIG_DIR="/etc/zm_api"
UNIT_DIR="/lib/systemd/system"
MAN_DIR="/usr/share/man"

if [[ $EUID -ne 0 ]]; then
  echo "This script must be run as root (try: sudo $0)" >&2
  exit 1
fi

# Match the packages: unit files under /lib on Debian-likes, /usr/lib elsewhere.
[[ -d /lib/systemd/system ]] || UNIT_DIR="/usr/lib/systemd/system"

release="${REPO_ROOT}/target/release"
if [[ ! -x "${release}/zm_api" ]]; then
  echo "error: ${release}/zm_api not found — run 'cargo build --release --bins' first" >&2
  exit 1
fi

echo "Installing binaries..."
install -D -m 0755 "${release}/zm_api"                  /usr/bin/zm_api
install -D -m 0755 "${release}/migrator"                /usr/bin/zm_api-db
install -D -m 0755 "${REPO_ROOT}/packaging/zm_api-takeover.sh" /usr/bin/zm_api-takeover
install -D -m 0755 "${REPO_ROOT}/packaging/setup-instance.sh"  /usr/share/zm_api/setup-instance.sh

echo "Installing man pages..."
install -D -m 0644 "${REPO_ROOT}/packaging/man/zm_api.8"          "${MAN_DIR}/man8/zm_api.8"
install -D -m 0644 "${REPO_ROOT}/packaging/man/zm_api-takeover.8" "${MAN_DIR}/man8/zm_api-takeover.8"
install -D -m 0644 "${REPO_ROOT}/packaging/man/zm_api-db.8"       "${MAN_DIR}/man8/zm_api-db.8"
install -D -m 0644 "${REPO_ROOT}/packaging/man/zm_api.env.5"      "${MAN_DIR}/man5/zm_api.env.5"

echo "Installing systemd unit..."
install -D -m 0644 "${REPO_ROOT}/packaging/systemd/zm_api.service" "${UNIT_DIR}/zm_api.service"

# Config files are never overwritten: local edits outlive a reinstall. base.toml
# is the exception — it is the packaged defaults layer and is meant to be
# replaced, with local changes belonging in prod.toml or zm_api.env.
echo "Installing configuration..."
install -D -m 0644 "${REPO_ROOT}/settings/base.toml" "${CONFIG_DIR}/base.toml"
for f in prod.toml:settings/prod.toml zm_api.env:packaging/systemd/zm_api.env; do
  dest="${CONFIG_DIR}/${f%%:*}"
  src="${REPO_ROOT}/${f##*:}"
  if [[ -e "$dest" ]]; then
    echo "  keeping existing $dest"
  else
    install -D -m 0644 "$src" "$dest"
  fi
done

# Creates the service account, /var/lib/zm_api/keys, and the JWT keys. Without
# this the service starts and immediately fails to sign tokens.
echo "Provisioning instance (user, state dirs, JWT keys)..."
/usr/share/zm_api/setup-instance.sh

systemctl daemon-reload

cat <<EOF

=== Installation complete ===

zm_api is installed in passive mode: it serves the REST API and leaves
ZoneMinder's daemons alone.

Next steps:
  1. Database — zm_api reads /etc/zm/zm.conf automatically. Override in
     ${CONFIG_DIR}/zm_api.env only if the database is elsewhere.
  2. Existing ZoneMinder database? Migrate it before first start:
       zm_api-db bridge -u mysql://zmuser:zmpass@localhost/zm
     A fresh, empty database instead: zm_api-db up -u mysql://...
  3. Serving a dashboard from another origin? Set APP_SERVER__ALLOWED_ORIGINS
     in ${CONFIG_DIR}/zm_api.env, or the browser will block every request.
  4. Start it:
       systemctl enable --now zm_api
       systemctl status zm_api
       journalctl -u zm_api -f

See man 8 zm_api, man 5 zm_api.env, and docs/deployment.md.
EOF
