#!/usr/bin/env bash
# Upgrade: reinstalling must keep the keys, the account and the mode, and
# must move an install from the old fixed `zoneminder` account onto
# ZoneMinder's.
. /e2e/lib.sh

dropin=/etc/systemd/system/zm-api.service.d/10-account.conf

section "Reinstall over a running install"
pkg_reinstall >"$ARTIFACTS/zm-api-reinstall.log" 2>&1 \
  || { tail -40 "$ARTIFACTS/zm-api-reinstall.log"; exit 1; }
check "JWT keys unchanged" sha256sum -c --quiet "$STATE/keys.sha256"
check_eq "service user unchanged" "$E2E_WEB_USER" "$(svc_prop zm-api.service User)"
check "zm-api is running" wait_until 60 svc_active zm-api.service
check "API answers" wait_until 60 api_up
check_eq "still passive" "APP_DAEMON__ENABLED=false" \
  "$(grep -E '^APP_DAEMON__ENABLED=' /etc/zm-api/zm-api.env)"

section "Upgrade from the fixed zoneminder account"
# What earlier packages left behind: a zoneminder account owning the state,
# and no drop-in.
systemctl stop zm-api.service
getent group zoneminder >/dev/null || groupadd --system zoneminder
id zoneminder >/dev/null 2>&1 || useradd --system --gid zoneminder --no-create-home zoneminder
rm -f "$dropin"
chown -R zoneminder:zoneminder /var/lib/zm-api /var/log/zm-api
systemctl daemon-reload
check_eq "starting point: unit runs as" "zoneminder" "$(svc_prop zm-api.service User)"

/usr/share/zm-api/setup-instance.sh >"$ARTIFACTS/setup-migrate.log" 2>&1 || true
systemctl daemon-reload
check_eq "service user after setup" "$E2E_WEB_USER" "$(svc_prop zm-api.service User)"
check "the account change is announced" \
  grep -q "service account changed from zoneminder to ${E2E_WEB_USER}" "$ARTIFACTS/setup-migrate.log"
check "JWT keys kept" sha256sum -c --quiet "$STATE/keys.sha256"
check_eq "keys re-owned" "$E2E_WEB_USER" \
  "$(stat -c %U /var/lib/zm-api/keys/private_access_rsa_key.pem)"
systemctl start zm-api.service
check "API answers as the new account" wait_until 60 api_up
check "login still works with the same keys" login

finish
