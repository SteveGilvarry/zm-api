//! Integration tests for the User Preferences API — happy-path plus error paths.
//!
//! Requires the test database — run with:
//!   APP_PROFILE=test-db cargo test --test it_user_preferences -- --include-ignored

mod common;

use axum::http::StatusCode;
use common::assertions::{assert_error, assert_status};
use common::fixtures::{insert_user_with_id, unique_name};
use common::harness::{superuser_token, TestApp};
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use serde_json::json;
use zm_api::dto::response::{PaginatedUserPreferencesResponse, UserPreferenceResponse};

/// Insert a `User_Preferences` row directly and return its id.
async fn insert_preference(db: &sea_orm::DatabaseConnection, user_id: u32, label: &str) -> u32 {
    zm_api::entity::user_preferences::ActiveModel {
        user_id: Set(user_id),
        name: Set(Some(unique_name(label))),
        value: Set(Some("on".to_string())),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("insert user preference fixture")
    .id
}

async fn delete_preference(db: &sea_orm::DatabaseConnection, id: u32) {
    let _ = zm_api::entity::user_preferences::Entity::delete_by_id(id)
        .exec(db)
        .await;
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn list_user_preferences_returns_inserted_row() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let user_id = 990_100;
    insert_user_with_id(&app.db, user_id, "UpListUser")
        .await
        .expect("insert user");
    let id = insert_preference(&app.db, user_id, "UpList").await;

    let resp = app
        .get("/api/v3/user_preferences?page=1&page_size=1000", &token)
        .await;
    assert_status(&resp, StatusCode::OK);
    let body: PaginatedUserPreferencesResponse = resp.json();
    assert!(
        body.items.iter().any(|p| p.id == id),
        "user preferences list should contain the fixture row"
    );

    delete_preference(&app.db, id).await;
    let _ = zm_api::entity::users::Entity::delete_by_id(user_id)
        .exec(&app.db)
        .await;
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn get_user_preference_returns_the_row() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let user_id = 990_101;
    insert_user_with_id(&app.db, user_id, "UpGetUser")
        .await
        .expect("insert user");
    let id = insert_preference(&app.db, user_id, "UpGet").await;

    let resp = app
        .get(&format!("/api/v3/user_preferences/{id}"), &token)
        .await;
    assert_status(&resp, StatusCode::OK);
    let body: UserPreferenceResponse = resp.json();
    assert_eq!(body.id, id);
    assert_eq!(body.user_id, user_id);

    delete_preference(&app.db, id).await;
    let _ = zm_api::entity::users::Entity::delete_by_id(user_id)
        .exec(&app.db)
        .await;
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn get_missing_user_preference_is_not_found() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    let resp = app.get("/api/v3/user_preferences/999000111", &token).await;
    assert_error(&resp, StatusCode::NOT_FOUND, "MESSAGE_NOT_FOUND_ERROR");
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn create_then_delete_user_preference_round_trips() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let user_id = 990_102;
    insert_user_with_id(&app.db, user_id, "UpRoundTripUser")
        .await
        .expect("insert user");

    let body = json!({
        "user_id": user_id,
        "name": unique_name("UpRoundTrip"),
        "value": "off",
    });
    let create = app
        .post_json("/api/v3/user_preferences", &token, &body)
        .await;
    assert!(
        create.status().is_success(),
        "create should succeed; body: {}",
        create.text()
    );
    let created: UserPreferenceResponse = create.json();

    let delete = app
        .delete(&format!("/api/v3/user_preferences/{}", created.id), &token)
        .await;
    assert!(
        delete.status().is_success(),
        "delete should succeed; got {}",
        delete.status()
    );

    let get = app
        .get(&format!("/api/v3/user_preferences/{}", created.id), &token)
        .await;
    assert_eq!(get.status(), StatusCode::NOT_FOUND);

    let _ = zm_api::entity::users::Entity::delete_by_id(user_id)
        .exec(&app.db)
        .await;
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn create_user_preference_with_invalid_body_is_rejected() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    // Missing the required `user_id` field.
    let resp = app
        .post_json("/api/v3/user_preferences", &token, &json!({ "name": "x" }))
        .await;
    assert!(
        resp.status().is_client_error(),
        "malformed create body should be a 4xx, got {}; body: {}",
        resp.status(),
        resp.text()
    );
}
