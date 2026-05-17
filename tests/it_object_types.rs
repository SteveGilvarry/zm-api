//! Integration tests for the Object Types API — happy-path plus error paths.
//!
//! Requires the test database — run with:
//!   APP_PROFILE=test-db cargo test --test it_object_types -- --include-ignored

mod common;

use axum::http::StatusCode;
use common::assertions::{assert_error, assert_status};
use common::fixtures::unique_name;
use common::harness::{superuser_token, TestApp};
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use serde_json::json;
use zm_api::dto::response::{ObjectTypeResponse, PaginatedObjectTypesResponse};

/// A unique fixture name that fits `Object_Types.Name` (`varchar(32)`).
fn ot_name(label: &str) -> String {
    unique_name(label).chars().take(32).collect()
}

/// Insert an `Object_Types` row directly and return its id.
async fn insert_object_type(db: &sea_orm::DatabaseConnection, label: &str) -> i32 {
    zm_api::entity::object_types::ActiveModel {
        name: Set(Some(ot_name(label))),
        human: Set(Some(unique_name(label))),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("insert object type fixture")
    .id
}

async fn delete_object_type(db: &sea_orm::DatabaseConnection, id: i32) {
    let _ = zm_api::entity::object_types::Entity::delete_by_id(id)
        .exec(db)
        .await;
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn list_object_types_returns_inserted_row() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let id = insert_object_type(&app.db, "OtList").await;

    let resp = app
        .get("/api/v3/object-types?page=1&page_size=1000", &token)
        .await;
    assert_status(&resp, StatusCode::OK);
    let body: PaginatedObjectTypesResponse = resp.json();
    assert!(
        body.items.iter().any(|o| o.id == id),
        "object types list should contain the fixture row"
    );

    delete_object_type(&app.db, id).await;
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn get_object_type_returns_the_row() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let id = insert_object_type(&app.db, "OtGet").await;

    let resp = app.get(&format!("/api/v3/object-types/{id}"), &token).await;
    assert_status(&resp, StatusCode::OK);
    let body: ObjectTypeResponse = resp.json();
    assert_eq!(body.id, id);

    delete_object_type(&app.db, id).await;
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn get_missing_object_type_is_not_found() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    let resp = app.get("/api/v3/object-types/999000111", &token).await;
    assert_error(&resp, StatusCode::NOT_FOUND, "FILE_NOT_FOUND_ERROR");
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn create_then_delete_object_type_round_trips() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    let body = json!({
        "name": ot_name("OtRoundTrip"),
        "human": "Round Trip",
    });
    let create = app.post_json("/api/v3/object-types", &token, &body).await;
    assert!(
        create.status().is_success(),
        "create should succeed; body: {}",
        create.text()
    );
    let created: ObjectTypeResponse = create.json();

    let delete = app
        .delete(&format!("/api/v3/object-types/{}", created.id), &token)
        .await;
    assert!(
        delete.status().is_success(),
        "delete should succeed; got {}",
        delete.status()
    );

    let get = app
        .get(&format!("/api/v3/object-types/{}", created.id), &token)
        .await;
    assert_eq!(get.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn create_object_type_with_invalid_body_is_rejected() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    // `name` and `human` are both optional, so the only way to force a
    // 4xx is to send a body of the wrong JSON shape.
    let resp = app
        .post_json("/api/v3/object-types", &token, &json!({ "name": 12345 }))
        .await;
    assert!(
        resp.status().is_client_error(),
        "malformed create body should be a 4xx, got {}; body: {}",
        resp.status(),
        resp.text()
    );
}
