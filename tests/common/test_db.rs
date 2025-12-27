// Integration test helper utilities for database testing

use sea_orm::{Database, DatabaseConnection, DbErr};
use std::env;

/// Get a connection to the test database
/// 
/// This creates a new database connection for integration tests.
/// Make sure to start the test database first:
/// ```bash
/// ./scripts/test-db.sh start
/// ```
pub async fn get_test_db() -> Result<DatabaseConnection, DbErr> {
    let db_url = env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
        "mysql://zm_test_user:zm_test_pass@127.0.0.1:3307/zm_test".to_string()
    });

    Database::connect(&db_url).await
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
