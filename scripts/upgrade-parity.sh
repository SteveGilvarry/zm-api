#!/bin/bash
# Upgrade-path parity: a legacy ZoneMinder database created at a historical
# release and upgraded via `migrator bridge` must structurally match a
# database created fresh by the baseline migration.
#
#   ./scripts/upgrade-parity.sh 1.36.35
#
# The historical db/ files are read from a local ZoneMinder checkout when
# ZM_REPO is set (fast path), otherwise fetched from GitHub by tag.
#
# Requirements: the test MariaDB from docker-compose.test.yml (or the CI
# service container) and a mysql/mariadb client on PATH.

set -euo pipefail

START_VERSION="${1:?usage: upgrade-parity.sh <zm-version-tag, e.g. 1.36.35>}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

MYSQL_HOST="${MYSQL_HOST:-127.0.0.1}"
MYSQL_PORT="${MYSQL_PORT:-3307}"
MYSQL_ROOT_USER="${MYSQL_ROOT_USER:-root}"
MYSQL_ROOT_PASSWORD="${MYSQL_ROOT_PASSWORD:-test_root_pass}"
LEGACY_DB="upgrade_legacy"
BASELINE_DB="upgrade_baseline"

# shellcheck source=scripts/schema_diff_lib.sh
source "$SCRIPT_DIR/schema_diff_lib.sh"

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

fetch_tag_file() { # fetch_tag_file <path-in-repo> <outfile>; ok if absent
    local path="$1" out="$2"
    if [ -n "${ZM_REPO:-}" ] && git -C "$ZM_REPO" rev-parse -q --verify "refs/tags/$START_VERSION" >/dev/null; then
        git -C "$ZM_REPO" show "$START_VERSION:$path" > "$out" 2>/dev/null || return 1
    else
        curl -fsSL "https://raw.githubusercontent.com/ZoneMinder/zoneminder/$START_VERSION/$path" > "$out" || return 1
    fi
}

echo "== Fetching ZoneMinder $START_VERSION schema"
fetch_tag_file "db/zm_create.sql.in" "$WORK/zm_create.sql.in" \
    || { echo "ERROR: cannot fetch db/zm_create.sql.in at tag $START_VERSION"; exit 1; }

# Resolve that era's `source @PKGDATADIR@/db/X.sql` directives from the tag.
grep -oE '^source @PKGDATADIR@/db/[A-Za-z_]+\.sql' "$WORK/zm_create.sql.in" \
    | sed 's|^source @PKGDATADIR@/db/||' | sort -u | while read -r f; do
    fetch_tag_file "db/$f" "$WORK/$f" \
        || { echo "ERROR: cannot fetch db/$f at tag $START_VERSION"; exit 1; }
done

echo "== Preparing databases on $MYSQL_HOST:$MYSQL_PORT"
sql <<EOF
DROP DATABASE IF EXISTS $LEGACY_DB;
DROP DATABASE IF EXISTS $BASELINE_DB;
CREATE DATABASE $LEGACY_DB;
CREATE DATABASE $BASELINE_DB;
EOF

echo "== Loading $START_VERSION schema into $LEGACY_DB"
awk -v dbdir="$WORK" '
    /^source @PKGDATADIR@\/db\// {
        f = $2; sub(/@PKGDATADIR@\/db\//, "", f)
        path = dbdir "/" f
        n = 0
        while ((getline line < path) > 0) { print line; n++ }
        close(path)
        if (n == 0) { print "ERROR: unresolved source: " f > "/dev/stderr"; exit 1 }
        next
    }
    { print }
' "$WORK/zm_create.sql.in" \
    | sed -e "s/@ZM_DB_NAME@/$LEGACY_DB/g" \
          -e "s/@ZM_MYSQL_ENGINE@/InnoDB/g" \
          -e "s|@ZM_DIR_EVENTS@|/var/cache/zoneminder/events|g" \
    | sql "$LEGACY_DB"

# On a real install zmupdate.pl -f seeds these Config rows from
# ConfigData.pm; the bridge reads ZM_DYN_DB_VERSION to find its start point.
sql "$LEGACY_DB" <<EOF
INSERT INTO Config (Id, Name, Value, Type, DefaultValue, Hint, Pattern, Format, Prompt, Help, Category, Readonly, Requires)
VALUES (990, 'ZM_DYN_DB_VERSION', '$START_VERSION', 'string', '', '', '', '', '', '', 'dynamic', 1, ''),
       (991, 'ZM_DYN_CURR_VERSION', '$START_VERSION', 'string', '', '', '', '', '', '', 'dynamic', 1, '');
EOF

echo "== Bridging $LEGACY_DB from $START_VERSION"
(cd "$PROJECT_ROOT" && \
    cargo run --quiet --bin migrator -- bridge \
    -u "mysql://$MYSQL_ROOT_USER:$MYSQL_ROOT_PASSWORD@$MYSQL_HOST:$MYSQL_PORT/$LEGACY_DB")

echo "== Creating fresh baseline in $BASELINE_DB"
(cd "$PROJECT_ROOT" && \
    cargo run --quiet --bin migrator -- up \
    -u "mysql://$MYSQL_ROOT_USER:$MYSQL_ROOT_PASSWORD@$MYSQL_HOST:$MYSQL_PORT/$BASELINE_DB")

echo "== Comparing bridged schema against fresh baseline"
# Reference is the fresh baseline: every table it has must exist in the
# bridged DB with identical structure. Seed rows are not compared - a
# database upgraded from $START_VERSION legitimately differs in data.
compare_structure "$BASELINE_DB" "$LEGACY_DB" "$WORK"
echo "== UPGRADE PARITY OK (from $START_VERSION)"
