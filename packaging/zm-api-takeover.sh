#!/usr/bin/env bash
#
# zm-api-takeover — switch zm-api between "passive" and "active" daemon control.
#
# Passive (default after install): zm-api serves only the REST API and leaves
# ZoneMinder's own daemons alone, so installing cannot disturb a running
# ZoneMinder. It is the on-ramp, not the end state.
#
# Active/takeover: zm-api supervises the ZoneMinder daemons (zmc, zmfilter, ...),
# replacing zmdc.pl and zmwatch.pl with one native supervisor. This requires
# zoneminder.service to be stopped/disabled so the two supervisors do not fight
# over the same processes and shared memory.
#
# Taking over checks first that zm-api can do the job (same account as
# ZoneMinder, write access to the events and socket directories), and checks
# afterwards that it did: if zm-api does not stay up, or capture was running
# before and is not running after, the switch is undone and ZoneMinder is
# started again.
#
# Usage:
#   sudo zm-api-takeover            # take over: disable zoneminder, activate zm-api
#   sudo zm-api-takeover --check    # run the pre-flight checks only, change nothing
#   sudo zm-api-takeover --revert   # hand back: passivate zm-api, re-enable zoneminder
#   sudo zm-api-takeover --yes ...  # skip the confirmation prompt
#   sudo zm-api-takeover --force    # take over even if a pre-flight check fails
#
set -euo pipefail

ENV_FILE="${ZM_API_ENV_FILE:-/etc/zm-api/zm-api.env}"
ZM_CONF_DIR="${ZM_CONF_DIR:-/etc/zm}"
ZM_API_SERVICE="zm-api.service"
ZONEMINDER_SERVICE="zoneminder.service"
# How long zm-api gets to come up, and capture to resume, before a takeover
# is judged to have failed.
START_TIMEOUT="${ZM_API_TAKEOVER_TIMEOUT:-60}"

REVERT=0
ASSUME_YES=0
CHECK_ONLY=0
FORCE=0
for arg in "$@"; do
  case "$arg" in
    --revert|--passive) REVERT=1 ;;
    --check) CHECK_ONLY=1 ;;
    --force) FORCE=1 ;;
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

# Last assignment of KEY across zm.conf and conf.d/*.conf.
zm_conf() {
  local key="$1" file
  local -a files=()
  [[ -f "${ZM_CONF_DIR}/zm.conf" ]] && files+=("${ZM_CONF_DIR}/zm.conf")
  for file in "${ZM_CONF_DIR}"/conf.d/*.conf; do
    [[ -f "$file" ]] && files+=("$file")
  done
  [[ ${#files[@]} -gt 0 ]] || return 0
  sed -n "s/^[[:space:]]*${key}[[:space:]]*=[[:space:]]*//p" "${files[@]}" \
    | tail -n1 | tr -d '"'"'" | sed 's/[[:space:]]*$//'
}

service_user() {
  local user
  user="$(systemctl show -p User --value "$ZM_API_SERVICE" 2>/dev/null || true)"
  echo "${user:-root}"
}

# Run a test(1) expression as the service account.
as_service_user() {
  runuser -u "$1" -- test "${@:2}" 2>/dev/null
}

# ---------------------------------------------------------------------------
# Pre-flight: everything that would leave the host recording nothing if it
# were discovered after ZoneMinder's own supervisor had been stopped.
# ---------------------------------------------------------------------------
preflight() {
  local user failures=0
  user="$(service_user)"

  fail() { echo "  FAIL  $*" >&2; failures=$((failures + 1)); }
  pass() { echo "  ok    $*"; }

  echo "Pre-flight checks (zm-api runs as ${user}):"

  if systemctl is-active --quiet "$ZM_API_SERVICE"; then
    pass "zm-api is running in passive mode"
  else
    fail "zm-api is not running. Get it working in passive mode first:" \
         "journalctl -u ${ZM_API_SERVICE} -n 50"
  fi

  if [[ ! -f "${ZM_CONF_DIR}/zm.conf" ]]; then
    echo "  skip  no ${ZM_CONF_DIR}/zm.conf: not checking ZoneMinder's paths"
  else
    local web_user events socks bin
    web_user="$(zm_conf ZM_WEB_USER)"
    if [[ -z "$web_user" || "$web_user" == "$user" ]]; then
      pass "same account as ZoneMinder's daemons (${web_user:-unset})"
    else
      fail "zm-api runs as ${user} but ZoneMinder's daemons run as ${web_user}." \
           "Recordings would change owner. Run /usr/share/zm-api/setup-instance.sh" \
           "and restart zm-api"
    fi

    if as_service_user "$user" -r "${ZM_CONF_DIR}/zm.conf"; then
      pass "can read ${ZM_CONF_DIR}/zm.conf"
    else
      fail "${user} cannot read ${ZM_CONF_DIR}/zm.conf"
    fi

    events="$(zm_conf ZM_DIR_EVENTS)"
    socks="$(zm_conf ZM_PATH_SOCKS)"
    bin="$(zm_conf ZM_PATH_BIN)"
    local dir
    for dir in "$events" "${socks:-/run/zm}" "$(zm_conf ZM_PATH_LOGS)" "$(zm_conf ZM_PATH_MAP)"; do
      [[ -n "$dir" ]] || continue
      if as_service_user "$user" -d "$dir" -a -w "$dir"; then
        pass "can write ${dir}"
      else
        fail "${user} cannot write ${dir}"
      fi
    done

    if as_service_user "$user" -x "${bin:-/usr/bin}/zmc"; then
      pass "can run ${bin:-/usr/bin}/zmc"
    else
      fail "${user} cannot run ${bin:-/usr/bin}/zmc"
    fi
  fi

  return "$failures"
}

# Put ZoneMinder back in charge. Used by --revert and by a failed takeover.
hand_back() {
  echo "Disabling daemon control in ${ENV_FILE}..."
  set_enabled false

  echo "Restarting ${ZM_API_SERVICE} (releases supervised daemons)..."
  systemctl restart "$ZM_API_SERVICE" || true

  if service_exists "$ZONEMINDER_SERVICE"; then
    echo "Re-enabling and starting ${ZONEMINDER_SERVICE}..."
    systemctl enable --now "$ZONEMINDER_SERVICE" || true
  else
    echo "Note: ${ZONEMINDER_SERVICE} not present; start your ZoneMinder daemons manually."
  fi
}

# True once zm-api has been up, with the same main process, for a few seconds.
# A unit that is crash-looping reports "active" between restarts.
zm_api_settled() {
  local pid
  systemctl is-active --quiet "$ZM_API_SERVICE" || return 1
  pid="$(systemctl show -p MainPID --value "$ZM_API_SERVICE")"
  [[ "$pid" != "0" ]] || return 1
  sleep 5
  systemctl is-active --quiet "$ZM_API_SERVICE" \
    && [[ "$(systemctl show -p MainPID --value "$ZM_API_SERVICE")" == "$pid" ]]
}

wait_for() {
  local deadline=$((SECONDS + START_TIMEOUT))
  until "$@"; do
    [[ $SECONDS -lt $deadline ]] || return 1
    sleep 2
  done
}

zmc_under_zm_api() {
  local pid
  pid="$(systemctl show -p MainPID --value "$ZM_API_SERVICE")"
  [[ "$pid" != "0" ]] && pgrep -x -P "$pid" zmc >/dev/null 2>&1
}

if [[ $CHECK_ONLY -eq 1 ]]; then
  preflight && { echo "Ready to take over."; exit 0; }
  echo "Not ready to take over." >&2
  exit 1
fi

if [[ $REVERT -eq 0 ]]; then
  echo "=== zm-api takeover: zm-api will supervise the ZoneMinder daemons ==="
  if ! preflight; then
    if [[ $FORCE -eq 1 ]]; then
      echo "Pre-flight checks failed; continuing because of --force." >&2
    else
      echo "Not taking over: fix the failures above, or pass --force." >&2
      exit 1
    fi
  fi

  echo "This will stop and disable ${ZONEMINDER_SERVICE} and activate daemon"
  echo "control in zm-api (${ENV_FILE})."
  confirm "Proceed?" || { echo "Aborted."; exit 0; }

  # Whether capture is running now decides whether it must be running after.
  capturing_before=0
  pgrep -x zmc >/dev/null 2>&1 && capturing_before=1

  if service_exists "$ZONEMINDER_SERVICE"; then
    echo "Stopping and disabling ${ZONEMINDER_SERVICE}..."
    systemctl disable --now "$ZONEMINDER_SERVICE" || true
  else
    echo "Note: ${ZONEMINDER_SERVICE} not present; nothing to disable."
  fi

  echo "Enabling daemon control in ${ENV_FILE}..."
  set_enabled true

  echo "Restarting ${ZM_API_SERVICE}..."
  systemctl restart "$ZM_API_SERVICE" || true

  echo "Waiting for zm-api to take over (up to ${START_TIMEOUT}s)..."
  problem=""
  if ! wait_for zm_api_settled; then
    problem="zm-api did not stay up in takeover mode"
  elif [[ $capturing_before -eq 1 ]] && ! wait_for zmc_under_zm_api; then
    problem="zmc was capturing before the takeover and zm-api has not started it"
  fi

  if [[ -n "$problem" ]]; then
    echo >&2
    echo "Takeover failed: ${problem}." >&2
    journalctl -u "$ZM_API_SERVICE" -n 25 --no-pager >&2 || true
    echo >&2
    echo "Handing control back to ZoneMinder..." >&2
    hand_back >&2
    echo "ZoneMinder is back in charge; zm-api is passive again." >&2
    exit 3
  fi

  echo
  echo "Done. zm-api is now in active/takeover mode."
  echo "Verify: ps -ef | grep -E 'zmc|zmfilter' ; journalctl -u ${ZM_API_SERVICE} -f"
else
  echo "=== zm-api revert: hand daemon control back to ZoneMinder ==="
  confirm "Passivate zm-api and re-enable ${ZONEMINDER_SERVICE}?" || { echo "Aborted."; exit 0; }

  hand_back

  echo
  echo "Done. zm-api is back in passive (REST-only) mode."
fi
