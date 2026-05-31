#!/usr/bin/env bash
#
# setup-instance.sh — idempotent first-run provisioning for a zm_api install.
#
# Installed to /usr/share/zm_api/setup-instance.sh and invoked by the
# distribution post-install scripts (deb postinst, rpm %post, Arch .install).
# Safe to run repeatedly. Does NOT enable or start the service and does NOT
# touch ZoneMinder's daemons — zm_api ships in passive mode (see zm_api.env).
#
set -euo pipefail

ZM_USER="zoneminder"
ZM_GROUP="zoneminder"
ZM_HOME="/var/lib/zoneminder"
STATE_DIR="/var/lib/zm_api"
KEY_DIR="${STATE_DIR}/keys"
LOG_DIR="/var/log/zm_api"

# 1. Reuse ZoneMinder's service account; create it only if absent so we never
#    clobber an existing ZoneMinder install's user/group.
if ! getent group "$ZM_GROUP" >/dev/null 2>&1; then
  groupadd --system "$ZM_GROUP"
fi
if ! id "$ZM_USER" >/dev/null 2>&1; then
  useradd --system --gid "$ZM_GROUP" --home-dir "$ZM_HOME" \
    --shell /usr/sbin/nologin --comment "ZoneMinder" "$ZM_USER"
fi
# video group is needed for local camera/device access; ignore if absent.
getent group video >/dev/null 2>&1 && usermod -aG video "$ZM_USER" || true

# 2. Writable state + log dirs owned by the service account.
install -d -o "$ZM_USER" -g "$ZM_GROUP" -m 0750 "$STATE_DIR" "$KEY_DIR" "$LOG_DIR"

# 3. Generate JWT signing keys on first install (idempotent — never overwrite).
if command -v openssl >/dev/null 2>&1; then
  for name in access refresh; do
    priv="${KEY_DIR}/private_${name}_rsa_key.pem"
    pub="${KEY_DIR}/public_${name}_rsa_key.pem"
    if [[ ! -f "$priv" || ! -f "$pub" ]]; then
      openssl genpkey -algorithm RSA -pkeyopt rsa_keygen_bits:2048 -out "$priv"
      openssl rsa -in "$priv" -pubout -out "$pub"
      chown "$ZM_USER:$ZM_GROUP" "$priv" "$pub"
      chmod 600 "$priv"
      chmod 644 "$pub"
    fi
  done
else
  echo "zm_api: openssl not found — generate JWT keys in ${KEY_DIR} before starting" >&2
fi
