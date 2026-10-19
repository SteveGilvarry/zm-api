#!/usr/bin/env bash
# Revert: ZoneMinder's own supervisor picks capture back up, and the files
# zm-api's zmc wrote are still ZoneMinder's to manage.
. /e2e/lib.sh

section "Revert"
last_event="$(max_event_id)"
check "zm-api-takeover --revert succeeds" zm-api-takeover --revert --yes

check "zoneminder.service is running" wait_until 60 svc_active zoneminder.service
check_eq "zoneminder.service is enabled" "enabled" "$(systemctl is-enabled zoneminder.service)"
check "zmc is running" wait_until 60 zmc_pid
check_eq "zmc is supervised by zmdc.pl" "$(zmdc_pid)" "$(zmc_parent)"
check "capturing" wait_until 90 capturing
check "a new event is recorded to disk" wait_until 90 recorded_after "$last_event"
check "every recording is owned by ${E2E_WEB_USER}" events_owned_by_web_user

check "zm-api stopped cleanly for the switch" stopped_cleanly

section "zm-api is passive again"
check "API answers" wait_until 60 api_up
TOKEN="$(login)" || { bad "login"; finish; }
check_eq "daemon status comes from zmdc.sock again" "200" "$(api_status GET /api/v3/daemons)"
check "snapshot from the stream socket is a JPEG" snapshot_is_jpeg

finish
