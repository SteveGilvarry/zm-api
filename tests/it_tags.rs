//! Integration tests for the Tags API — happy-path plus error paths.
//!
//! Requires the test database — run with:
//!   APP_PROFILE=test-db cargo test --test it_tags -- --include-ignored

mod common;

use axum::http::StatusCode;
use common::assertions::{assert_error, assert_status};
use common::fixtures::unique_name;
use common::harness::{superuser_token, TestApp};
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use serde_json::json;
use zm_api::dto::response::{PaginatedTagsResponse, TagResponse};

/// Insert a `Tags` row directly and return its id.
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

async fn delete_tag(db: &sea_orm::DatabaseConnection, id: u64) {
    let _ = zm_api::entity::tags::Entity::delete_by_id(id)
        .exec(db)
        .await;
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn list_tags_returns_inserted_row() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let id = insert_tag(&app.db, "TagList").await;

    let resp = app.get("/api/v3/tags?page=1&page_size=1000", &token).await;
    assert_status(&resp, StatusCode::OK);
    let body: PaginatedTagsResponse = resp.json();
    assert!(
        body.items.iter().any(|t| t.id == id),
        "list should contain the fixture tag"
    );

    delete_tag(&app.db, id).await;
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn get_tag_returns_the_row() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let id = insert_tag(&app.db, "TagGet").await;

    let resp = app.get(&format!("/api/v3/tags/{id}"), &token).await;
    assert_status(&resp, StatusCode::OK);
    let body: TagResponse = resp.json();
    assert_eq!(body.id, id);

    delete_tag(&app.db, id).await;
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn get_missing_tag_is_not_found() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    let resp = app.get("/api/v3/tags/999000111", &token).await;
    assert_error(&resp, StatusCode::NOT_FOUND, "MESSAGE_NOT_FOUND_ERROR");
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn create_then_delete_tag_round_trips() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    let body = json!({ "name": unique_name("TagRoundTrip") });
    let create = app.post_json("/api/v3/tags", &token, &body).await;
    assert!(
        create.status().is_success(),
        "create should succeed; body: {}",
        create.text()
    );
    let created: TagResponse = create.json();

    let delete = app
        .delete(&format!("/api/v3/tags/{}", created.id), &token)
        .await;
    assert!(
        delete.status().is_success(),
        "delete should succeed; got {}",
        delete.status()
    );

    let get = app
        .get(&format!("/api/v3/tags/{}", created.id), &token)
        .await;
    assert_eq!(get.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn create_tag_with_invalid_body_is_rejected() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    // Missing the required `name` field.
    let resp = app.post_json("/api/v3/tags", &token, &json!({})).await;
    assert!(
        resp.status().is_client_error(),
        "malformed create body should be a 4xx, got {}; body: {}",
        resp.status(),
        resp.text()
    );
}
