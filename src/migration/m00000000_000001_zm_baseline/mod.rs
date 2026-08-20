//! Baseline migration: the full ZoneMinder schema (54 tables, 13 MySQL
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
pub(crate) mod triggers;

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::{DatabaseBackend, Statement};
use sea_orm_migration::sea_query::extension::postgres::Type;

pub struct Migration;

// Explicit: DeriveMigrationName resolves to the file stem, which for a
// directory module is "mod". The bridge stamps this exact string.
impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m00000000_000001_zm_baseline"
    }
}

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
                for (_name, sql) in triggers::mysql_triggers() {
                    conn.execute_unprepared(sql).await?;
                }
            }
            DatabaseBackend::Postgres => {
                // Seeds insert explicit auto-increment ids; advance the
                // sequences so later inserts don't collide.
                for (table, col) in triggers::autoinc_tables() {
                    let sql = format!(
                        "SELECT setval(pg_get_serial_sequence('\"{table}\"', '{col}'), \
                         (SELECT COALESCE(MAX(\"{col}\"), 0) + 1 FROM \"{table}\"), false)"
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
