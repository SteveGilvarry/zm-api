#!/bin/bash
# Schema parity check: the baseline SeaORM migration must produce the exact
# same MySQL schema as the legacy zm_create.sql.in (+ sourced db/*.sql files).
#
# Creates two throwaway databases on the test MariaDB server:
#   parity_legacy   - loaded from zm_create.sql.in the way packaging does
#   parity_baseline - created by `migrator up` (baseline migration)
# then diffs normalized structure (columns, indexes, triggers) from
# information_schema. Exits non-zero on any drift.
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

find_client() {
    for c in mariadb mysql /opt/homebrew/opt/mysql-client/bin/mysql; do
        if command -v "$c" &>/dev/null; then echo "$c"; return; fi
    done
    echo "ERROR: no mysql/mariadb client found" >&2
    exit 1
}
CLIENT="$(find_client)"

sql() { # sql <database> [args...]  - reads statements from stdin
    MYSQL_PWD="$MYSQL_ROOT_PASSWORD" "$CLIENT" -h "$MYSQL_HOST" -P "$MYSQL_PORT" \
        -u "$MYSQL_ROOT_USER" --protocol=TCP "$@"
}

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

# Normalized structure dump. Display widths on integer types are stripped
# (deprecated; the legacy file and sea-query differ only cosmetically there).
# Comparison is restricted to the legacy table set: the baseline DB also
# carries zm_api-owned tables (event_synopsis, ...) and seaql_migrations,
# which are additive by design. A table missing from the baseline still
# fails because its rows appear only in the legacy dump.
dump_db() { # dump_db <database> <outfile>
    local db="$1" out="$2"
    sql -N -B <<EOF > "$out.columns"
SELECT c.TABLE_NAME, c.ORDINAL_POSITION, c.COLUMN_NAME,
       REGEXP_REPLACE(c.COLUMN_TYPE,
         '^(tinyint|smallint|mediumint|int|bigint)\\\\([0-9]+\\\\)', '\\\\1'),
       c.IS_NULLABLE, COALESCE(c.COLUMN_DEFAULT, 'NULL'),
       UPPER(c.EXTRA), c.CHARACTER_SET_NAME, c.COLLATION_NAME
FROM information_schema.COLUMNS c
JOIN information_schema.TABLES t
  ON t.TABLE_SCHEMA = c.TABLE_SCHEMA AND t.TABLE_NAME = c.TABLE_NAME
WHERE c.TABLE_SCHEMA = '$db'
  AND c.TABLE_NAME IN (SELECT TABLE_NAME FROM information_schema.TABLES
                       WHERE TABLE_SCHEMA = '$LEGACY_DB')
ORDER BY c.TABLE_NAME, c.ORDINAL_POSITION;
EOF
    sql -N -B <<EOF > "$out.indexes"
SELECT TABLE_NAME, INDEX_NAME, SEQ_IN_INDEX, COLUMN_NAME, NON_UNIQUE
FROM information_schema.STATISTICS
WHERE TABLE_SCHEMA = '$db'
  AND TABLE_NAME IN (SELECT TABLE_NAME FROM information_schema.TABLES
                     WHERE TABLE_SCHEMA = '$LEGACY_DB')
ORDER BY TABLE_NAME, INDEX_NAME, SEQ_IN_INDEX;
EOF
    sql -N -B <<EOF | sed -e 's/[[:space:]]\+/ /g' > "$out.triggers"
SELECT TRIGGER_NAME, EVENT_MANIPULATION, EVENT_OBJECT_TABLE, ACTION_TIMING,
       ACTION_STATEMENT
FROM information_schema.TRIGGERS
WHERE TRIGGER_SCHEMA = '$db'
ORDER BY TRIGGER_NAME;
EOF
    cat "$out.columns" "$out.indexes" "$out.triggers" > "$out"
}

dump_db "$LEGACY_DB" "$WORK/legacy"
dump_db "$BASELINE_DB" "$WORK/baseline"

if diff -u "$WORK/legacy" "$WORK/baseline" > "$WORK/schema.diff"; then
    TABLES=$(sql -N -B <<< "SELECT COUNT(*) FROM information_schema.TABLES WHERE TABLE_SCHEMA='$LEGACY_DB'")
    echo "== PARITY OK ($TABLES tables match)"
else
    echo "== PARITY FAILED - schema drift between legacy and baseline:"
    cat "$WORK/schema.diff"
    exit 1
fi

echo "== Comparing seed row counts"
sql -N -B <<EOF > "$WORK/counts.diff" || true
SELECT l.TABLE_NAME, l.TABLE_ROWS, b.TABLE_ROWS
FROM information_schema.TABLES l
JOIN information_schema.TABLES b ON b.TABLE_NAME = l.TABLE_NAME
WHERE l.TABLE_SCHEMA = '$LEGACY_DB' AND b.TABLE_SCHEMA = '$BASELINE_DB';
EOF
# TABLE_ROWS is approximate for InnoDB; compare exact counts per table.
FAILED=0
while read -r tbl _ _; do
    lc=$(sql -N -B <<< "SELECT COUNT(*) FROM $LEGACY_DB.\`$tbl\`")
    bc=$(sql -N -B <<< "SELECT COUNT(*) FROM $BASELINE_DB.\`$tbl\`")
    if [ "$lc" != "$bc" ]; then
        echo "   seed mismatch in $tbl: legacy=$lc baseline=$bc"
        FAILED=1
    fi
done < "$WORK/counts.diff"
if [ "$FAILED" != "0" ]; then
    echo "== SEED PARITY FAILED"
    exit 1
fi
echo "== SEED PARITY OK"
