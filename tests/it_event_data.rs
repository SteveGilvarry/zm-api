//! Integration tests for the Event Data API — happy-path plus error paths.
//!
//! Requires the test database — run with:
//!   APP_PROFILE=test-db cargo test --test it_event_data -- --include-ignored

mod common;

use axum::http::StatusCode;
use common::assertions::{assert_error, assert_status};
use common::fixtures::{delete_monitor, insert_monitor, unique_name};
use common::harness::{superuser_token, TestApp};
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use serde_json::json;
use zm_api::dto::response::{EventDataResponse, PaginatedEventDataResponse};

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

/// Insert an `Event_Data` row directly and return its id.
async fn insert_event_data(
    db: &sea_orm::DatabaseConnection,
    event_id: u64,
    monitor_id: u32,
) -> u64 {
    zm_api::entity::event_data::ActiveModel {
        event_id: Set(Some(event_id)),
        monitor_id: Set(Some(monitor_id)),
        data: Set(Some("{\"detections\":[]}".to_string())),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("insert event_data fixture")
    .id
}

async fn delete_event_data_row(db: &sea_orm::DatabaseConnection, id: u64) {
    let _ = zm_api::entity::event_data::Entity::delete_by_id(id)
        .exec(db)
        .await;
}

async fn delete_event(db: &sea_orm::DatabaseConnection, id: u64) {
    let _ = zm_api::entity::events::Entity::delete_by_id(id)
        .exec(db)
        .await;
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn list_event_data_returns_inserted_row() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let monitor = insert_monitor(&app.db, "EvtDataList")
        .await
        .expect("insert monitor");
    let event_id = insert_event(&app.db, monitor.id, "EvtDataListEvt").await;
    let data_id = insert_event_data(&app.db, event_id, monitor.id).await;

    // `EventDataQuery` flattens `PaginationParams`; the handler uses
    // `axum_extra`'s Query so numeric/flattened query params deserialize
    // correctly. A large page size guarantees the fixture lands on the page.
    let resp = app
        .get(
            &format!("/api/v3/event-data?event_id={event_id}&page=1&page_size=1000"),
            &token,
        )
        .await;
    assert_status(&resp, StatusCode::OK);
    let body: PaginatedEventDataResponse = resp.json();
    assert!(
        body.items.iter().any(|d| d.id == data_id),
        "list should contain the fixture row"
    );

    delete_event_data_row(&app.db, data_id).await;
    delete_event(&app.db, event_id).await;
    delete_monitor(&app.db, monitor.id).await.expect("cleanup");
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn get_event_data_returns_the_row() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let monitor = insert_monitor(&app.db, "EvtDataGet")
        .await
        .expect("insert monitor");
    let event_id = insert_event(&app.db, monitor.id, "EvtDataGetEvt").await;
    let data_id = insert_event_data(&app.db, event_id, monitor.id).await;

    let resp = app
        .get(&format!("/api/v3/event-data/{data_id}"), &token)
        .await;
    assert_status(&resp, StatusCode::OK);
    let body: EventDataResponse = resp.json();
    assert_eq!(body.id, data_id);
    assert_eq!(body.event_id, Some(event_id));

    delete_event_data_row(&app.db, data_id).await;
    delete_event(&app.db, event_id).await;
    delete_monitor(&app.db, monitor.id).await.expect("cleanup");
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn get_missing_event_data_is_not_found() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    let resp = app.get("/api/v3/event-data/999000111", &token).await;
    assert_error(&resp, StatusCode::NOT_FOUND, "FILE_NOT_FOUND_ERROR");
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn create_then_delete_event_data_round_trips() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let monitor = insert_monitor(&app.db, "EvtDataRoundTrip")
        .await
        .expect("insert monitor");
    let event_id = insert_event(&app.db, monitor.id, "EvtDataRoundTripEvt").await;

    let body = json!({
        "event_id": event_id,
        "monitor_id": monitor.id,
        "data": "{\"detections\":[1]}",
    });
    let create = app.post_json("/api/v3/event-data", &token, &body).await;
    assert!(
        create.status().is_success(),
        "create should succeed; body: {}",
        create.text()
    );
    let created: EventDataResponse = create.json();

    let delete = app
        .delete(&format!("/api/v3/event-data/{}", created.id), &token)
        .await;
    assert!(
        delete.status().is_success(),
        "delete should succeed; got {}",
        delete.status()
    );

    let get = app
        .get(&format!("/api/v3/event-data/{}", created.id), &token)
        .await;
    assert_eq!(get.status(), StatusCode::NOT_FOUND);

    delete_event(&app.db, event_id).await;
    delete_monitor(&app.db, monitor.id).await.expect("cleanup");
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn create_event_data_with_invalid_body_is_rejected() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    // `event_id` must be a number, not a string.
    let resp = app
        .post_json(
            "/api/v3/event-data",
            &token,
            &json!({ "event_id": "not-a-number" }),
        )
        .await;
    assert!(
        resp.status().is_client_error(),
        "malformed create body should be a 4xx, got {}; body: {}",
        resp.status(),
        resp.text()
    );
}
