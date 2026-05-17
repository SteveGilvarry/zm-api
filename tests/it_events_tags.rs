//! Integration tests for the Events-Tags API — happy-path plus error paths.
//!
//! Requires the test database — run with:
//!   APP_PROFILE=test-db cargo test --test it_events_tags -- --include-ignored

mod common;

use axum::http::StatusCode;
use common::assertions::{assert_error, assert_status};
use common::fixtures::{delete_monitor, insert_monitor, unique_name};
use common::harness::{superuser_token, TestApp};
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use serde_json::json;
use zm_api::dto::response::{EventTagResponse, PaginatedEventsTagsResponse};

/// Insert an `Events` row for a monitor and return its id.
async fn insert_event(db: &sea_orm::DatabaseConnection, monitor_id: u32, label: &str) -> u64 {
    zm_api::entity::events::ActiveModel {
        monitor_id: Set(monitor_id),
        state_id: Set(1),
        name: Set(unique_name(label)),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("insert event fixture")
    .id
}

/// Insert a `Tags` row and return its id.
async fn insert_tag(db: &sea_orm::DatabaseConnection, label: &str) -> u64 {
    zm_api::entity::tags::ActiveModel {
        name: Set(unique_name(label)),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("insert tag fixture")
    .id
}

/// Insert an `Events_Tags` association directly.
async fn insert_event_tag(db: &sea_orm::DatabaseConnection, tag_id: u64, event_id: u64) {
    zm_api::entity::events_tags::ActiveModel {
        tag_id: Set(tag_id),
        event_id: Set(event_id),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("insert events_tags fixture");
}

async fn delete_event_tag_row(db: &sea_orm::DatabaseConnection, tag_id: u64, event_id: u64) {
    let _ = zm_api::entity::events_tags::Entity::delete_by_id((tag_id, event_id))
        .exec(db)
        .await;
}

async fn delete_event(db: &sea_orm::DatabaseConnection, id: u64) {
    let _ = zm_api::entity::events::Entity::delete_by_id(id)
        .exec(db)
        .await;
}

async fn delete_tag(db: &sea_orm::DatabaseConnection, id: u64) {
    let _ = zm_api::entity::tags::Entity::delete_by_id(id)
        .exec(db)
        .await;
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn list_events_tags_returns_inserted_row() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let monitor = insert_monitor(&app.db, "EvtTagList")
        .await
        .expect("insert monitor");
    let event_id = insert_event(&app.db, monitor.id, "EvtTagListEvt").await;
    let tag_id = insert_tag(&app.db, "EvtTagListTag").await;
    insert_event_tag(&app.db, tag_id, event_id).await;

    let resp = app
        .get(
            &format!("/api/v3/events-tags?event_id={event_id}&page=1&page_size=1000"),
            &token,
        )
        .await;
    assert_status(&resp, StatusCode::OK);
    let body: PaginatedEventsTagsResponse = resp.json();
    assert!(
        body.items
            .iter()
            .any(|t| t.tag_id == tag_id && t.event_id == event_id),
        "list should contain the fixture association"
    );

    delete_event_tag_row(&app.db, tag_id, event_id).await;
    delete_event(&app.db, event_id).await;
    delete_tag(&app.db, tag_id).await;
    delete_monitor(&app.db, monitor.id).await.expect("cleanup");
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn get_event_tag_returns_the_row() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let monitor = insert_monitor(&app.db, "EvtTagGet")
        .await
        .expect("insert monitor");
    let event_id = insert_event(&app.db, monitor.id, "EvtTagGetEvt").await;
    let tag_id = insert_tag(&app.db, "EvtTagGetTag").await;
    insert_event_tag(&app.db, tag_id, event_id).await;

    let resp = app
        .get(&format!("/api/v3/events-tags/{tag_id}/{event_id}"), &token)
        .await;
    assert_status(&resp, StatusCode::OK);
    let body: EventTagResponse = resp.json();
    assert_eq!(body.tag_id, tag_id);
    assert_eq!(body.event_id, event_id);

    delete_event_tag_row(&app.db, tag_id, event_id).await;
    delete_event(&app.db, event_id).await;
    delete_tag(&app.db, tag_id).await;
    delete_monitor(&app.db, monitor.id).await.expect("cleanup");
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn get_missing_event_tag_is_not_found() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    let resp = app
        .get("/api/v3/events-tags/999000111/999000222", &token)
        .await;
    assert_error(&resp, StatusCode::NOT_FOUND, "EVENT_TAG_NOT_FOUND_ERROR");
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn create_then_delete_event_tag_round_trips() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let monitor = insert_monitor(&app.db, "EvtTagRoundTrip")
        .await
        .expect("insert monitor");
    let event_id = insert_event(&app.db, monitor.id, "EvtTagRoundTripEvt").await;
    let tag_id = insert_tag(&app.db, "EvtTagRoundTripTag").await;

    let body = json!({ "event_id": event_id, "tag_id": tag_id });
    let create = app.post_json("/api/v3/events-tags", &token, &body).await;
    assert!(
        create.status().is_success(),
        "create should succeed; body: {}",
        create.text()
    );

    let delete = app
        .delete(&format!("/api/v3/events-tags/{tag_id}/{event_id}"), &token)
        .await;
    assert!(
        delete.status().is_success(),
        "delete should succeed; got {}",
        delete.status()
    );

    let get = app
        .get(&format!("/api/v3/events-tags/{tag_id}/{event_id}"), &token)
        .await;
    assert_eq!(get.status(), StatusCode::NOT_FOUND);

    delete_event(&app.db, event_id).await;
    delete_tag(&app.db, tag_id).await;
    delete_monitor(&app.db, monitor.id).await.expect("cleanup");
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn create_event_tag_with_invalid_body_is_rejected() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    // `event_id` must be a number, not a string.
    let resp = app
        .post_json(
            "/api/v3/events-tags",
            &token,
            &json!({ "event_id": "not-a-number", "tag_id": 1 }),
        )
        .await;
    assert!(
        resp.status().is_client_error(),
        "malformed create body should be a 4xx, got {}; body: {}",
        resp.status(),
        resp.text()
    );
}
