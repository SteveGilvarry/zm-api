#!/usr/bin/env bash
#
# run.sh: install the zm-api package on a real systemd host and prove the
# install process end to end.
#
# The host is a container running systemd as PID 1, so the package's
# maintainer scripts, the unit, and `systemctl` behave as they do on a real
# machine. Two flows, chosen by what upstream publishes for the distro
# (distros.sh):
#
#   ZoneMinder master available (Debian, Ubuntu, EL):
#     database → zoneminder → install → passive → takeover → revert
#       → upgrade → remove
#     A ZoneMinder built from upstream master records from a synthetic RTSP
#     camera. zm-api is installed beside it, checked in passive mode, takes
#     over supervision, hands it back, is upgraded and is removed. Capture
#     and recording are checked at every step.
#
#   No ZoneMinder master package (Fedora, openSUSE, Arch):
#     database → layout → install → passive → upgrade → remove → standalone
#     The package is installed against the accounts and zm.conf ZoneMinder's
#     packaging would have created, then with no ZoneMinder at all.
#
# Usage:
#   packaging/e2e/build.sh ubuntu-24.04      # once, and after code changes
#   packaging/e2e/run.sh ubuntu-24.04
#
# Options:
#   --package DIR     directory holding the package (default: build.sh's output)
#   --stages a,b,c    run only these stages, in the order given
#   --no-zoneminder   use the second flow even where ZoneMinder is available
#   --keep            leave the container running afterwards, for a shell:
#                       docker exec -it zm-api-e2e-<distro> bash
#
# Logs land in target/e2e/<distro>/logs/.
#
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
# shellcheck source=packaging/e2e/distros.sh
. "$HERE/distros.sh"

distro="${1:?usage: run.sh <distro> [options] (one of: $(e2e_distros))}"
shift
e2e_distro "$distro"

pkg_dir="$ROOT/target/e2e/${distro}/pkg"
stages=""
keep=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --package) pkg_dir="$(cd "$2" && pwd)"; shift 2 ;;
    --stages) stages="${2//,/ }"; shift 2 ;;
    --no-zoneminder) E2E_ZM=none; shift ;;
    --keep) keep=1; shift ;;
    *) echo "unknown option: $1" >&2; exit 2 ;;
  esac
done

if [[ -z "$stages" ]]; then
  if [[ "$E2E_ZM" == "none" ]]; then
    stages="database layout install passive upgrade remove standalone"
  else
    stages="database zoneminder install passive takeover revert upgrade remove"
  fi
fi

if ! ls "$pkg_dir"/zm-api* >/dev/null 2>&1; then
  echo "no package in $pkg_dir. Run packaging/e2e/build.sh $distro first." >&2
  exit 1
fi

image="zm-api-e2e-systemd:${distro}"
name="zm-api-e2e-${distro}"
logs="$ROOT/target/e2e/${distro}/logs"
rm -rf "$logs" && mkdir -p "$logs"

log() { printf '\033[0;32m[e2e %s]\033[0m %s\n' "$distro" "$*"; }

# ---------------------------------------------------------------------------
# Host image: the distro's base image plus systemd and the few tools the
# stages use. Container images mask or omit things a real install has, and
# the ones that would change the result are put back.
# ---------------------------------------------------------------------------
host_dockerfile() {
  echo "FROM ${E2E_IMAGE}"
  echo "ENV container=docker"
  case "$E2E_FAMILY" in
    deb)
      # policy-rc.d in the base image forbids starting services from a
      # maintainer script, which is the behaviour under test.
      cat <<'EOF'
ENV DEBIAN_FRONTEND=noninteractive
RUN rm -f /usr/sbin/policy-rc.d \
    && apt-get update \
    && apt-get install -y --no-install-recommends \
         systemd systemd-sysv dbus ca-certificates curl gnupg jq procps iproute2 \
    && rm -rf /var/lib/apt/lists/*
EOF
      ;;
    rpm)
      if [[ "$distro" == opensuse-* ]]; then
        echo "RUN zypper --non-interactive install systemd dbus-1 curl jq procps iproute2 util-linux gzip tar"
      else
        # --allowerasing: EL images ship curl-minimal, which conflicts with curl.
        echo "RUN dnf -y install --allowerasing systemd dbus curl jq procps-ng iproute util-linux gzip tar"
      fi
      ;;
    arch)
      echo "RUN pacman -Syu --noconfirm systemd dbus curl jq procps-ng iproute2 util-linux"
      ;;
  esac
  cat <<'EOF'
RUN for u in getty.target console-getty.service systemd-logind.service \
             systemd-udevd.service systemd-resolved.service \
             apt-daily.timer apt-daily-upgrade.timer dnf-makecache.timer; do \
      systemctl mask "$u" 2>/dev/null || true; \
    done
STOPSIGNAL SIGRTMIN+3
CMD ["/usr/lib/systemd/systemd"]
EOF
}

if ! docker image inspect "$image" >/dev/null 2>&1; then
  log "building host image $image"
  host_dockerfile | docker build -q -t "$image" - >/dev/null
fi

docker rm -f "$name" >/dev/null 2>&1 || true

collect_logs() {
  docker exec "$name" bash -c '
    journalctl -u zm-api --no-pager -o short-iso >/artifacts/zm-api.journal 2>&1
    journalctl -u zoneminder --no-pager -o short-iso >/artifacts/zoneminder.journal 2>&1
    systemctl status zm-api zoneminder --no-pager >/artifacts/status.txt 2>&1
    systemctl cat zm-api >/artifacts/unit.txt 2>&1
    ps -eo user,pid,ppid,args >/artifacts/ps.txt 2>&1
    chmod -R a+rX /artifacts
  ' >/dev/null 2>&1 || true
}

cleanup() {
  collect_logs
  if [[ $keep -eq 1 ]]; then
    log "container left running: docker exec -it $name bash"
  else
    docker rm -f "$name" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

# --privileged with a private cgroup namespace is what systemd needs to run as
# PID 1 under cgroup v2. /run and /tmp must be tmpfs, as on a booted host.
# zmc maps its frame buffers in /dev/shm, and Docker's 64 MB default is too
# small for them.
log "booting $E2E_IMAGE with systemd"
docker run -d --name "$name" \
  --privileged --cgroupns=private \
  --tmpfs /run --tmpfs /run/lock --tmpfs /tmp \
  --shm-size=1g \
  -v "$HERE/guest":/e2e:ro \
  -v "$pkg_dir":/pkg:ro \
  -v "$logs":/artifacts \
  -e E2E_DISTRO="$distro" -e E2E_FAMILY="$E2E_FAMILY" -e E2E_ZM="$E2E_ZM" \
  -e E2E_WEB_USER="$E2E_WEB_USER" -e E2E_WEB_GROUP="$E2E_WEB_GROUP" \
  "$image" >/dev/null

for _ in $(seq 1 60); do
  state="$(docker exec "$name" systemctl is-system-running 2>/dev/null || true)"
  [[ "$state" == "running" || "$state" == "degraded" ]] && break
  sleep 1
done
if [[ "$state" != "running" && "$state" != "degraded" ]]; then
  echo "systemd did not come up in the container (state: ${state:-none})" >&2
  exit 1
fi

failed=""
for stage in $stages; do
  printf '\n\033[1;34m######## %s: %s\033[0m\n' "$distro" "$stage"
  if ! docker exec "$name" bash "/e2e/stages/${stage}.sh" 2>&1 | tee "$logs/stage-${stage}.log"; then
    failed="$stage"
    break
  fi
done

echo
if [[ -n "$failed" ]]; then
  collect_logs
  printf '\033[31m[e2e %s] FAILED in stage "%s". Logs: %s\033[0m\n' "$distro" "$failed" "$logs"
  echo "--- last lines of the zm-api journal"
  tail -n 30 "$logs/zm-api.journal" 2>/dev/null | cut -c1-400 || true
  exit 1
fi
if [[ -f "$logs/zoneminder-version" ]]; then
  log "passed against ZoneMinder $(cat "$logs/zoneminder-version")"
else
  log "passed"
fi
