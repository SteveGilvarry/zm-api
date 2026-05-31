#!/usr/bin/env bash
#
# zm_api-takeover — switch zm_api between "passive" and "active" daemon control.
#
# Passive (default after install): zm_api serves only the REST API and leaves
# ZoneMinder's own daemons alone, so it coexists with a running ZoneMinder.
#
# Active/takeover: zm_api supervises the ZoneMinder daemons (zmc, zmfilter, ...).
# This requires zoneminder.service to be stopped/disabled so the two supervisors
# do not fight over the same processes and shared memory.
#
# Usage:
#   sudo zm_api-takeover            # take over: disable zoneminder, activate zm_api
#   sudo zm_api-takeover --revert   # hand back: passivate zm_api, re-enable zoneminder
#   sudo zm_api-takeover --yes ...  # skip the confirmation prompt
#
set -euo pipefail

ENV_FILE="${ZM_API_ENV_FILE:-/etc/zm_api/zm_api.env}"
ZM_API_SERVICE="zm_api.service"
ZONEMINDER_SERVICE="zoneminder.service"

REVERT=0
ASSUME_YES=0
for arg in "$@"; do
  case "$arg" in
    --revert|--passive) REVERT=1 ;;
    --yes|-y) ASSUME_YES=1 ;;
    -h|--help)
      # Print the header comment block (lines after the shebang, up to the
      # first non-comment line), stripping the leading "# ".
      awk 'NR>1 && /^#/ {sub(/^# ?/, ""); print; next} NR>1 {exit}' "$0"
      exit 0
      ;;
    *) echo "Unknown argument: $arg" >&2; exit 2 ;;
  esac
done

if [[ $EUID -ne 0 ]]; then
  echo "This script must be run as root (try: sudo $0 $*)" >&2
  exit 1
fi

if [[ ! -f "$ENV_FILE" ]]; then
  echo "Env file not found: $ENV_FILE (set ZM_API_ENV_FILE to override)" >&2
  exit 1
fi

# Set APP_DAEMON__ENABLED to the given value, editing in place. Appends the key
# if it is missing. Matches the key whether currently commented or not.
set_enabled() {
  local value="$1"
  if grep -qE '^[#[:space:]]*APP_DAEMON__ENABLED=' "$ENV_FILE"; then
    sed -i -E "s|^[#[:space:]]*APP_DAEMON__ENABLED=.*|APP_DAEMON__ENABLED=${value}|" "$ENV_FILE"
  else
    printf '\nAPP_DAEMON__ENABLED=%s\n' "$value" >>"$ENV_FILE"
  fi
}

service_exists() {
  systemctl list-unit-files "$1" >/dev/null 2>&1 \
    && systemctl cat "$1" >/dev/null 2>&1
}

confirm() {
  [[ $ASSUME_YES -eq 1 ]] && return 0
  read -r -p "$1 [y/N] " reply
  [[ "$reply" =~ ^[Yy]$ ]]
}

if [[ $REVERT -eq 0 ]]; then
  echo "=== zm_api takeover: zm_api will supervise the ZoneMinder daemons ==="
  echo "This will stop and disable ${ZONEMINDER_SERVICE} and activate daemon"
  echo "control in zm_api (${ENV_FILE})."
  confirm "Proceed?" || { echo "Aborted."; exit 0; }

  if service_exists "$ZONEMINDER_SERVICE"; then
    echo "Stopping and disabling ${ZONEMINDER_SERVICE}..."
    systemctl disable --now "$ZONEMINDER_SERVICE" || true
  else
    echo "Note: ${ZONEMINDER_SERVICE} not present; nothing to disable."
  fi

  echo "Enabling daemon control in ${ENV_FILE}..."
  set_enabled true

  echo "Restarting ${ZM_API_SERVICE}..."
  systemctl restart "$ZM_API_SERVICE"

  echo
  echo "Done. zm_api is now in active/takeover mode."
  echo "Verify: ps -ef | grep -E 'zmc|zmfilter' ; journalctl -u ${ZM_API_SERVICE} -f"
else
  echo "=== zm_api revert: hand daemon control back to ZoneMinder ==="
  confirm "Passivate zm_api and re-enable ${ZONEMINDER_SERVICE}?" || { echo "Aborted."; exit 0; }

  echo "Disabling daemon control in ${ENV_FILE}..."
  set_enabled false

  echo "Restarting ${ZM_API_SERVICE} (releases supervised daemons)..."
  systemctl restart "$ZM_API_SERVICE"

  if service_exists "$ZONEMINDER_SERVICE"; then
    echo "Re-enabling and starting ${ZONEMINDER_SERVICE}..."
    systemctl enable --now "$ZONEMINDER_SERVICE" || true
  else
    echo "Note: ${ZONEMINDER_SERVICE} not present; start your ZoneMinder daemons manually."
  fi

  echo
  echo "Done. zm_api is back in passive (REST-only) mode."
fi
