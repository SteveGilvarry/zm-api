//! Integration tests for the Reports API — happy-path plus error paths.
//!
//! Requires the test database — run with:
//!   APP_PROFILE=test-db cargo test --test it_reports -- --include-ignored

mod common;

use axum::http::StatusCode;
use common::assertions::{assert_error, assert_status};
use common::fixtures::RowGuard;
use common::harness::{superuser_token, TestApp};
use sea_orm::{ActiveModelTrait, Set};
use serde_json::json;
use zm_api::dto::response::{PaginatedReportsResponse, ReportResponse};

/// `Reports.Name` is `varchar(30)` — too short for the usual long test
/// prefix. Reports are not name-unique and the tests key off the id, so a
/// short constant name is sufficient.
const REPORT_NAME: &str = "ZM_IT_REPORT";

/// Insert a `Reports` row directly and return its id.
async fn insert_report(db: &sea_orm::DatabaseConnection) -> u32 {
    zm_api::entity::reports::ActiveModel {
        name: Set(Some(REPORT_NAME.to_string())),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("insert report fixture")
    .id
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn list_reports_returns_inserted_row() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let id = insert_report(&app.db).await;
    let _guard = RowGuard::report(id);

    let resp = app
        .get("/api/v3/reports?page=1&page_size=1000", &token)
        .await;
    assert_status(&resp, StatusCode::OK);
    let body: PaginatedReportsResponse = resp.json();
    assert!(
        body.items.iter().any(|r| r.id == id),
        "list should contain the fixture report"
    );
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn get_report_returns_the_row() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let id = insert_report(&app.db).await;
    let _guard = RowGuard::report(id);

    let resp = app.get(&format!("/api/v3/reports/{id}"), &token).await;
    assert_status(&resp, StatusCode::OK);
    let body: ReportResponse = resp.json();
    assert_eq!(body.id, id);
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn get_missing_report_is_not_found() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    let resp = app.get("/api/v3/reports/999000111", &token).await;
    assert_error(&resp, StatusCode::NOT_FOUND, "FILE_NOT_FOUND_ERROR");
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn create_then_delete_report_round_trips() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    let body = json!({
        "name": REPORT_NAME,
        "interval": 604800,
    });
    let create = app.post_json("/api/v3/reports", &token, &body).await;
    assert!(
        create.status().is_success(),
        "create should succeed; body: {}",
        create.text()
    );
    let created: ReportResponse = create.json();
    let _guard = RowGuard::report(created.id);

    let delete = app
        .delete(&format!("/api/v3/reports/{}", created.id), &token)
        .await;
    assert!(
        delete.status().is_success(),
        "delete should succeed; got {}",
        delete.status()
    );

    let get = app
        .get(&format!("/api/v3/reports/{}", created.id), &token)
        .await;
    assert_eq!(get.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn create_report_with_invalid_body_is_rejected() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    // `interval` must be a number, not a string.
    let resp = app
        .post_json(
            "/api/v3/reports",
            &token,
            &json!({ "interval": "not-a-number" }),
        )
        .await;
    assert!(
        resp.status().is_client_error(),
        "malformed create body should be a 4xx, got {}; body: {}",
        resp.status(),
        resp.text()
    );
}
