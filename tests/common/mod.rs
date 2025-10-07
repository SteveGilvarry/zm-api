// Common test helpers for integration tests
use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult};
use zm_api::server::state::AppState;

/// Create a test AppState with a mocked database
pub fn create_test_state() -> AppState {
    let db = MockDatabase::new(DatabaseBackend::MySql).into_connection();
    AppState::for_test_with_db(db)
}

/// Create a test AppState with exec results (for insert/update/delete operations)
pub fn create_test_state_with_exec(exec_results: Vec<MockExecResult>) -> AppState {
    let mut db = MockDatabase::new(DatabaseBackend::MySql);
    for result in exec_results {
        db = db.append_exec_results(vec![result]);
    }
    AppState::for_test_with_db(db.into_connection())
}
