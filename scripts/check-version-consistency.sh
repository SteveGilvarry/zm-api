#!/usr/bin/env bash
#
# check-version-consistency.sh — verify the four places the version is written
# still agree with Cargo.toml.
#
# Each packaging format spells a pre-release differently, so they cannot simply
# be string-compared:
#
#   Cargo.toml   3.0.0-alpha.1     SemVer
#   rpm spec     3.0.0 + 0.1.alpha1%{?dist}
#                                  Version / Release, so the eventual stable
#                                  (Release: 1) sorts above the pre-release
#   PKGBUILD     3.0.0~alpha1      pacman's `~` sorts before the empty string
#   PKGBUILD     v3.0.0-alpha.1    _pkgtag, the git tag to fetch
#
# Run from the repo root. Exits non-zero on any mismatch.
#
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

fail=0
note() { printf '  %s\n' "$1"; }
bad() { printf 'MISMATCH: %s\n' "$1" >&2; fail=1; }

cargo_version=$(grep -m1 '^version = ' Cargo.toml | sed -E 's/version = "(.*)"/\1/')
[[ -n "$cargo_version" ]] || { echo "could not read version from Cargo.toml" >&2; exit 1; }
echo "Cargo.toml version: ${cargo_version}"

# Split "3.0.0-alpha.1" into base "3.0.0" and pre-release "alpha.1" (may be empty).
base="${cargo_version%%-*}"
pre=""
[[ "$cargo_version" == *-* ]] && pre="${cargo_version#*-}"

# --- rpm spec ---------------------------------------------------------------
spec=packaging/rpm/zm-api.spec
spec_version=$(grep -m1 '^Version:' "$spec" | awk '{print $2}')
spec_release=$(grep -m1 '^Release:' "$spec" | awk '{print $2}')

[[ "$spec_version" == "$base" ]] \
  || bad "${spec} Version is '${spec_version}', expected '${base}'"

if [[ -n "$pre" ]]; then
  # e.g. alpha.1 -> alpha1, so Release should be 0.<n>.alpha1%{?dist}
  expected_tag="${pre//./}"
  [[ "$spec_release" == 0.*".${expected_tag}"* ]] \
    || bad "${spec} Release is '${spec_release}'; a pre-release needs the form '0.N.${expected_tag}%{?dist}' so the stable release sorts above it"
else
  [[ "$spec_release" != 0.* ]] \
    || bad "${spec} Release is '${spec_release}', but ${cargo_version} is a stable version — it should start at '1%{?dist}'"
fi

# --- PKGBUILD ---------------------------------------------------------------
pkgbuild=packaging/arch/PKGBUILD
pkgver=$(grep -m1 '^pkgver=' "$pkgbuild" | cut -d= -f2)
pkgtag=$(grep -m1 '^_pkgtag=' "$pkgbuild" | cut -d= -f2)

if [[ -n "$pre" ]]; then
  expected_pkgver="${base}~${pre//./}"
else
  expected_pkgver="$base"
fi
[[ "$pkgver" == "$expected_pkgver" ]] \
  || bad "${pkgbuild} pkgver is '${pkgver}', expected '${expected_pkgver}'"

[[ "$pkgtag" == "v${cargo_version}" ]] \
  || bad "${pkgbuild} _pkgtag is '${pkgtag}', expected 'v${cargo_version}'"

# --- tag, when running on one -----------------------------------------------
tag="${GITHUB_REF_NAME:-}"
if [[ "${GITHUB_REF_TYPE:-}" == "tag" && -n "$tag" ]]; then
  [[ "$tag" == "v${cargo_version}" ]] \
    || bad "git tag is '${tag}', expected 'v${cargo_version}'"
fi

if [[ $fail -eq 0 ]]; then
  echo "All version strings agree."
  note "rpm      ${spec_version}-${spec_release}"
  note "PKGBUILD ${pkgver} (tag ${pkgtag})"
else
  echo >&2
  echo "Bumping a version touches: Cargo.toml, ${spec} (Version + Release + %changelog)," >&2
  echo "and ${pkgbuild} (pkgver + _pkgtag). See docs/deployment.md." >&2
fi
exit $fail
