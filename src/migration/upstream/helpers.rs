//! Portable building blocks for the mirrored upstream migrations.
//!
//! Every `zm_update-1.39.x.sql` is a MySQL client script: `PREPARE` around an
//! `INFORMATION_SCHEMA` probe, `DELIMITER` for stored procedures, backtick
//! quoting. The migrations in this module express the same change with
//! sea-query DDL, which renders for MySQL/MariaDB and Postgres alike, and
//! with the existence probes below so each migration is safe to re-run and
//! safe on a database that reached the same state another way.

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::{ConnectionTrait, DatabaseBackend, Statement, StatementBuilder};
use sea_orm_migration::sea_query::extension::postgres::Type;

pub fn t(name: &str) -> Alias {
    Alias::new(name)
}

pub fn c(name: &str) -> Alias {
    Alias::new(name)
}

pub fn backend(m: &SchemaManager<'_>) -> DatabaseBackend {
    m.get_connection().get_database_backend()
}

async fn probe(
    m: &SchemaManager<'_>,
    mysql: &str,
    pg: &str,
    params: &[&str],
) -> Result<bool, DbErr> {
    let conn = m.get_connection();
    let values = params.iter().map(|p| (*p).into()).collect::<Vec<_>>();
    let stmt = match backend(m) {
        DatabaseBackend::MySql => {
            Statement::from_sql_and_values(DatabaseBackend::MySql, mysql, values)
        }
        _ => Statement::from_sql_and_values(DatabaseBackend::Postgres, pg, values),
    };
    Ok(conn.query_one(stmt).await?.is_some())
}

pub async fn table_exists(m: &SchemaManager<'_>, table: &str) -> Result<bool, DbErr> {
    probe(
        m,
        "SELECT 1 FROM information_schema.TABLES WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = ?",
        "SELECT 1 FROM information_schema.tables WHERE table_schema = current_schema() AND table_name = $1",
        &[table],
    )
    .await
}

pub async fn column_exists(
    m: &SchemaManager<'_>,
    table: &str,
    column: &str,
) -> Result<bool, DbErr> {
    probe(
        m,
        "SELECT 1 FROM information_schema.COLUMNS WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = ? AND COLUMN_NAME = ?",
        "SELECT 1 FROM information_schema.columns WHERE table_schema = current_schema() AND table_name = $1 AND column_name = $2",
        &[table, column],
    )
    .await
}

/// The column's `DATA_TYPE` as information_schema reports it (`decimal`,
/// `varchar`, `numeric`, ...), or `None` when the column is absent.
pub async fn column_data_type(
    m: &SchemaManager<'_>,
    table: &str,
    column: &str,
) -> Result<Option<String>, DbErr> {
    let conn = m.get_connection();
    let stmt = match backend(m) {
        DatabaseBackend::MySql => Statement::from_sql_and_values(
            DatabaseBackend::MySql,
            "SELECT DATA_TYPE AS data_type FROM information_schema.COLUMNS WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = ? AND COLUMN_NAME = ?",
            [table.into(), column.into()],
        ),
        _ => Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "SELECT data_type FROM information_schema.columns WHERE table_schema = current_schema() AND table_name = $1 AND column_name = $2",
            [table.into(), column.into()],
        ),
    };
    Ok(conn
        .query_one(stmt)
        .await?
        .and_then(|r| r.try_get::<String>("", "data_type").ok())
        .map(|s| s.to_lowercase()))
}

/// Index names are per-table in MySQL but per-schema in Postgres, and
/// ZoneMinder reuses names like `Name` across tables: keep upstream's exact
/// name on MySQL (the parity job compares it) and prefix the table on
/// Postgres unless the name already starts with it. Same rule as the
/// baseline generator.
pub fn index_name(m: &SchemaManager<'_>, table: &str, name: &str) -> String {
    match backend(m) {
        DatabaseBackend::MySql => name.to_string(),
        _ if name.starts_with(table) => name.to_string(),
        _ => format!("{table}_{name}"),
    }
}

/// `(index name, columns, unique)` for [`create_table`].
pub type IndexSpec<'a> = (&'a str, &'a [&'a str], bool);

/// Create a table with its secondary indexes. MySQL takes them inline
/// (`KEY ...` in CREATE TABLE, which is how upstream declares them and what
/// the parity job compares against); Postgres has no inline index syntax, so
/// there they are created afterwards, with per-backend names.
pub async fn create_table(
    m: &SchemaManager<'_>,
    mut stmt: TableCreateStatement,
    table: &str,
    indexes: &[IndexSpec<'_>],
) -> Result<(), DbErr> {
    if backend(m) == DatabaseBackend::MySql {
        for (name, cols, unique) in indexes {
            let mut idx = Index::create();
            idx.name(*name);
            for col in cols.iter() {
                idx.col(c(col));
            }
            if *unique {
                idx.unique();
            }
            stmt.index(&mut idx);
        }
        return run(m, &stmt).await;
    }
    run(m, &stmt).await?;
    for (name, cols, unique) in indexes {
        add_index_if_missing(m, table, name, cols, *unique).await?;
    }
    Ok(())
}

pub async fn index_exists(m: &SchemaManager<'_>, table: &str, index: &str) -> Result<bool, DbErr> {
    let index = index_name(m, table, index);
    let index = index.as_str();
    probe(
        m,
        "SELECT 1 FROM information_schema.STATISTICS WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = ? AND INDEX_NAME = ?",
        "SELECT 1 FROM pg_indexes WHERE schemaname = current_schema() AND tablename = $1 AND indexname = $2",
        &[table, index],
    )
    .await
}

/// Execute a DDL/DML statement, naming the rendered SQL in any error — a
/// migration that fails on one backend must say what it sent.
pub async fn run<S: StatementBuilder>(m: &SchemaManager<'_>, stmt: &S) -> Result<(), DbErr> {
    let built = backend(m).build(stmt);
    let head: String = built.sql.chars().take(200).collect();
    m.get_connection()
        .execute(built)
        .await
        .map(|_| ())
        .map_err(|e| DbErr::Custom(format!("{e}\n  while executing: {head}")))
}

pub async fn add_column_if_missing(
    m: &SchemaManager<'_>,
    table: &str,
    column: &str,
    def: ColumnDef,
) -> Result<(), DbErr> {
    if column_exists(m, table, column).await? {
        return Ok(());
    }
    let mut def = def;
    run(
        m,
        &Table::alter()
            .table(t(table))
            .add_column(&mut def)
            .to_owned(),
    )
    .await
}

pub async fn drop_column_if_present(
    m: &SchemaManager<'_>,
    table: &str,
    column: &str,
) -> Result<(), DbErr> {
    if !column_exists(m, table, column).await? {
        return Ok(());
    }
    run(
        m,
        &Table::alter()
            .table(t(table))
            .drop_column(c(column))
            .to_owned(),
    )
    .await
}

/// `ALTER TABLE ... MODIFY` (MySQL) / `ALTER COLUMN` (Postgres) with the
/// column's full new definition.
pub async fn modify_column(
    m: &SchemaManager<'_>,
    table: &str,
    def: ColumnDef,
) -> Result<(), DbErr> {
    let mut def = def;
    run(
        m,
        &Table::alter()
            .table(t(table))
            .modify_column(&mut def)
            .to_owned(),
    )
    .await
}

pub async fn add_index_if_missing(
    m: &SchemaManager<'_>,
    table: &str,
    name: &str,
    columns: &[&str],
    unique: bool,
) -> Result<(), DbErr> {
    if index_exists(m, table, name).await? {
        return Ok(());
    }
    let mut idx = Index::create();
    idx.name(index_name(m, table, name)).table(t(table));
    for col in columns {
        idx.col(c(col));
    }
    if unique {
        idx.unique();
    }
    run(m, &idx.to_owned()).await
}

pub async fn drop_index_if_present(
    m: &SchemaManager<'_>,
    table: &str,
    name: &str,
) -> Result<(), DbErr> {
    if !index_exists(m, table, name).await? {
        return Ok(());
    }
    run(
        m,
        &Index::drop()
            .name(index_name(m, table, name))
            .table(t(table))
            .to_owned(),
    )
    .await
}

/// Postgres: create the named enum type if it does not exist. MySQL has
/// inline ENUM columns and needs nothing.
pub async fn ensure_enum_type(
    m: &SchemaManager<'_>,
    name: &str,
    values: &[&str],
) -> Result<(), DbErr> {
    if backend(m) != DatabaseBackend::Postgres {
        return Ok(());
    }
    let exists = m
        .get_connection()
        .query_one(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "SELECT 1 FROM pg_type WHERE typname = $1",
            [name.into()],
        ))
        .await?
        .is_some();
    if exists {
        return Ok(());
    }
    run(
        m,
        &Type::create()
            .as_enum(t(name))
            .values(values.iter().map(|v| c(v)).collect::<Vec<_>>())
            .to_owned(),
    )
    .await
}

/// Add variants to an enum. MySQL re-declares the column with the full list
/// (the caller passes the complete new definition); Postgres appends the
/// values to the named type, then applies the definition for its default.
pub async fn widen_enum(
    m: &SchemaManager<'_>,
    table: &str,
    column: &str,
    type_name: &str,
    all_values: &[&str],
    def: ColumnDef,
) -> Result<(), DbErr> {
    if backend(m) == DatabaseBackend::Postgres {
        for v in all_values {
            m.get_connection()
                .execute(Statement::from_string(
                    DatabaseBackend::Postgres,
                    format!("ALTER TYPE \"{type_name}\" ADD VALUE IF NOT EXISTS '{v}'"),
                ))
                .await?;
        }
    }
    let _ = column;
    modify_column(m, table, def).await
}

/// Portable `INSERT ... WHERE NOT EXISTS (probe)`: insert `row` unless
/// `probe` finds a match.
pub async fn insert_if_absent(
    m: &SchemaManager<'_>,
    probe: SelectStatement,
    insert: InsertStatement,
) -> Result<bool, DbErr> {
    let conn = m.get_connection();
    let found = conn.query_one(backend(m).build(&probe)).await?.is_some();
    if found {
        return Ok(false);
    }
    conn.execute(backend(m).build(&insert)).await?;
    Ok(true)
}

/// `SELECT 1 FROM table WHERE col = value`.
pub fn probe_eq(table: &str, col: &str, value: &str) -> SelectStatement {
    Query::select()
        .expr(Expr::value(1))
        .from(t(table))
        .and_where(Expr::col(c(col)).eq(value))
        .to_owned()
}

pub async fn exec_stmt<S: StatementBuilder>(m: &SchemaManager<'_>, stmt: &S) -> Result<u64, DbErr> {
    Ok(m.get_connection()
        .execute(backend(m).build(stmt))
        .await?
        .rows_affected())
}

/// `CAST(col AS BIGINT)`, so unsigned MySQL integers and Postgres integers
/// both decode as `i64` without sqlx refusing the unsigned→signed mapping.
pub fn as_i64(m: &SchemaManager<'_>, col: &str) -> SimpleExpr {
    let target = match backend(m) {
        DatabaseBackend::MySql => "SIGNED",
        _ => "BIGINT",
    };
    Expr::col(c(col)).cast_as(Alias::new(target))
}

/// `as_i64` with the column qualified by table, for joins.
pub fn as_i64_of(m: &SchemaManager<'_>, table: &str, col: &str) -> SimpleExpr {
    let target = match backend(m) {
        DatabaseBackend::MySql => "SIGNED",
        _ => "BIGINT",
    };
    Expr::col((t(table), c(col))).cast_as(Alias::new(target))
}

/// `as_f64` with the column qualified by table, for joins.
pub fn as_f64_of(m: &SchemaManager<'_>, table: &str, col: &str) -> SimpleExpr {
    let target = match backend(m) {
        DatabaseBackend::MySql => "DOUBLE",
        _ => "DOUBLE PRECISION",
    };
    Expr::col((t(table), c(col))).cast_as(Alias::new(target))
}

/// `CAST(col AS DOUBLE)` for decimal columns read into `f64`.
pub fn as_f64(m: &SchemaManager<'_>, col: &str) -> SimpleExpr {
    let target = match backend(m) {
        DatabaseBackend::MySql => "DOUBLE",
        _ => "DOUBLE PRECISION",
    };
    Expr::col(c(col)).cast_as(Alias::new(target))
}

/// Backend-specific column type helpers matching the baseline generator's
/// mapping (`scripts/gen_baseline_migration.py`), so a column added by a
/// migration is byte-identical in `information_schema` to the same column
/// created fresh — which is what the parity job compares.
pub fn tinytext(m: &SchemaManager<'_>, col: &str) -> ColumnDef {
    let mut d = ColumnDef::new(c(col));
    match backend(m) {
        DatabaseBackend::MySql => d.custom(Alias::new("tinytext")),
        _ => d.text(),
    };
    d
}

pub fn decimal_unsigned(m: &SchemaManager<'_>, col: &str, p: u32, s: u32) -> ColumnDef {
    let mut d = ColumnDef::new(c(col));
    match backend(m) {
        DatabaseBackend::MySql => d.custom(Alias::new(format!("decimal({p},{s}) unsigned"))),
        _ => d.decimal_len(p, s),
    };
    d
}

/// An auto-increment primary key: `int unsigned` (or `bigint unsigned`) on
/// MySQL, `serial`/`bigserial` on Postgres, which has no unsigned serial and
/// where sea-query refuses `unsigned` + `auto_increment`.
pub fn autoinc_pk(m: &SchemaManager<'_>, col: &str, big: bool) -> ColumnDef {
    let mut d = ColumnDef::new(c(col));
    match (backend(m), big) {
        (DatabaseBackend::MySql, false) => d.unsigned(),
        (DatabaseBackend::MySql, true) => d.big_unsigned(),
        (_, false) => d.integer(),
        (_, true) => d.big_integer(),
    };
    d.not_null().auto_increment().primary_key();
    d
}

/// `tinyint(1)` — MySQL's rendering of BOOLEAN — for columns upstream
/// declares as `tinyint(1)` without `unsigned`.
pub fn tinyint1(m: &SchemaManager<'_>, col: &str, default: Option<bool>) -> ColumnDef {
    let mut d = ColumnDef::new(c(col));
    match backend(m) {
        DatabaseBackend::MySql => {
            d.custom(Alias::new("tinyint(1)"));
            if let Some(v) = default {
                d.default(if v { 1 } else { 0 });
            }
        }
        _ => {
            d.boolean();
            if let Some(v) = default {
                d.default(v);
            }
        }
    };
    d
}

/// An enum member as a value: plain on MySQL, `CAST(... AS type)` on
/// Postgres, where a text parameter does not compare to or assign into an
/// enum column.
pub fn enum_val(type_name: &str, value: &str) -> SimpleExpr {
    Expr::val(value).as_enum(Alias::new(type_name))
}

pub fn enum_col(col: &str, type_name: &str, values: &[&str]) -> ColumnDef {
    let mut d = ColumnDef::new(c(col));
    d.enumeration(
        Alias::new(type_name),
        values.iter().map(|v| Alias::new(*v)).collect::<Vec<_>>(),
    );
    d
}

/// A `timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP`
/// column; the ON UPDATE half exists only on MySQL (the baseline makes the
/// same choice).
pub fn timestamp_on_update(m: &SchemaManager<'_>, col: &str) -> ColumnDef {
    let mut d = ColumnDef::new(c(col));
    d.timestamp().not_null().default(Expr::current_timestamp());
    if backend(m) == DatabaseBackend::MySql {
        d.extra("ON UPDATE CURRENT_TIMESTAMP");
    }
    d
}
