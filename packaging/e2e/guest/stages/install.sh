#!/usr/bin/env bash
# Install the package the way a user would and check what it set up, and
# that it left ZoneMinder alone.
. /e2e/lib.sh

section "Install $(basename "$(pkg_file)")"
pkg_install >"$ARTIFACTS/zm-api-install.log" 2>&1 \
  || { tail -40 "$ARTIFACTS/zm-api-install.log"; exit 1; }
grep -E '^zm-api: ' "$ARTIFACTS/zm-api-install.log" | sed 's/^/  /' || true

for f in /usr/bin/zm-api /usr/bin/zm-api-db /usr/bin/zm-api-takeover \
         /usr/share/zm-api/setup-instance.sh /etc/zm-api/zm-api.env \
         /etc/zm-api/base.toml /etc/zm-api/prod.toml; do
  check "installed $f" test -e "$f"
done
check "unit file is installed" systemctl cat zm-api.service

section "Service account"
dropin=/etc/systemd/system/zm-api.service.d/10-account.conf
check "account drop-in written" test -f "$dropin"
check_eq "service user" "$E2E_WEB_USER" "$(svc_prop zm-api.service User)"
check_eq "service group" "$E2E_WEB_GROUP" "$(svc_prop zm-api.service Group)"
check_eq "ZoneMinder's account is unmodified" \
  "$(cat "$STATE/web-user.before")" "$(id "$E2E_WEB_USER")"
check "service account can read zm.conf" \
  runuser -u "$E2E_WEB_USER" -- test -r /etc/zm/zm.conf
for g in $(svc_prop zm-api.service SupplementaryGroups); do
  check "supplementary group $g exists" getent group "$g"
done

section "State"
key=/var/lib/zm-api/keys/private_access_rsa_key.pem
check "JWT keys generated" test -s "$key" -a -s /var/lib/zm-api/keys/public_refresh_rsa_key.pem
check_eq "private key owner" "$E2E_WEB_USER" "$(stat -c %U "$key")"
check_eq "private key mode" "600" "$(stat -c %a "$key")"
sha256sum /var/lib/zm-api/keys/*.pem >"$STATE/keys.sha256"

section "Service"
# Arch packages never enable a unit; the deb and rpm start it in passive mode.
if [[ "$E2E_FAMILY" == "arch" ]]; then
  systemctl enable --now zm-api.service
fi
check_eq "unit is enabled" "enabled" "$(systemctl is-enabled zm-api.service)"
check "zm-api is running" wait_until 60 svc_active zm-api.service
check "API answers" wait_until 60 api_up
check_eq "installed in passive mode" "APP_DAEMON__ENABLED=false" \
  "$(grep -E '^APP_DAEMON__ENABLED=' /etc/zm-api/zm-api.env)"

if has_zoneminder; then
  section "ZoneMinder was not disturbed"
  check "zoneminder.service still running" svc_active zoneminder.service
  check_eq "same zmc process as before the install" \
    "$(cat "$STATE/zmc-pid.before")" "$(zmc_pid)"
  check_eq "zmdc.sock untouched" \
    "$(cat "$STATE/zmdc-sock.before")" "$(inode "$(socks_dir)/zmdc.sock")"
  check "still capturing" capturing
else
  # zm-api created the schema itself on first start: the fresh-database path.
  check "schema created on first start" wait_until 120 own_tables_exist
  db "UPDATE Users SET Password='${ADMIN_HASH}' WHERE Username='${ADMIN_USER}'"
fi

finish
