#!/usr/bin/env bash
# Takeover: zm-api replaces zmdc.pl as the supervisor, and recording carries
# on under the same account.
. /e2e/lib.sh

mid="$(monitor_id)"
env_file=/etc/zm-api/zm-api.env

section "A takeover that leaves zm-api down is rolled back"
# Break zm-api's next start, not ZoneMinder: a wrong database password is read
# only when the takeover restarts the service. Passive mode is still up, so
# the pre-flight passes, and the failure appears exactly where it would hurt,
# after ZoneMinder's own supervisor has been stopped.
echo "APP_DB__PASSWORD=wrong-on-purpose" >>"$env_file"
rc=0
ZM_API_TAKEOVER_TIMEOUT=30 zm-api-takeover --yes \
  >"$ARTIFACTS/takeover-rollback.log" 2>&1 || rc=$?
sed -i '/^APP_DB__PASSWORD=wrong-on-purpose$/d' "$env_file"
check_eq "failed takeover exits 3" "3" "$rc"
check "zoneminder.service was started again" wait_until 60 svc_active zoneminder.service
check_eq "zoneminder.service is enabled again" "enabled" "$(systemctl is-enabled zoneminder.service)"
check_eq "zm-api is passive again" "APP_DAEMON__ENABLED=false" \
  "$(grep -E '^APP_DAEMON__ENABLED=' "$env_file")"
check "capture resumed under ZoneMinder" wait_until 90 capturing
systemctl restart zm-api.service
check "zm-api recovers once the fault is removed" wait_until 60 api_up

section "Takeover"
last_event="$(max_event_id)"
check "zm-api-takeover succeeds" zm-api-takeover --yes
cat "$_out" >"$ARTIFACTS/takeover.log"

check_eq "zoneminder.service is stopped" "inactive" "$(systemctl is-active zoneminder.service || true)"
check_eq "zoneminder.service is disabled" "disabled" "$(systemctl is-enabled zoneminder.service || true)"
check "zm-api is running" svc_active zm-api.service
check "no zmdc.pl left" no_zmdc
check "zm-api stopped cleanly for the switch" stopped_cleanly

section "Capture under zm-api"
check "zmc is running" wait_until 60 zmc_pid
check_eq "zmc is supervised by zm-api" "$(zm_api_pid)" "$(zmc_parent)"
check_eq "zmc runs as" "$E2E_WEB_USER" "$(zmc_user)"
check "capturing" wait_until 90 capturing
check "a new event is recorded to disk" wait_until 90 recorded_after "$last_event"
check "every recording is owned by ${E2E_WEB_USER}" events_owned_by_web_user

section "Clients"
check "API answers" wait_until 30 api_up
TOKEN="$(login)" || { bad "login"; finish; }
check "daemon control lists zmc" api_grep GET /api/v3/daemons zmc
check "snapshot from the stream socket is a JPEG" snapshot_is_jpeg
# ZoneMinder's web console asks zmdc.sock whether the system is running.
check_eq "zmdc.pl check, as the web user" "running" \
  "$(runuser -u "$E2E_WEB_USER" -- zmdc.pl check 2>/dev/null | tail -1)"

finish
