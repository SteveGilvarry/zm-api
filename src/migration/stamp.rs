//! Baseline stamping for databases whose ZoneMinder schema already exists.
//!
//! The baseline migration creates the full ZM schema and must never execute
//! against a database that already has one (duplicate tables/indexes/seeds).
//! Two situations produce such a database:
//!
//! - CI / dev environments that load `zm_create.sql.in` directly (current
//!   schema, externally created): record the baseline as applied and move on.
//! - A real legacy install at an older schema version: refuse with a pointer
//!   to `migrator bridge`, which walks the update chain first (phase 2).
//!
//! The two are distinguished by a marker column that only the current schema
//! has (`Monitors.ObjectDetection`, also added by the final legacy chain
//! file, so bridged databases pass the check too).

use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseConnection, DbErr, Statement};
use sea_orm_migration::MigratorTrait;
use tracing::info;

use super::Migrator;

/// The baseline migration's recorded version string. Must match
/// `m00000000_000001_zm_baseline::Migration::name()` (asserted in tests).
pub const BASELINE_VERSION: &str = "m00000000_000001_zm_baseline";

async fn exists(conn: &DatabaseConnection, sql: &str, param: &str) -> Result<bool, DbErr> {
    Ok(conn
        .query_one(Statement::from_sql_and_values(
            conn.get_database_backend(),
            sql,
            vec![param.into()],
        ))
        .await?
        .is_some())
}

async fn table_exists(conn: &DatabaseConnection, table: &str) -> Result<bool, DbErr> {
    exists(
        conn,
        "SELECT TABLE_NAME FROM information_schema.TABLES \
         WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = ?",
        table,
    )
    .await
}

/// Record the baseline as applied in `seaql_migrations` without running it.
pub async fn record_baseline(conn: &DatabaseConnection) -> Result<(), DbErr> {
    Migrator::install(conn).await?;
    conn.execute(Statement::from_sql_and_values(
        conn.get_database_backend(),
        "INSERT INTO seaql_migrations (version, applied_at) VALUES (?, UNIX_TIMESTAMP())",
        vec![BASELINE_VERSION.into()],
    ))
    .await?;
    Ok(())
}

/// Stamp the baseline when the current ZM schema already exists; error when
/// an older legacy schema is found. Returns true when a stamp was recorded.
pub async fn stamp_baseline_if_schema_current(conn: &DatabaseConnection) -> Result<bool, DbErr> {
    if conn.get_database_backend() != DatabaseBackend::MySql {
        // Non-MySQL databases can only have been created by the baseline
        // itself, which records its own application.
        return Ok(false);
    }
    if !table_exists(conn, "Monitors").await? {
        return Ok(false); // fresh database: let the baseline run
    }
    if table_exists(conn, "seaql_migrations").await?
        && exists(
            conn,
            "SELECT version FROM seaql_migrations WHERE version = ?",
            BASELINE_VERSION,
        )
        .await?
    {
        return Ok(false); // already stamped or applied
    }
    let current = exists(
        conn,
        "SELECT COLUMN_NAME FROM information_schema.COLUMNS \
         WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'Monitors' AND COLUMN_NAME = ?",
        "ObjectDetection",
    )
    .await?;
    if !current {
        return Err(DbErr::Custom(
            "existing ZoneMinder schema predates the baseline migration; \
             run `migrator bridge` to upgrade it onto the migration system"
                .into(),
        ));
    }
    info!("existing current ZoneMinder schema found; stamping baseline as applied");
    record_baseline(conn).await?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use sea_orm_migration::MigrationName;

    #[test]
    fn baseline_version_matches_migration_name() {
        assert_eq!(
            super::super::m00000000_000001_zm_baseline::Migration.name(),
            super::BASELINE_VERSION
        );
    }

    /// Needs STAMP_TEST_DATABASE_URL pointing at a database freshly loaded
    /// from zm_create.sql.in (no seaql_migrations), e.g. the parity_legacy
    /// DB left behind by scripts/schema-parity.sh.
    #[tokio::test]
    #[ignore]
    async fn stamps_externally_created_schema_then_migrates() {
        let url = std::env::var("STAMP_TEST_DATABASE_URL").expect("STAMP_TEST_DATABASE_URL");
        let conn = sea_orm::Database::connect(&url).await.expect("connect");
        assert!(super::stamp_baseline_if_schema_current(&conn)
            .await
            .expect("first stamp"));
        assert!(!super::stamp_baseline_if_schema_current(&conn)
            .await
            .expect("second stamp is a no-op"));
        use sea_orm_migration::MigratorTrait;
        super::Migrator::up(&conn, None)
            .await
            .expect("remaining migrations apply cleanly");
    }
}
