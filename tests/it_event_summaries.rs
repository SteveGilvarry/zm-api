//! Integration tests for the Event Summaries API — read-only (list + get).
//!
//! Requires the test database — run with:
//!   APP_PROFILE=test-db cargo test --test it_event_summaries -- --include-ignored

mod common;

use axum::http::StatusCode;
use common::assertions::{assert_error, assert_status};
use common::fixtures::{insert_monitor, RowGuard};
use common::harness::{superuser_token, TestApp};
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use zm_api::dto::response::{EventSummaryResponse, PaginatedEventSummariesResponse};

/// Guard an `Event_Summaries` row (keyed by monitor id).
fn guard_summary(monitor_id: u32) -> RowGuard {
    RowGuard::new(
        format!("Event_Summaries#{monitor_id}"),
        move |db| async move {
            let _ = zm_api::entity::event_summaries::Entity::delete_by_id(monitor_id)
                .exec(&db)
                .await;
        },
    )
}

/// Insert an `Event_Summaries` row for a monitor.
async fn insert_summary(db: &sea_orm::DatabaseConnection, monitor_id: u32) {
    zm_api::entity::event_summaries::ActiveModel {
        monitor_id: Set(monitor_id),
        total_events: Set(Some(42)),
        total_event_disk_space: Set(Some(1024)),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("insert event summary fixture");
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn list_event_summaries_returns_inserted_row() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let monitor = insert_monitor(&app.db, "SummaryList")
        .await
        .expect("insert monitor");
    let _mon = RowGuard::monitor(monitor.id);
    insert_summary(&app.db, monitor.id).await;
    let _summary = guard_summary(monitor.id);

    let resp = app
        .get("/api/v3/event-summaries?page=1&page_size=1000", &token)
        .await;
    assert_status(&resp, StatusCode::OK);
    let body: PaginatedEventSummariesResponse = resp.json();
    assert!(
        body.items.iter().any(|s| s.monitor_id == monitor.id),
        "event summary list should contain the fixture row"
    );
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn get_event_summary_returns_the_row() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let monitor = insert_monitor(&app.db, "SummaryGet")
        .await
        .expect("insert monitor");
    let _mon = RowGuard::monitor(monitor.id);
    insert_summary(&app.db, monitor.id).await;
    let _summary = guard_summary(monitor.id);

    let resp = app
        .get(&format!("/api/v3/event-summaries/{}", monitor.id), &token)
        .await;
    assert_status(&resp, StatusCode::OK);
    let body: EventSummaryResponse = resp.json();
    assert_eq!(body.monitor_id, monitor.id);
    assert_eq!(body.total_events, 42);
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn get_missing_event_summary_is_not_found() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    let resp = app.get("/api/v3/event-summaries/999000111", &token).await;
    assert_error(
        &resp,
        StatusCode::NOT_FOUND,
        "EVENT_SUMMARY_NOT_FOUND_ERROR",
    );
}
