#!/bin/bash
# Schema parity check: the baseline SeaORM migration must produce the exact
# same MySQL schema as the legacy zm_create.sql.in (+ sourced db/*.sql files).
#
# Creates two throwaway databases on the test MariaDB server:
#   parity_legacy   - loaded from zm_create.sql.in the way packaging does
#   parity_baseline - created by `migrator up` (baseline migration)
# then diffs normalized structure (columns, indexes, triggers) from
# information_schema, plus exact seed row counts. Exits non-zero on drift.
#
# Requirements: the test DB from docker-compose.test.yml (or CI service)
# reachable, and a mysql/mariadb client on PATH.
#
#   MYSQL_HOST=127.0.0.1 MYSQL_PORT=3307 MYSQL_ROOT_PASSWORD=test_root_pass \
#     ./scripts/schema-parity.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

MYSQL_HOST="${MYSQL_HOST:-127.0.0.1}"
MYSQL_PORT="${MYSQL_PORT:-3307}"
MYSQL_ROOT_USER="${MYSQL_ROOT_USER:-root}"
MYSQL_ROOT_PASSWORD="${MYSQL_ROOT_PASSWORD:-test_root_pass}"
LEGACY_DB="parity_legacy"
BASELINE_DB="parity_baseline"

# shellcheck source=scripts/schema_diff_lib.sh
source "$SCRIPT_DIR/schema_diff_lib.sh"

echo "== Preparing databases on $MYSQL_HOST:$MYSQL_PORT"
sql <<EOF
DROP DATABASE IF EXISTS $LEGACY_DB;
DROP DATABASE IF EXISTS $BASELINE_DB;
CREATE DATABASE $LEGACY_DB;
CREATE DATABASE $BASELINE_DB;
EOF

echo "== Loading legacy schema into $LEGACY_DB"
# Substitute cmake placeholders and inline `source @PKGDATADIR@/db/X.sql`
# from the vendored copies, exactly like packaging / setup-ci-db.sh.
awk -v dbdir="$PROJECT_ROOT/db" '
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
' "$PROJECT_ROOT/zm_create.sql.in" \
    | sed -e "s/@ZM_DB_NAME@/$LEGACY_DB/g" \
          -e "s/@ZM_MYSQL_ENGINE@/InnoDB/g" \
          -e "s|@ZM_DIR_EVENTS@|/var/cache/zoneminder/events|g" \
    | sql "$LEGACY_DB"

echo "== Running baseline migration into $BASELINE_DB"
(cd "$PROJECT_ROOT" && \
    cargo run --quiet --bin migrator -- up \
    -u "mysql://$MYSQL_ROOT_USER:$MYSQL_ROOT_PASSWORD@$MYSQL_HOST:$MYSQL_PORT/$BASELINE_DB")

echo "== Comparing schemas"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

# Reference is the legacy DB's table set: additive zm-api-owned tables and
# seaql_migrations in the baseline DB are by design.
compare_structure "$LEGACY_DB" "$BASELINE_DB" "$WORK"

echo "== Comparing seed row counts"
FAILED=0
while read -r tbl; do
    lc=$(sql -N -B <<< "SELECT COUNT(*) FROM $LEGACY_DB.\`$tbl\`")
    bc=$(sql -N -B <<< "SELECT COUNT(*) FROM $BASELINE_DB.\`$tbl\`")
    if [ "$lc" != "$bc" ]; then
        echo "   seed mismatch in $tbl: legacy=$lc baseline=$bc"
        FAILED=1
    fi
done < <(sql -N -B <<< "SELECT TABLE_NAME FROM information_schema.TABLES WHERE TABLE_SCHEMA='$LEGACY_DB'")
if [ "$FAILED" != "0" ]; then
    echo "== SEED PARITY FAILED"
    exit 1
fi
echo "== SEED PARITY OK"
