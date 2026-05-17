//! Integration tests for the Logs API — happy-path plus error paths.
//!
//! Logs are system-generated, so the API is read-only (list + get).
//!
//! Requires the test database — run with:
//!   APP_PROFILE=test-db cargo test --test it_logs -- --include-ignored

mod common;

use axum::http::StatusCode;
use common::assertions::{assert_error, assert_status};
use common::harness::{superuser_token, TestApp};
use sea_orm::prelude::Decimal;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use zm_api::dto::response::logs::PaginatedLogsResponse;
use zm_api::dto::response::LogResponse;

/// Insert a `Logs` row directly and return its id.
async fn insert_log(db: &sea_orm::DatabaseConnection) -> u32 {
    zm_api::entity::logs::ActiveModel {
        time_key: Set(Decimal::from(1_700_000_000i64)),
        component: Set("it-test".to_string()),
        level: Set(0),
        code: Set("INF".to_string()),
        message: Set("integration test log entry".to_string()),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("insert log fixture")
    .id
}

async fn delete_log(db: &sea_orm::DatabaseConnection, id: u32) {
    let _ = zm_api::entity::logs::Entity::delete_by_id(id)
        .exec(db)
        .await;
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn list_logs_returns_inserted_row() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let id = insert_log(&app.db).await;

    let resp = app.get("/api/v3/logs?page=1&page_size=1000", &token).await;
    assert_status(&resp, StatusCode::OK);
    let body: PaginatedLogsResponse = resp.json();
    assert!(
        body.items.iter().any(|l| l.id == id),
        "list should contain the fixture log"
    );

    delete_log(&app.db, id).await;
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn get_log_returns_the_row() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let id = insert_log(&app.db).await;

    let resp = app.get(&format!("/api/v3/logs/{id}"), &token).await;
    assert_status(&resp, StatusCode::OK);
    let body: LogResponse = resp.json();
    assert_eq!(body.id, id);

    delete_log(&app.db, id).await;
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn get_missing_log_is_not_found() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    let resp = app.get("/api/v3/logs/999000111", &token).await;
    assert_error(&resp, StatusCode::NOT_FOUND, "MESSAGE_NOT_FOUND_ERROR");
}
