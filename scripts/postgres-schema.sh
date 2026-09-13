#!/bin/bash
# Postgres schema check: `migrator up` on an empty Postgres must produce every
# table and column that `migrator up` produces on an empty MySQL. Names only —
# the type mapping between the two is the baseline generator's documented
# decision, and MySQL-only triggers are expected to be absent.
#
# Requirements: the MariaDB and Postgres services from docker-compose.test.yml
# (or the CI services), plus mysql/mariadb and psql clients on PATH.
#
#   MYSQL_HOST=127.0.0.1 MYSQL_PORT=3307 MYSQL_ROOT_PASSWORD=test_root_pass \
#   POSTGRES_HOST=127.0.0.1 POSTGRES_PORT=5433 POSTGRES_PASSWORD=test_root_pass \
#     ./scripts/postgres-schema.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

MYSQL_HOST="${MYSQL_HOST:-127.0.0.1}"
MYSQL_PORT="${MYSQL_PORT:-3307}"
MYSQL_ROOT_USER="${MYSQL_ROOT_USER:-root}"
MYSQL_ROOT_PASSWORD="${MYSQL_ROOT_PASSWORD:-test_root_pass}"
POSTGRES_HOST="${POSTGRES_HOST:-127.0.0.1}"
POSTGRES_PORT="${POSTGRES_PORT:-5433}"
POSTGRES_USER="${POSTGRES_USER:-postgres}"
POSTGRES_PASSWORD="${POSTGRES_PASSWORD:-test_root_pass}"
MYSQL_DB="pgcheck_mysql"
PG_DB="pgcheck_pg"

# shellcheck source=scripts/schema_diff_lib.sh
source "$SCRIPT_DIR/schema_diff_lib.sh"

pg() { # pg <database> [psql args...] - reads statements from stdin
    local db="$1"; shift
    PGPASSWORD="$POSTGRES_PASSWORD" psql -h "$POSTGRES_HOST" -p "$POSTGRES_PORT" \
        -U "$POSTGRES_USER" -d "$db" -v ON_ERROR_STOP=1 -q "$@"
}

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

echo "== Preparing $MYSQL_DB on MySQL and $PG_DB on Postgres"
sql <<EOF
DROP DATABASE IF EXISTS $MYSQL_DB;
CREATE DATABASE $MYSQL_DB;
EOF
pg postgres <<EOF
DROP DATABASE IF EXISTS $PG_DB;
CREATE DATABASE $PG_DB;
EOF

echo "== migrator up on MySQL"
(cd "$PROJECT_ROOT" && cargo run --quiet --bin migrator -- up \
    -u "mysql://$MYSQL_ROOT_USER:$MYSQL_ROOT_PASSWORD@$MYSQL_HOST:$MYSQL_PORT/$MYSQL_DB")

echo "== migrator up on Postgres"
(cd "$PROJECT_ROOT" && cargo run --quiet --bin migrator -- up \
    -u "postgres://$POSTGRES_USER:$POSTGRES_PASSWORD@$POSTGRES_HOST:$POSTGRES_PORT/$PG_DB")

echo "== Comparing table.column sets"
sql -N -B <<EOF | sort > "$WORK/mysql.txt"
SELECT CONCAT(TABLE_NAME, '.', COLUMN_NAME) FROM information_schema.COLUMNS
 WHERE TABLE_SCHEMA = '$MYSQL_DB' AND TABLE_NAME <> 'seaql_migrations'
EOF
pg "$PG_DB" -t -A <<EOF | sed '/^$/d' | sort > "$WORK/pg.txt"
SELECT table_name || '.' || column_name FROM information_schema.columns
 WHERE table_schema = 'public' AND table_name <> 'seaql_migrations'
EOF

if ! diff -u "$WORK/mysql.txt" "$WORK/pg.txt" > "$WORK/diff.txt"; then
    echo "== POSTGRES SCHEMA MISMATCH (- MySQL only, + Postgres only)"
    grep '^[-+][^-+]' "$WORK/diff.txt" || true
    exit 1
fi
echo "== POSTGRES SCHEMA OK ($(wc -l < "$WORK/pg.txt" | tr -d ' ') columns match)"
