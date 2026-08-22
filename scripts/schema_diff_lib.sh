# Shared helpers for schema comparison scripts (schema-parity.sh,
# upgrade-parity.sh). Bash, sourced - not executable on its own.
#
# Expects: MYSQL_HOST, MYSQL_PORT, MYSQL_ROOT_USER, MYSQL_ROOT_PASSWORD.

find_client() {
    for c in mariadb mysql /opt/homebrew/opt/mysql-client/bin/mysql; do
        if command -v "$c" &>/dev/null; then echo "$c"; return; fi
    done
    echo "ERROR: no mysql/mariadb client found" >&2
    exit 1
}
CLIENT="$(find_client)"

sql() { # sql [client args...]  - reads statements from stdin
    MYSQL_PWD="$MYSQL_ROOT_PASSWORD" "$CLIENT" -h "$MYSQL_HOST" -P "$MYSQL_PORT" \
        -u "$MYSQL_ROOT_USER" --protocol=TCP "$@"
}

# dump_db <database> <outfile> <table_source_db>
#
# Normalized structural dump: columns, indexes, triggers. Integer display
# widths are stripped (deprecated, cosmetic) and column ORDER is ignored:
# legacy ALTER chains append columns in a different physical order than a
# fresh create, which is semantically irrelevant and would require full
# table rebuilds on user databases to converge. Comparison is restricted to
# the table set of <table_source_db> so additive zm-api-owned tables and
# seaql_migrations don't produce noise; a table missing from the candidate
# still fails because its rows appear only in the reference dump.
dump_db() {
    local db="$1" out="$2" src="$3"
    sql -N -B <<EOF > "$out.columns"
SELECT c.TABLE_NAME, c.COLUMN_NAME,
       REGEXP_REPLACE(c.COLUMN_TYPE,
         '^(tinyint|smallint|mediumint|int|bigint)\\\\([0-9]+\\\\)', '\\\\1'),
       c.IS_NULLABLE, COALESCE(c.COLUMN_DEFAULT, 'NULL'),
       UPPER(c.EXTRA), c.CHARACTER_SET_NAME, c.COLLATION_NAME
FROM information_schema.COLUMNS c
WHERE c.TABLE_SCHEMA = '$db'
  AND c.TABLE_NAME IN (SELECT TABLE_NAME FROM information_schema.TABLES
                       WHERE TABLE_SCHEMA = '$src')
ORDER BY c.TABLE_NAME, c.COLUMN_NAME;
EOF
    sql -N -B <<EOF > "$out.indexes"
SELECT TABLE_NAME, INDEX_NAME, SEQ_IN_INDEX, COLUMN_NAME, NON_UNIQUE
FROM information_schema.STATISTICS
WHERE TABLE_SCHEMA = '$db'
  AND TABLE_NAME IN (SELECT TABLE_NAME FROM information_schema.TABLES
                     WHERE TABLE_SCHEMA = '$src')
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

# compare_structure <reference_db> <candidate_db> <workdir>
compare_structure() {
    local ref="$1" cand="$2" work="$3"
    dump_db "$ref" "$work/reference" "$ref"
    dump_db "$cand" "$work/candidate" "$ref"
    if diff -u "$work/reference" "$work/candidate" > "$work/schema.diff"; then
        local tables
        tables=$(sql -N -B <<< "SELECT COUNT(*) FROM information_schema.TABLES WHERE TABLE_SCHEMA='$ref'")
        echo "== STRUCTURE PARITY OK ($tables tables)"
        return 0
    fi
    echo "== STRUCTURE PARITY FAILED:"
    cat "$work/schema.diff"
    return 1
}
