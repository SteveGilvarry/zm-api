#!/usr/bin/env python3
"""One-shot generator: parse ZoneMinder's zm_create.sql.in and emit the
baseline SeaORM migration (src/migration/m00000000_000001_zm_baseline/).

The output is committed and reviewed; rerun only when the vendored
zm_create.sql.in is refreshed before the cutover release:

    python3 scripts/gen_baseline_migration.py

Mapping decisions (see issue #11):
- ENUM columns      -> ColumnDef::enumeration() (native ENUM on MySQL, real
                       enum type on Postgres, created in up() when needed)
- SET('X10')        -> custom type on MySQL, string on Postgres (single
                       vestigial column, Monitors.Triggers)
- tinytext/longtext -> custom type on MySQL (exact parity), text on Postgres
- mediumint         -> custom type on MySQL, integer on Postgres
- latin1_bin cols   -> custom full type on MySQL; plain string on Postgres
                       (Postgres compares case-sensitively by default)
- ON UPDATE CURRENT_TIMESTAMP -> .extra() on MySQL only; Postgres callers
                       must set the column explicitly (no trigger is created)
- int display widths ((10), (6), ...) are dropped; deprecated in MySQL and
                       stripped by the parity normalizer
- @ZM_MYSQL_ENGINE@ -> InnoDB (MySQL default; engine clause omitted)
- @PKGDATADIR@ / @ZM_DIR_EVENTS@ in seed rows -> Rust consts in seeds.rs
"""

import re
import subprocess
import sys
from pathlib import Path
from dataclasses import dataclass, field

ROOT = Path(__file__).resolve().parent.parent
SQL_PATH = ROOT / "zm_create.sql.in"
OUT_DIR = ROOT / "src" / "migration" / "m00000000_000001_zm_baseline"

# ---------------------------------------------------------------------------
# Parsing
# ---------------------------------------------------------------------------


@dataclass
class Column:
    name: str
    base: str            # lowercased base type, e.g. "int", "varchar", "enum"
    args: str            # raw parenthesized args, e.g. "10", "'a','b'", "5,2"
    unsigned: bool = False
    nullable: bool = True
    default: object = None   # None | ("str", s) | ("num", s) | ("now",)
    autoinc: bool = False
    on_update_now: bool = False
    charset: str = ""    # raw "CHARACTER SET ... COLLATE ..." clause if present
    inline_unique: bool = False


@dataclass
class Table:
    name: str
    columns: list = field(default_factory=list)
    primary: list = field(default_factory=list)
    uniques: list = field(default_factory=list)   # (index_name, [cols])
    keys: list = field(default_factory=list)      # (index_name, [cols])
    fks: list = field(default_factory=list)       # (col, ref_table, ref_col, on_delete)


@dataclass
class Insert:
    table: str
    columns: list          # [] when the statement has no column list
    rows: list             # list of list of raw value tokens


def read_sql() -> str:
    """Read zm_create.sql.in, inlining `source @PKGDATADIR@/db/X.sql`
    directives from the vendored copies in db/ (triggers.sql is handled
    separately - MySQL trigger bodies are not table DDL)."""
    text = SQL_PATH.read_text()
    def inline(m):
        name = m.group(1)
        if name == "triggers":
            return ""
        return (ROOT / "db" / f"{name}.sql").read_text()
    text = re.sub(r"(?m)^source\s+@PKGDATADIR@/db/(\w+)\.sql\s*$", inline, text)
    text = text.replace("@ZM_MYSQL_ENGINE@", "InnoDB")
    # Strip block comments and line comments (none of ZM's are load-bearing).
    text = re.sub(r"/\*.*?\*/", "", text, flags=re.S)
    text = re.sub(r"^\s*--.*$", "", text, flags=re.M)
    text = re.sub(r"^\s*#.*$", "", text, flags=re.M)
    return text


def parse_triggers() -> list:
    """Split db/triggers.sql into individual CREATE TRIGGER statements.
    The DELIMITER // dance is a mysql-client artifact; each statement is sent
    individually at runtime so inner semicolons are fine."""
    text = (ROOT / "db" / "triggers.sql").read_text()
    text = re.sub(r"/\*.*?\*/", "", text, flags=re.S)
    text = re.sub(r"(?m)^\s*--.*$", "", text)
    text = re.sub(r"(?im)^\s*delimiter\s.*$", "", text)
    out = []
    # DROP lines are terminated inconsistently (some `;`, some `//`), and
    # comments precede some CREATEs, so search inside each chunk.
    for chunk in text.split("//"):
        m = re.search(r"(?is)CREATE\s+TRIGGER\s.*", chunk)
        if m:
            out.append(m.group(0).strip().rstrip(";").strip() + "\n")
    return out


def split_statements(text: str):
    """Split on semicolons that are not inside quoted strings."""
    stmts, buf, i, n = [], [], 0, len(text)
    quote = None
    while i < n:
        c = text[i]
        if quote:
            if c == "\\" and quote == "'":
                buf.append(text[i:i + 2])
                i += 2
                continue
            if c == quote:
                if i + 1 < n and text[i + 1] == quote:  # '' escape
                    buf.append(c * 2)
                    i += 2
                    continue
                quote = None
            buf.append(c)
        elif c in ("'", '"', "`"):
            quote = c
            buf.append(c)
        elif c == ";":
            s = "".join(buf).strip()
            if s:
                stmts.append(s)
            buf = []
        else:
            buf.append(c)
        i += 1
    s = "".join(buf).strip()
    if s:
        stmts.append(s)
    return stmts


COL_RE = re.compile(
    r"^`(?P<name>\w+)`\s+(?P<base>\w+)(?:\((?P<args>[^)]*(?:\)[^)]*)*?)\))?"
    r"(?P<rest>.*)$",
    re.S,
)

TYPE_ARGS_RE = re.compile(r"^`?(?P<name>\w+)`?\s+(?P<base>\w+)(?P<tail>.*)$", re.S)


def split_top_level(s: str, sep: str = ","):
    """Split on sep at paren/quote depth 0."""
    parts, buf, depth, quote = [], [], 0, None
    i, n = 0, len(s)
    while i < n:
        c = s[i]
        if quote:
            if c == "\\" and quote == "'":
                buf.append(s[i:i + 2])
                i += 2
                continue
            if c == quote:
                if i + 1 < n and s[i + 1] == quote:
                    buf.append(c * 2)
                    i += 2
                    continue
                quote = None
            buf.append(c)
        elif c in ("'", '"', "`"):
            quote = c
            buf.append(c)
        elif c == "(":
            depth += 1
            buf.append(c)
        elif c == ")":
            depth -= 1
            buf.append(c)
        elif c == sep and depth == 0:
            parts.append("".join(buf).strip())
            buf = []
        else:
            buf.append(c)
        i += 1
    tail = "".join(buf).strip()
    if tail:
        parts.append(tail)
    return parts


def parse_column(line: str) -> Column:
    m = TYPE_ARGS_RE.match(line)
    if not m:
        raise ValueError(f"unparseable column: {line}")
    name, base, tail = m.group("name"), m.group("base").lower(), m.group("tail")
    args = ""
    tail = tail.strip()
    if tail.startswith("("):
        depth, j = 0, 0
        for j, c in enumerate(tail):
            if c == "(":
                depth += 1
            elif c == ")":
                depth -= 1
                if depth == 0:
                    break
        args = tail[1:j]
        tail = tail[j + 1:]
    rest = tail.strip()
    col = Column(name=name, base=base, args=args)

    cm = re.search(r"(?i)\bCHARACTER\s+SET\s+\w+(\s+COLLATE\s+\w+)?", rest)
    if cm:
        col.charset = re.sub(r"\s+", " ", cm.group(0))
        rest = rest[:cm.start()] + rest[cm.end():]
    if re.search(r"(?i)\bunsigned\b", rest):
        col.unsigned = True
    if re.search(r"(?i)\bNOT\s+NULL\b", rest):
        col.nullable = False
    if re.search(r"(?i)\bauto_increment\b", rest):
        col.autoinc = True
    if re.search(r"(?i)\bUNIQUE\b", rest):
        col.inline_unique = True
    if re.search(r"(?i)\bon\s+update\s+CURRENT_TIMESTAMP\b", rest):
        col.on_update_now = True
        rest = re.sub(r"(?i)\bon\s+update\s+CURRENT_TIMESTAMP\b", "", rest)
    dm = re.search(r"(?i)\bdefault\s+(?P<v>'(?:[^'\\]|\\.|'')*'|CURRENT_TIMESTAMP\(\d+\)|[\w.+-]+)", rest)
    if dm:
        v = dm.group("v")
        if v.upper() == "NULL":
            col.default = None
        elif v.upper() == "CURRENT_TIMESTAMP":
            col.default = ("now",)
        elif v.upper().startswith("CURRENT_TIMESTAMP("):
            col.default = ("now_prec", re.search(r"\((\d+)\)", v).group(1))
        elif v.startswith("'"):
            col.default = ("str", unquote_sql(v))
        else:
            col.default = ("num", v)
    return col


def unquote_sql(v: str) -> str:
    assert v.startswith("'") and v.endswith("'")
    inner = v[1:-1]
    inner = inner.replace("''", "'")
    inner = re.sub(r"\\(['\"\\])", r"\1", inner)
    inner = inner.replace("\\n", "\n").replace("\\t", "\t")
    return inner


def parse_key_cols(argstr: str):
    return [re.sub(r"`|\(\d+\)", "", c).strip() for c in argstr.split(",")]


def parse_tables_and_inserts(stmts):
    tables, order, inserts, indexes = {}, [], [], []
    raw_seed_sql = []
    for st in stmts:
        st_stripped = st.strip()
        up = st_stripped.upper()
        if up.startswith("CREATE TABLE"):
            m = re.match(r"(?is)CREATE\s+TABLE\s+(?:IF\s+NOT\s+EXISTS\s+)?`?(\w+)`?\s*\((?P<body>.*)\)\s*(?P<opts>[^)]*)$", st_stripped)
            if not m:
                raise ValueError(f"unparseable CREATE TABLE: {st_stripped[:80]}")
            t = Table(name=m.group(1))
            body = m.group("body")
            for line in split_top_level(body):
                lu = line.upper()
                if lu.startswith("PRIMARY KEY"):
                    km = re.search(r"\((.*)\)", line, re.S)
                    t.primary = parse_key_cols(km.group(1))
                elif lu.startswith("UNIQUE"):
                    km = re.match(r"(?i)UNIQUE(?:\s+(?:KEY|INDEX))?\s*(?:`?(\w+)`?\s*)?\((.*)\)", line, re.S)
                    cols = parse_key_cols(km.group(2))
                    # MySQL auto-names an unnamed unique key after its first column.
                    t.uniques.append((km.group(1) or cols[0], cols))
                elif lu.startswith("KEY") or lu.startswith("INDEX"):
                    km = re.match(r"(?i)(?:KEY|INDEX)\s+`?(\w+)`?\s*\((.*)\)", line, re.S)
                    t.keys.append((km.group(1), parse_key_cols(km.group(2))))
                elif lu.startswith("FOREIGN KEY") or lu.startswith("CONSTRAINT"):
                    fm = re.match(
                        r"(?is)(?:CONSTRAINT\s+`?(?P<cname>\w+)`?\s+)?FOREIGN\s+KEY\s*\(([^)]*)\)\s*"
                        r"REFERENCES\s+`?(\w+)`?\s*\(([^)]*)\)(.*)",
                        line,
                    )
                    if not fm:
                        raise ValueError(f"unparseable FK in {t.name}: {line[:60]}")
                    fcols = parse_key_cols(fm.group(2))
                    rcols = parse_key_cols(fm.group(4))
                    tail = fm.group(5)
                    on_delete = None
                    dm2 = re.search(r"(?i)ON\s+DELETE\s+(CASCADE|SET\s+NULL|RESTRICT|NO\s+ACTION)", tail)
                    if dm2:
                        on_delete = dm2.group(1).upper().replace(" ", "_")
                    assert len(fcols) == 1 and len(rcols) == 1, f"multi-col FK in {t.name}"
                    t.fks.append((fcols[0], fm.group(3), rcols[0], on_delete, fm.group("cname")))
                elif line.startswith("`") or re.match(r"^\w+\s+\w+", line):
                    t.columns.append(parse_column(line))
                else:
                    raise ValueError(f"unhandled line in {t.name}: {line[:60]}")
            for c in t.columns:
                if c.inline_unique:
                    t.uniques.append((c.name, [c.name]))
            tables[t.name] = t
            order.append(t.name)
        elif up.startswith("CREATE INDEX") or up.startswith("CREATE UNIQUE INDEX"):
            m = re.match(
                r"(?is)CREATE\s+(UNIQUE\s+)?INDEX\s+`?(\w+)`?\s+ON\s+`?(\w+)`?\s*\((.*)\)",
                st_stripped,
            )
            indexes.append((m.group(3), m.group(2), parse_key_cols(m.group(4)), bool(m.group(1))))
        elif up.startswith("INSERT"):
            if re.search(r"(?is)\)\s*SELECT\s", st_stripped) or re.search(r"(?is)VALUES", st_stripped) is None:
                # INSERT ... SELECT: portable once IGNORE is dropped (fresh DB
                # has no duplicates); keep as raw SQL.
                raw_seed_sql.append(re.sub(r"(?i)^INSERT\s+IGNORE\s+", "INSERT ", st_stripped))
                continue
            m = re.match(
                r"(?is)INSERT\s+(?:IGNORE\s+)?INTO\s+`?(\w+)`?\s*(?:\((?P<cols>[^)]*)\)\s*)?VALUES\s*(?P<vals>.*)$",
                st_stripped,
            )
            if not m:
                raise ValueError(f"unparseable INSERT: {st_stripped[:80]}")
            cols = []
            if m.group("cols"):
                cols = [c.strip().strip("`") for c in m.group("cols").split(",")]
            rows = []
            for row in split_top_level(m.group("vals")):
                row = row.strip()
                assert row.startswith("(") and row.endswith(")"), row[:40]
                rows.append(split_top_level(row[1:-1]))
            inserts.append(Insert(table=m.group(1), columns=cols, rows=rows))
        # DROP TABLE / USE / SET / GRANT etc. are ignored.
    for tbl, name, cols, uniq in indexes:
        if uniq:
            tables[tbl].uniques.append((name, cols))
        else:
            tables[tbl].keys.append((name, cols))
    return tables, order, inserts, raw_seed_sql


# ---------------------------------------------------------------------------
# Emission
# ---------------------------------------------------------------------------

def rs_str(s: str) -> str:
    out = s.replace("\\", "\\\\").replace('"', '\\"').replace("\n", "\\n").replace("\t", "\\t")
    return f'"{out}"'


def snake(name: str) -> str:
    s = re.sub(r"(?<=[a-z0-9])([A-Z])", r"_\1", name)
    s = re.sub(r"__+", "_", s)
    return s.lower()


def enum_type_name(table: str, col: str) -> str:
    return f"{snake(table)}_{snake(col)}"


INT_KINDS = {
    "tinyint": ("tiny_integer", "tiny_unsigned"),
    "smallint": ("small_integer", "small_unsigned"),
    "int": ("integer", "unsigned"),
    "integer": ("integer", "unsigned"),
    "bigint": ("big_integer", "big_unsigned"),
}


def column_type_calls(t: Table, c: Column):
    """Return (portable_call, mysql_call_or_None).

    portable_call is a chain suffix like `.integer()`. When mysql_call is not
    None the generated code branches on backend: mysql_call for
    MySql/MariaDB (exact legacy parity), portable_call for everything else.
    """
    b = c.base
    if b in INT_KINDS:
        signed, uns = INT_KINDS[b]
        return (f".{uns}()" if c.unsigned else f".{signed}()", None)
    if b == "mediumint":
        my = "mediumint unsigned" if c.unsigned else "mediumint"
        return (".integer()", f'.custom(Alias::new("{my}"))')
    if b == "varchar":
        if c.charset:
            my = f"varchar({c.args}) {c.charset}"
            return (f".string_len({c.args})", f'.custom(Alias::new("{my}"))')
        return (f".string_len({c.args})", None)
    if b == "char":
        return (f".char_len({c.args})", None)
    if b == "text":
        return (".text()", None)
    if b == "tinytext":
        return (".text()", '.custom(Alias::new("tinytext"))')
    if b == "longtext":
        return (".text()", '.custom(Alias::new("longtext"))')
    if b == "datetime":
        return (".date_time()", None)
    if b == "timestamp":
        if c.args:
            return (f'.custom(Alias::new("timestamp({c.args})"))', None)
        return (".timestamp()", None)
    if b == "date":
        return (".date()", None)
    if b == "time":
        return (".time()", None)
    if b == "decimal":
        p, s = (c.args.split(",") + ["0"])[:2]
        p, s = p.strip(), s.strip()
        if c.unsigned:
            return (f".decimal_len({p}, {s})", f'.custom(Alias::new("decimal({p},{s}) unsigned"))')
        return (f".decimal_len({p}, {s})", None)
    if b == "float":
        return (".float()", None)
    if b == "double":
        return (".double()", None)
    if b == "json":
        return (".json()", None)
    if b in ("boolean", "bool"):
        # MySQL renders BOOLEAN as tinyint(1); keep that exact on MySQL.
        return (".boolean()", '.custom(Alias::new("tinyint(1)"))')
    if b == "enum":
        variants = [unquote_sql(v.strip()) for v in split_top_level(c.args)]
        vlist = ", ".join(f"Alias::new({rs_str(v)})" for v in variants)
        name = enum_type_name(t.name, c.name)
        return (f'.enumeration(Alias::new("{name}"), [{vlist}])', None)
    if b == "set":
        my = f"set({c.args})"
        return (".string_len(64)", f'.custom(Alias::new("{my}"))')
    raise ValueError(f"unmapped type {b} on {t.name}.{c.name}")


def emit_column(t: Table, c: Column, single_pk: str) -> str:
    portable, mysql = column_type_calls(t, c)
    chain = ""
    if mysql:
        chain += f"\n            .apply(|c| match backend {{\n                DatabaseBackend::MySql => c{mysql},\n                _ => c{portable},\n            }})"
    else:
        chain += portable
    if not c.nullable:
        chain += ".not_null()"
    if c.autoinc:
        chain += ".auto_increment()"
    if c.name == single_pk:
        chain += ".primary_key()"
    if c.default is not None:
        kind = c.default[0]
        if kind == "str":
            chain += f".default({rs_str(c.default[1])})"
        elif kind == "num":
            v = c.default[1]
            if v.lower() in ("true", "false"):
                chain += f".default({v.lower()})"
            elif re.match(r"^[+-]?\d+$", v):
                chain += f".default({v})"
            else:
                chain += f".default({v}_f64)"
        elif kind == "now":
            chain += ".default(Expr::current_timestamp())"
        elif kind == "now_prec":
            chain += f'.default(Expr::cust("CURRENT_TIMESTAMP({c.default[1]})"))'
    if c.on_update_now:
        chain += (
            "\n            .apply(|c| match backend {\n"
            '                DatabaseBackend::MySql => c.extra("ON UPDATE CURRENT_TIMESTAMP"),\n'
            "                _ => c,\n            })"
        )
    return f"        .col(ColumnDef::new(Alias::new({rs_str(c.name)})){chain})"


def emit_table_fn(t: Table) -> str:
    single_pk = t.primary[0] if len(t.primary) == 1 else None
    lines = [
        f"pub(super) fn {snake(t.name)}_table(backend: DatabaseBackend) -> TableCreateStatement {{",
        "    let mut t = Table::create();",
        f"    t.table(Alias::new({rs_str(t.name)}))",
        "        .if_not_exists()",
    ]
    for c in t.columns:
        lines.append(emit_column(t, c, single_pk))
    if t.primary and not single_pk:
        cols = "".join(f".col(Alias::new({rs_str(c)}))" for c in t.primary)
        lines.append(f"        .primary_key(Index::create(){cols})")
    for (col, ref, refcol, on_delete, cname) in t.fks:
        fk = (
            f"        .foreign_key(ForeignKey::create()"
            + (f".name({rs_str(cname)})" if cname else "")
            + f".from(Alias::new({rs_str(t.name)}), Alias::new({rs_str(col)}))"
            f".to(Alias::new({rs_str(ref)}), Alias::new({rs_str(refcol)}))"
        )
        if on_delete == "CASCADE":
            fk += ".on_delete(ForeignKeyAction::Cascade)"
        elif on_delete == "SET_NULL":
            fk += ".on_delete(ForeignKeyAction::SetNull)"
        fk += ")"
        lines.append(fk)
    lines[-1] += ";"
    lines.append("    let _ = backend;")
    lines.append("    t")
    lines.append("}")
    return "\n".join(lines)


def emit_index_fns(t: Table) -> str:
    stmts = []
    for name, cols in t.uniques:
        colcalls = "".join(f".col(Alias::new({rs_str(c)}))" for c in cols)
        stmts.append(
            f"        Index::create().name({rs_str(name)}).table(Alias::new({rs_str(t.name)})){colcalls}.unique().to_owned(),"
        )
    for name, cols in t.keys:
        colcalls = "".join(f".col(Alias::new({rs_str(c)}))" for c in cols)
        stmts.append(
            f"        Index::create().name({rs_str(name)}).table(Alias::new({rs_str(t.name)})){colcalls}.to_owned(),"
        )
    if not stmts:
        return ""
    body = "\n".join(stmts)
    return (
        f"pub(super) fn {snake(t.name)}_indexes() -> Vec<IndexCreateStatement> {{\n"
        f"    vec![\n{body}\n    ]\n}}"
    )


def emit_enum_types(tables, order) -> str:
    lines = [
        "/// Every ENUM column as (postgres type name, variants). Used to create",
        "/// real enum types when the backend is Postgres; MySQL inlines them.",
        "pub(super) fn enum_types() -> Vec<(&'static str, Vec<&'static str>)> {",
        "    vec![",
    ]
    for tn in order:
        for c in tables[tn].columns:
            if c.base == "enum":
                variants = [unquote_sql(v.strip()) for v in split_top_level(c.args)]
                vlist = ", ".join(rs_str(v) for v in variants)
                lines.append(f"        ({rs_str(enum_type_name(tn, c.name))}, vec![{vlist}]),")
    lines.append("    ]")
    lines.append("}")
    return "\n".join(lines)


def value_token_to_rust(tok: str, subst: dict) -> str:
    tok = tok.strip()
    if tok.upper() == "NULL":
        return "SimpleExpr::Keyword(Keyword::Null)"
    if tok.lower() in ("true", "false"):
        return f"{tok.lower()}.into()"
    if tok.startswith("'"):
        s = unquote_sql(tok)
        for k, v in subst.items():
            if s == k:
                return f"{v}.to_string().into()"
            if k in s:
                s = s.replace(k, f"{{{v}}}")
                return f"format!({rs_str(s)}).into()"
        return f"{rs_str(s)}.into()"
    if re.match(r"^[+-]?\d+$", tok):
        return f"({tok}_i64).into()"
    if re.match(r"^[+-]?[\d.]+(e[+-]?\d+)?$", tok, re.I):
        return f"({tok}_f64).into()"
    raise ValueError(f"unparseable seed value: {tok[:40]}")


def emit_seeds(tables, inserts, raw_seed_sql) -> str:
    subst = {"@PKGDATADIR@": "PKGDATADIR", "@ZM_DIR_EVENTS@": "ZM_DIR_EVENTS"}
    out = [
        "//! Seed rows shipped with a fresh ZoneMinder database, converted from",
        "//! the INSERT statements in zm_create.sql.in. Auto-increment ids that",
        "//! the legacy SQL supplied as NULL are omitted so both MySQL and",
        "//! Postgres allocate them natively.",
        "",
        "use sea_orm_migration::prelude::*;",
        "use sea_orm_migration::sea_query::{Keyword, SimpleExpr};",
        "",
        "/// Install path default substituted for @ZM_DIR_EVENTS@ (the default",
        "/// Storage row); matches the Debian packaging.",
        'pub(super) const ZM_DIR_EVENTS: &str = "/var/cache/zoneminder/events";',
        "",
        "#[allow(clippy::vec_init_then_push, clippy::too_many_lines)]",
        "pub(super) fn seed_statements() -> Vec<InsertStatement> {",
        "    let mut stmts: Vec<InsertStatement> = Vec::new();",
    ]
    # Legacy files use INSERT IGNORE with overlapping rows (manufacturers.sql
    # ships both plain and IGNORE inserts). Resolve statically: keep the first
    # row per unique key and emit plain inserts.
    seen_keys = {}
    for ins in inserts:
        t = tables[ins.table]
        cols = ins.columns or [c.name for c in t.columns]
        if t.uniques:
            ucols = t.uniques[0][1]
            try:
                idxs = [cols.index(c) for c in ucols]
            except ValueError:
                idxs = None
            if idxs is not None:
                seen = seen_keys.setdefault(ins.table, set())
                kept = []
                for r in ins.rows:
                    key = tuple(r[i].strip() for i in idxs)
                    if key in seen:
                        continue
                    seen.add(key)
                    kept.append(r)
                ins.rows = kept
        if not ins.rows:
            continue
        by_name = {c.name: c for c in t.columns}
        # Drop a leading NULL auto-increment id if every row has it as NULL.
        drop_idx = None
        for i, cn in enumerate(cols):
            c = by_name.get(cn)
            if c is not None and c.autoinc:
                if all(r[i].strip().upper() == "NULL" for r in ins.rows):
                    drop_idx = i
                break
        use_cols = [c for i, c in enumerate(cols) if i != drop_idx]
        col_list = ", ".join(f"Alias::new({rs_str(c)})" for c in use_cols)
        out.append("    stmts.push({")
        out.append("        let mut s = Query::insert();")
        out.append(f"        s.into_table(Alias::new({rs_str(ins.table)}))")
        out.append(f"            .columns([{col_list}]);")
        for r in ins.rows:
            vals = [v for i, v in enumerate(r) if i != drop_idx]
            rvals = ", ".join(value_token_to_rust(v, subst) for v in vals)
            out.append(f"        s.values_panic([{rvals}]);")
        out.append("        s.to_owned()")
        out.append("    });")
    out.append("    stmts")
    out.append("}")
    out.append("")
    out.append("/// Derived seeds (INSERT ... SELECT) kept as raw portable SQL.")
    out.append("pub(super) fn raw_seed_sql() -> Vec<&'static str> {")
    out.append("    vec![")
    for sql in raw_seed_sql:
        out.append(f"        r#\"{sql}\"#,")
    out.append("    ]")
    out.append("}")
    return "\n".join(out)


def emit_triggers(triggers, tables, order) -> str:
    out = [
        "//! The 13 MySQL triggers that maintain Event_Summaries and the",
        "//! Events_Hour/Day/Week/Month rollup tables, plus Zones counts,",
        "//! extracted from db/triggers.sql. MySQL dialect only - the Postgres",
        "//! port (functions + triggers) lands with the Postgres schema work;",
        "//! until then Postgres installs run without summary triggers.",
        "",
        "pub(super) fn mysql_triggers() -> Vec<&'static str> {",
        "    vec![",
    ]
    for t in triggers:
        out.append(f"        r#\"{t}\"#,")
    out.append("    ]")
    out.append("}")
    out.append("")
    out.append("/// Tables with an auto-increment id whose seeds insert explicit ids;")
    out.append("/// Postgres sequences must be advanced past them after seeding.")
    out.append("pub(super) fn autoinc_tables() -> Vec<(&'static str, &'static str)> {")
    out.append("    vec![")
    for tn in order:
        for c in tables[tn].columns:
            if c.autoinc:
                out.append(f"        ({rs_str(tn)}, {rs_str(c.name)}),")
    out.append("    ]")
    out.append("}")
    return "\n".join(out)


def topo_order(tables, order):
    """Creation order honoring FK dependencies (Kahn, stable on file order).
    Self-references are fine inline and ignored here."""
    remaining = list(order)
    done, out = set(), []
    while remaining:
        progressed = False
        for tn in list(remaining):
            deps = {ref for (_c, ref, _rc, _d, _n) in tables[tn].fks if ref != tn}
            if deps <= done:
                out.append(tn)
                done.add(tn)
                remaining.remove(tn)
                progressed = True
        if not progressed:
            raise ValueError(f"FK cycle among: {remaining}")
    return out


def emit_tables_rs(tables, order) -> str:
    order = topo_order(tables, order)
    parts = [
        "//! Generated DDL builders for every ZoneMinder table, one function per",
        "//! table plus its secondary indexes. Generated by",
        "//! scripts/gen_baseline_migration.py from zm_create.sql.in - edit the",
        "//! generator, not this file.",
        "",
        "use sea_orm_migration::prelude::*;",
        "use sea_orm_migration::sea_orm::DatabaseBackend;",
        "",
        "/// Chain helper so backend-specific column types stay inline.",
        "pub(super) trait ColumnDefApply {",
        "    fn apply(&mut self, f: impl FnOnce(&mut Self) -> &mut Self) -> &mut Self;",
        "}",
        "",
        "impl ColumnDefApply for ColumnDef {",
        "    fn apply(&mut self, f: impl FnOnce(&mut Self) -> &mut Self) -> &mut Self {",
        "        f(self)",
        "    }",
        "}",
        "",
    ]
    for tn in order:
        parts.append(emit_table_fn(tables[tn]))
        parts.append("")
        idx = emit_index_fns(tables[tn])
        if idx:
            parts.append(idx)
            parts.append("")
    parts.append(emit_enum_types(tables, order))
    parts.append("")
    table_list = ",\n        ".join(f"({rs_str(tn)}, {snake(tn)}_table as TableFn)" for tn in order)
    parts.append("pub(super) type TableFn = fn(DatabaseBackend) -> TableCreateStatement;")
    parts.append("")
    parts.append("/// Creation order (legacy file order); drop in reverse.")
    parts.append("pub(super) fn all_tables() -> Vec<(&'static str, TableFn)> {")
    parts.append(f"    vec![\n        {table_list},\n    ]")
    parts.append("}")
    parts.append("")
    idx_tables = [tn for tn in order if tables[tn].uniques or tables[tn].keys]
    idx_list = ",\n        ".join(f"{snake(tn)}_indexes()" for tn in idx_tables)
    parts.append("pub(super) fn all_indexes() -> Vec<Vec<IndexCreateStatement>> {")
    parts.append(f"    vec![\n        {idx_list},\n    ]")
    parts.append("}")
    return "\n".join(parts)


MOD_RS = '''//! Baseline migration: the full ZoneMinder schema (54 tables, 13 MySQL
//! triggers) plus seed rows, expressed as portable SeaORM DDL. Replaces
//! zm_create.sql.in (and its sourced db/*.sql fragments) as the source of
//! truth for fresh installs (issue #11, migration system phase 1).
//!
//! Legacy installs never run this migration - the upgrade bridge walks the
//! frozen zm_update-*.sql chain to the cutover release and then records this
//! migration as applied without executing it ("baseline stamping").
//!
//! DDL builders live in `tables.rs` (generated - see
//! scripts/gen_baseline_migration.py) so statements can be rendered and
//! asserted offline, following the event_synopsis pattern.

mod seeds;
mod tables;
mod triggers;

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::{DatabaseBackend, Statement};
use sea_orm_migration::sea_query::extension::postgres::Type;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let backend = manager.get_database_backend();

        if backend == DatabaseBackend::Postgres {
            for (name, variants) in tables::enum_types() {
                manager
                    .create_type(
                        Type::create()
                            .as_enum(Alias::new(name))
                            .values(variants.into_iter().map(Alias::new))
                            .to_owned(),
                    )
                    .await?;
            }
        }

        for (_name, table_fn) in tables::all_tables() {
            manager.create_table(table_fn(backend)).await?;
        }

        for group in tables::all_indexes() {
            for idx in group {
                manager.create_index(idx).await?;
            }
        }

        let conn = manager.get_connection();
        for stmt in seeds::seed_statements() {
            conn.execute(backend.build(&stmt)).await?;
        }
        for sql in seeds::raw_seed_sql() {
            conn.execute_unprepared(sql).await?;
        }

        match backend {
            DatabaseBackend::MySql => {
                // Summary/rollup maintenance triggers (MySQL dialect).
                for sql in triggers::mysql_triggers() {
                    conn.execute_unprepared(sql).await?;
                }
            }
            DatabaseBackend::Postgres => {
                // Seeds insert explicit auto-increment ids; advance the
                // sequences so later inserts don't collide.
                for (table, col) in triggers::autoinc_tables() {
                    let sql = format!(
                        "SELECT setval(pg_get_serial_sequence('\\"{table}\\"', '{col}'), \\
                         (SELECT COALESCE(MAX(\\"{col}\\"), 0) + 1 FROM \\"{table}\\"), false)"
                    );
                    conn.execute(Statement::from_string(backend, sql)).await?;
                }
            }
            DatabaseBackend::Sqlite => {}
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for (name, _table_fn) in tables::all_tables().into_iter().rev() {
            manager
                .drop_table(Table::drop().table(Alias::new(name)).if_exists().to_owned())
                .await?;
        }
        if manager.get_database_backend() == DatabaseBackend::Postgres {
            for (name, _variants) in tables::enum_types() {
                manager
                    .drop_type(Type::drop().name(Alias::new(name)).if_exists().to_owned())
                    .await?;
            }
        }
        Ok(())
    }
}
'''


def main():
    text = read_sql()
    stmts = split_statements(text)
    tables, order, inserts, raw_seed_sql = parse_tables_and_inserts(stmts)
    triggers = parse_triggers()
    print(f"parsed {len(tables)} tables, {len(inserts)} insert statements, {len(triggers)} triggers")
    assert len(tables) == 54, f"expected 54 tables, got {len(tables)}"
    # 12 active; event_insert_trigger is commented out upstream.
    assert len(triggers) == 12, f"expected 12 triggers, got {len(triggers)}"

    OUT_DIR.mkdir(parents=True, exist_ok=True)
    (OUT_DIR / "tables.rs").write_text(emit_tables_rs(tables, order) + "\n")
    (OUT_DIR / "seeds.rs").write_text(emit_seeds(tables, inserts, raw_seed_sql) + "\n")
    (OUT_DIR / "triggers.rs").write_text(emit_triggers(triggers, tables, order) + "\n")
    (OUT_DIR / "mod.rs").write_text(MOD_RS)
    # Committed output must satisfy `cargo fmt --check`.
    subprocess.run(
        ["rustfmt", "--edition", "2021"] + [str(f) for f in OUT_DIR.glob("*.rs")],
        check=True,
    )
    print(f"wrote {OUT_DIR}")


if __name__ == "__main__":
    sys.exit(main())
