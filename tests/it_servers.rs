//! Integration tests for the Servers API — happy-path plus error paths.
//!
//! Requires the test database — run with:
//!   APP_PROFILE=test-db cargo test --test it_servers -- --include-ignored

mod common;

use axum::http::StatusCode;
use common::assertions::{assert_error, assert_status};
use common::fixtures::unique_name;
use common::harness::{superuser_token, TestApp};
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use serde_json::json;
use zm_api::dto::response::{PaginatedServersResponse, ServerResponse};
use zm_api::entity::sea_orm_active_enums::Status;

/// Insert a `Servers` row directly and return its id.
async fn insert_server(db: &sea_orm::DatabaseConnection, label: &str) -> u32 {
    zm_api::entity::servers::ActiveModel {
        name: Set(unique_name(label)),
        status: Set(Status::Unknown),
        zmstats: Set(0),
        zmaudit: Set(0),
        zmtrigger: Set(0),
        zmeventnotification: Set(0),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("insert server fixture")
    .id
}

async fn delete_server(db: &sea_orm::DatabaseConnection, id: u32) {
    let _ = zm_api::entity::servers::Entity::delete_by_id(id)
        .exec(db)
        .await;
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn list_servers_returns_inserted_row() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let id = insert_server(&app.db, "SrvList").await;

    let resp = app
        .get("/api/v3/servers?page=1&page_size=1000", &token)
        .await;
    assert_status(&resp, StatusCode::OK);
    let body: PaginatedServersResponse = resp.json();
    assert!(
        body.items.iter().any(|s| s.id == id),
        "servers list should contain the fixture row"
    );

    delete_server(&app.db, id).await;
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn get_server_returns_the_row() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let id = insert_server(&app.db, "SrvGet").await;

    let resp = app.get(&format!("/api/v3/servers/{id}"), &token).await;
    assert_status(&resp, StatusCode::OK);
    let body: ServerResponse = resp.json();
    assert_eq!(body.id, id);

    delete_server(&app.db, id).await;
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn get_missing_server_is_not_found() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    let resp = app.get("/api/v3/servers/999000111", &token).await;
    assert_error(&resp, StatusCode::NOT_FOUND, "MESSAGE_NOT_FOUND_ERROR");
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn create_then_delete_server_round_trips() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    let body = json!({
        "name": unique_name("SrvRoundTrip"),
        "hostname": "127.0.0.1",
        "port": 9000,
        "status": "Unknown",
    });
    let create = app.post_json("/api/v3/servers", &token, &body).await;
    assert!(
        create.status().is_success(),
        "create should succeed; body: {}",
        create.text()
    );
    let created: ServerResponse = create.json();

    let delete = app
        .delete(&format!("/api/v3/servers/{}", created.id), &token)
        .await;
    assert!(
        delete.status().is_success(),
        "delete should succeed; got {}",
        delete.status()
    );

    let get = app
        .get(&format!("/api/v3/servers/{}", created.id), &token)
        .await;
    assert_eq!(get.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn create_server_with_invalid_body_is_rejected() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    // Missing the required `name` field.
    let resp = app
        .post_json("/api/v3/servers", &token, &json!({ "hostname": "x" }))
        .await;
    assert!(
        resp.status().is_client_error(),
        "malformed create body should be a 4xx, got {}; body: {}",
        resp.status(),
        resp.text()
    );
}
