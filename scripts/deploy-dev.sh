#!/usr/bin/env bash
# Build a zm_api .deb for the dev ZoneMinder box and install it over SSH.
#
# macOS cannot link FFmpeg/OpenSSL for Linux, so the package is built inside a
# Linux container whose distro + arch match the target box (Ubuntu 24.04 arm64).
# The resulting .deb carries versioned FFmpeg/OpenSSL deps, and apt resolves the
# matching runtime libs on the box. The shipped systemd unit runs passive (REST
# only) alongside stock ZoneMinder, and the postinst generates JWT keys.
#
# Usage:
#   ZM_SSH_USER=youruser ./scripts/deploy-dev.sh          # full .deb deploy
#   ZM_SSH_USER=youruser ./scripts/deploy-dev.sh --fast   # binary-only rsync
#
# Env (with defaults):
#   ZM_HOST=192.168.0.45        target box IP/hostname
#   ZM_SSH_USER=<required>      SSH user with passwordless sudo on the box
#   ARCH=arm64                  target arch (arm64|amd64)
#   DISTRO_IMAGE=ubuntu:24.04   build base; MUST match the box's distro+FFmpeg
#   SERVICE=zm_api              systemd unit name
set -euo pipefail

ZM_HOST="${ZM_HOST:-192.168.0.45}"
ZM_SSH_USER="${ZM_SSH_USER:?set ZM_SSH_USER to an SSH user with sudo on $ZM_HOST}"
ARCH="${ARCH:-arm64}"
DISTRO_IMAGE="${DISTRO_IMAGE:-ubuntu:24.04}"
SERVICE="${SERVICE:-zm_api}"
FAST="no"
[ "${1:-}" = "--fast" ] && FAST="yes"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
SSH_TARGET="${ZM_SSH_USER}@${ZM_HOST}"
PLATFORM="linux/${ARCH}"
BUILDER_IMAGE="zm-api-deb-builder:${DISTRO_IMAGE//[:\/]/-}-${ARCH}"

GREEN='\033[0;32m'; YELLOW='\033[1;33m'; RED='\033[0;31m'; NC='\033[0m'
info() { echo -e "${GREEN}[deploy]${NC} $1"; }
warn() { echo -e "${YELLOW}[deploy]${NC} $1"; }
die()  { echo -e "${RED}[deploy]${NC} $1" >&2; exit 1; }

command -v docker >/dev/null 2>&1 && docker info >/dev/null 2>&1 \
    || die "Docker is required to build the Linux package; start Docker Desktop."

# ---------------------------------------------------------------------------
# 1. Build (or reuse) a builder image with the FFmpeg/OpenSSL dev libs + Rust.
#    Cached as an image so only the first run pays the toolchain install cost.
# ---------------------------------------------------------------------------
if ! docker image inspect "$BUILDER_IMAGE" >/dev/null 2>&1; then
    info "Building builder image $BUILDER_IMAGE (first run only, a few minutes)..."
    docker build --platform "$PLATFORM" -t "$BUILDER_IMAGE" - <<DOCKERFILE
FROM ${DISTRO_IMAGE}
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y --no-install-recommends \
        curl ca-certificates build-essential pkg-config libssl-dev \
        libavutil-dev libavformat-dev libavfilter-dev libavdevice-dev \
        libavcodec-dev libswscale-dev libswresample-dev \
    && rm -rf /var/lib/apt/lists/*
RUN curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
ENV PATH="/root/.cargo/bin:\${PATH}"
RUN cargo install cargo-deb --locked
DOCKERFILE
else
    info "Reusing builder image $BUILDER_IMAGE"
fi

# Persist cargo registry + git caches between runs; target/ lives on the host
# (bind-mounted) so builds are incremental. Use a Linux-only target dir so it
# never clashes with the host macOS target/.
CARGO_CACHE_VOL="zm-api-cargo-cache"
docker volume inspect "$CARGO_CACHE_VOL" >/dev/null 2>&1 || docker volume create "$CARGO_CACHE_VOL" >/dev/null
LINUX_TARGET="target/deploy-${ARCH}"

run_builder() {
    docker run --rm --platform "$PLATFORM" \
        -v "$PROJECT_ROOT":/src -w /src \
        -v "${CARGO_CACHE_VOL}":/root/.cargo/registry \
        -e CARGO_TERM_COLOR=always \
        -e CARGO_TARGET_DIR="/src/${LINUX_TARGET}" \
        "$BUILDER_IMAGE" bash -euo pipefail -c "$1"
}

if [ "$FAST" = "yes" ]; then
    # ----- Fast path: build only the binary and rsync it over the installed one.
    # Requires a prior full deploy so the unit/user/keys already exist.
    info "Fast build: compiling release binary in container..."
    run_builder "cargo build --release --bin zm_api"
    BIN="${PROJECT_ROOT}/${LINUX_TARGET}/release/zm_api"
    [ -f "$BIN" ] || die "binary not found at $BIN"
    info "Shipping binary to ${SSH_TARGET} and restarting ${SERVICE}..."
    # Stage to /tmp (non-root scp), then move into place with sudo.
    scp "$BIN" "${SSH_TARGET}:/tmp/zm_api.new"
    ssh "$SSH_TARGET" "sudo install -m 0755 /tmp/zm_api.new /usr/bin/zm_api \
        && rm -f /tmp/zm_api.new \
        && sudo systemctl restart ${SERVICE} \
        && systemctl --no-pager --full status ${SERVICE} | head -n 15"
else
    # ----- Full path: build the .deb and install it (resolves runtime deps).
    info "Building .deb in container ($DISTRO_IMAGE / $ARCH)..."
    run_builder "cargo deb --output /src/${LINUX_TARGET}/zm_api.deb"
    DEB="${PROJECT_ROOT}/${LINUX_TARGET}/zm_api.deb"
    [ -f "$DEB" ] || die "no .deb produced at $DEB"
    info "Built $(basename "$DEB") ($(du -h "$DEB" | cut -f1))"
    info "Installing on ${SSH_TARGET}..."
    scp "$DEB" "${SSH_TARGET}:/tmp/zm_api.deb"
    ssh "$SSH_TARGET" "sudo apt-get install -y /tmp/zm_api.deb \
        && rm -f /tmp/zm_api.deb \
        && sudo systemctl enable --now ${SERVICE} \
        && sudo systemctl restart ${SERVICE} \
        && systemctl --no-pager --full status ${SERVICE} | head -n 20"
fi

# ---------------------------------------------------------------------------
# Health check: the unit binds 0.0.0.0:8080 (settings/prod.toml).
# ---------------------------------------------------------------------------
info "Checking health endpoint on the box..."
if ssh "$SSH_TARGET" "curl -fsS http://127.0.0.1:8080/api/v3/server/health_check >/dev/null 2>&1"; then
    info "✅ ${SERVICE} is up and answering on :8080"
    info "   Reach it from here at: http://${ZM_HOST}:8080"
else
    warn "Service restarted but the health check did not pass yet."
    warn "Inspect with: ssh ${SSH_TARGET} 'journalctl -u ${SERVICE} -n 50 --no-pager'"
fi
