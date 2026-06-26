//! Integration tests for NL/semantic event search (Phase 0).
//!
//! Verifies the *functional* vector-capability probe against the real DB: on a
//! server without native `VECTOR` (the box is MySQL 8.0), `auto` must resolve to
//! the sqlite floor and a strict `mariadb` request must disable search.
//!
//! Requires the test database — run with:
//!   APP_PROFILE=test-db cargo test --test it_search -- --include-ignored

mod common;

use common::test_db::get_test_db;
use zm_api::configure::search::BackendPref;
use zm_api::service::search::store::{detect_backend, probe_mariadb_vector, Backend};

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn capability_probe_resolves_backend() {
    let db = get_test_db().await.expect("test db");

    // The functional probe (CREATE TEMPORARY TABLE … VECTOR(3)) — true only on a
    // native-vector server (MariaDB 11.8+ / MySQL 9+). The CI/dev box is MySQL
    // 8.0, so this is false.
    let native = probe_mariadb_vector(&db).await;

    if native {
        assert_eq!(
            detect_backend(&db, BackendPref::Auto).await,
            Backend::Mariadb
        );
        assert_eq!(
            detect_backend(&db, BackendPref::Mariadb).await,
            Backend::Mariadb
        );
    } else {
        // auto falls to the universal sqlite floor; strict mariadb disables.
        assert_eq!(
            detect_backend(&db, BackendPref::Auto).await,
            Backend::Sqlite
        );
        assert_eq!(
            detect_backend(&db, BackendPref::Mariadb).await,
            Backend::None
        );
    }

    // Explicit preferences never probe.
    assert_eq!(
        detect_backend(&db, BackendPref::Sqlite).await,
        Backend::Sqlite
    );
    assert_eq!(detect_backend(&db, BackendPref::None).await, Backend::None);
}
