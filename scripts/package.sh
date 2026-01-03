#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: ./scripts/package.sh [deb|rpm|all]

Builds Debian and/or RPM packages using cargo-deb and cargo-rpm.
EOF
}

target="${1:-all}"

if [[ "$target" == "-h" || "$target" == "--help" ]]; then
  usage
  exit 0
fi

if [[ "$target" != "deb" && "$target" != "rpm" && "$target" != "all" ]]; then
  echo "Unknown target: $target" >&2
  usage >&2
  exit 1
fi

if [[ "$target" == "deb" || "$target" == "all" ]]; then
  if ! command -v cargo-deb >/dev/null 2>&1; then
    echo "cargo-deb is required: cargo install cargo-deb" >&2
    exit 1
  fi
  cargo deb
fi

if [[ "$target" == "rpm" || "$target" == "all" ]]; then
  if ! command -v cargo-rpm >/dev/null 2>&1; then
    echo "cargo-rpm is required: cargo install cargo-rpm" >&2
    exit 1
  fi
  cargo rpm build
fi
