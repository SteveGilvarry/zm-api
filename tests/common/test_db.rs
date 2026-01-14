// Integration test helper utilities for database testing

use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbErr, Statement, Value};
use std::env;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

/// Get a connection to the test database
///
/// This creates a new database connection for integration tests.
/// Make sure to start the test database first:
/// ```bash
/// ./scripts/test-db.sh start
/// ```
pub async fn get_test_db() -> Result<DatabaseConnection, DbErr> {
    let db_url = env::var("TEST_DATABASE_URL")
        .or_else(|_| env::var("DATABASE_URL"))
        .unwrap_or_else(|_| "mysql://zmuser:zmpass@127.0.0.1:3307/zm_test".to_string());

    Database::connect(&db_url).await
}

/// Stable identifier for the current test run.
#[allow(dead_code)]
pub fn test_run_id() -> &'static str {
    static TEST_RUN_ID: OnceLock<String> = OnceLock::new();
    TEST_RUN_ID
        .get_or_init(|| {
            env::var("TEST_RUN_ID").unwrap_or_else(|_| {
                let nanos = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("time")
                    .as_nanos();
                format!("run_{nanos}")
            })
        })
        .as_str()
}

/// Prefix for test data that should be cleaned up.
#[allow(dead_code)]
pub fn test_prefix() -> String {
    format!("Test_{}", test_run_id())
}

/// Delete rows where `column` starts with `prefix`.
/// Table/column names are controlled by tests; keep inputs literal.
#[allow(dead_code)]
pub async fn cleanup_by_prefix(
    db: &DatabaseConnection,
    table: &str,
    column: &str,
    prefix: &str,
) -> Result<u64, DbErr> {
    let backend = db.get_database_backend();
    let sql = format!("DELETE FROM `{}` WHERE `{}` LIKE ?", table, column);
    let statement =
        Statement::from_sql_and_values(backend, sql, vec![Value::from(format!("{prefix}%"))]);
    let result = db.execute(statement).await?;
    Ok(result.rows_affected())
}

/// Cleanup multiple tables using the same prefix.
#[allow(dead_code)]
pub async fn cleanup_by_prefixes(
    db: &DatabaseConnection,
    prefix: &str,
    targets: &[(&str, &str)],
) -> Result<(), DbErr> {
    for (table, column) in targets {
        let _ = cleanup_by_prefix(db, table, column, prefix).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "Requires running test database"]
    async fn test_can_connect_to_db() {
        let result = get_test_db().await;
        assert!(result.is_ok(), "Should connect to test database");
    }
}
