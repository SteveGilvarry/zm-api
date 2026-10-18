#!/usr/bin/env bash
# Passive mode: the REST API works against the live database and the live
# stream sockets, and ZoneMinder keeps supervising its own daemons.
. /e2e/lib.sh

section "API"
check "health check" api_up
TOKEN="$(login)" || { bad "login as ${ADMIN_USER}"; finish; }
ok "login as ${ADMIN_USER}"

version="$(curl -fsS -m 10 "$API/api/v3/host/getVersion")"
note "getVersion: ${version}"
if has_zoneminder; then
  check_eq "reports ZoneMinder's schema version" \
    "$(db "SELECT Value FROM Config WHERE Name='ZM_DYN_DB_VERSION'")" \
    "$(echo "$version" | jq -r '.db_version')"
fi
check "startup migrations created zm-api's own tables" own_tables_exist
check_eq "GET /monitors" "200" "$(api_status GET /api/v3/monitors)"
check_eq "GET /events" "200" "$(api_status GET '/api/v3/events?limit=1')"
check_eq "daemon control is off in passive mode" "503" "$(api_status GET /api/v3/daemons)"

if has_zoneminder; then
  section "Against the running ZoneMinder"
  mid="$(monitor_id)"
  check "the monitor is listed" \
    api_json GET /api/v3/monitors ".items[] | select(.name == \"${MONITOR_NAME}\")"
  check "monitor status shows frames arriving" \
    api_json GET "/api/v3/monitor-status/${mid}" '(.capture_fps | tonumber) > 1'
  check "recorded events are listed" \
    api_json GET '/api/v3/events?limit=1' '.items | length > 0'
  check "snapshot from the stream socket is a JPEG" snapshot_is_jpeg
  check "live HLS from the stream socket has segments" hls_has_segments
  if has_chromium; then
    check "live WebRTC plays in a headless browser" webrtc_plays
  else
    note "no chromium on ${E2E_DISTRO}: WebRTC browser check skipped"
  fi

  section "ZoneMinder still in charge"
  check "zoneminder.service running" svc_active zoneminder.service
    check_eq "zmc is supervised by zmdc.pl" "$(zmdc_pid)" "$(zmc_parent)"

  # #82: a zm-api restart once deleted ZoneMinder's socket directory.
  before="$(inode "$(socks_dir)/zmdc.sock")"
  systemctl restart zm-api.service
  check "API back after a restart" wait_until 60 api_up
  check "zm-api stopped cleanly for the restart" stopped_cleanly
  check_eq "zmdc.sock survives a zm-api restart" "$before" "$(inode "$(socks_dir)/zmdc.sock")"
  check "stream socket survives a zm-api restart" test -S "$(socks_dir)/stream_${mid}.sock"
  check "still capturing" wait_until 30 capturing

  check "takeover pre-flight passes" zm-api-takeover --check
fi

finish
