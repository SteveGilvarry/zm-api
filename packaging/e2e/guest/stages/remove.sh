#!/usr/bin/env bash
# Removal: the package goes, ZoneMinder stays as it was, and a host that was
# in takeover mode is not left without a supervisor.
. /e2e/lib.sh

dropin=/etc/systemd/system/zm-api.service.d/10-account.conf

if has_zoneminder; then
  section "Remove while in takeover mode"
  check "take over first" zm-api-takeover --yes
  check "capturing under zm-api" wait_until 90 capturing
fi

pkg_remove >"$ARTIFACTS/zm-api-remove.log" 2>&1 \
  || { tail -40 "$ARTIFACTS/zm-api-remove.log"; exit 1; }

section "After removal"
check "binary removed" sh -c '! test -e /usr/bin/zm-api'
check "no zm-api process" sh -c '! pgrep -x zm-api'
check_eq "ZoneMinder's account is unmodified" \
  "$(cat "$STATE/web-user.before")" "$(id "$E2E_WEB_USER")"

if has_zoneminder; then
  check "zoneminder.service was handed control back" wait_until 60 svc_active zoneminder.service
  check_eq "zoneminder.service is enabled" "enabled" "$(systemctl is-enabled zoneminder.service)"
  check "capturing under ZoneMinder" wait_until 90 capturing
fi

if [[ "$E2E_FAMILY" == "deb" ]]; then
  section "Purge"
  check "remove keeps the keys" test -s /var/lib/zm-api/keys/private_access_rsa_key.pem
  DEBIAN_FRONTEND=noninteractive apt-get purge -y zm-api >>"$ARTIFACTS/zm-api-remove.log" 2>&1
  check "state removed" sh -c '! test -e /var/lib/zm-api'
  check "configuration removed" sh -c '! test -e /etc/zm-api/zm-api.env'
  check "account drop-in removed" sh -c "! test -e $dropin"
else
  check "account drop-in removed" sh -c "! test -e $dropin"
fi

finish
