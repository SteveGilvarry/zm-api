// Integration tests for filters, zones, snapshots, and streaming auth gates
// Run with: cargo test --test handlers_filters_zones_snapshots_streaming_integration -- --include-ignored

mod common;

use axum::body::{self, Body};
use axum::http::{header, Request, StatusCode};
use common::test_db::{get_test_db, test_prefix};
use sea_orm::{ActiveModelTrait, DatabaseConnection, DbErr, EntityTrait, Set};
use tower::ServiceExt;
use zm_api::dto::request::filters::CreateFilterRequest;
use zm_api::dto::request::snapshots::CreateSnapshotRequest;
use zm_api::dto::request::zones::CreateZoneRequest;
use zm_api::dto::response::{FilterResponse, SnapshotResponse, ZoneResponse};
use zm_api::entity::monitors;

fn auth_header() -> String {
    let token = zm_api::service::token::generate_tokens("tester".to_string())
        .expect("token")
        .access_token;
    format!("Bearer {}", token)
}

fn build_app(db: DatabaseConnection) -> axum::Router {
    let state = zm_api::server::state::AppState::for_test_with_db(db);
    zm_api::routes::create_router_app(state)
}

async fn create_monitor_db(db: &DatabaseConnection) -> Result<monitors::Model, DbErr> {
    let name = format!("{}zone_monitor", test_prefix());
    let model = monitors::ActiveModel {
        name: Set(name),
        ..Default::default()
    };
    model.insert(db).await
}

async fn cleanup_monitor_db(db: &DatabaseConnection, id: u32) -> Result<(), DbErr> {
    monitors::Entity::delete_by_id(id).exec(db).await?;
    Ok(())
}

#[tokio::test]
#[ignore = "Requires running test database - run with: ./scripts/db-manager.sh mysql"]
async fn test_api_filters_create_get_delete() {
    let db = get_test_db().await.expect("Failed to connect to test database");
    let app = build_app(db);

    let name = format!("{}filter", test_prefix());
    let create_body = serde_json::to_vec(&CreateFilterRequest {
        name: name.clone(),
        query_json: "{\"filter\":\"all\"}".to_string(),
        user_id: None,
        execute_interval: None,
        email_format: None,
    })
    .expect("serialize filter");

    let response = app
        .clone()
        .oneshot(
            Request::post("/api/v3/filters")
                .header(header::AUTHORIZATION, auth_header())
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(create_body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let bytes = body::to_bytes(response.into_body(), 64 * 1024).await.unwrap();
    let created: FilterResponse = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(created.name, name);

    let response = app
        .clone()
        .oneshot(
            Request::get(&format!("/api/v3/filters/{}", created.id))
                .header(header::AUTHORIZATION, auth_header())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let response = app
        .oneshot(
            Request::delete(&format!("/api/v3/filters/{}", created.id))
                .header(header::AUTHORIZATION, auth_header())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
#[ignore = "Requires running test database - run with: ./scripts/db-manager.sh mysql"]
async fn test_api_zones_create_get_delete() {
    let db = get_test_db().await.expect("Failed to connect to test database");
    let monitor = create_monitor_db(&db).await.expect("Failed to create monitor");
    let app = build_app(db);

    let name = format!("{}zone", test_prefix());
    let create_body = serde_json::to_vec(&CreateZoneRequest {
        name: name.clone(),
        r#type: "active".to_string(),
        units: "pixels".to_string(),
        coords: "0,0 1,1 2,2 3,3".to_string(),
        num_coords: 4,
        check_method: None,
    })
    .expect("serialize zone");

    let response = app
        .clone()
        .oneshot(
            Request::post(&format!("/api/v3/monitors/{}/zones", monitor.id))
                .header(header::AUTHORIZATION, auth_header())
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(create_body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let bytes = body::to_bytes(response.into_body(), 64 * 1024).await.unwrap();
    let created: ZoneResponse = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(created.name, name);

    let response = app
        .clone()
        .oneshot(
            Request::get(&format!("/api/v3/zones/{}", created.id))
                .header(header::AUTHORIZATION, auth_header())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let response = app
        .oneshot(
            Request::delete(&format!("/api/v3/zones/{}", created.id))
                .header(header::AUTHORIZATION, auth_header())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    let cleanup_db = get_test_db().await.expect("Failed to get cleanup connection");
    cleanup_monitor_db(&cleanup_db, monitor.id)
        .await
        .expect("Failed to cleanup monitor");
}

#[tokio::test]
#[ignore = "Requires running test database - run with: ./scripts/db-manager.sh mysql"]
async fn test_api_snapshots_create_get_delete() {
    let db = get_test_db().await.expect("Failed to connect to test database");
    let app = build_app(db);

    let name = format!("{}snapshot", test_prefix());
    let create_body = serde_json::to_vec(&CreateSnapshotRequest {
        name: Some(name.clone()),
        description: Some("test snapshot".to_string()),
        created_by: None,
        created_on: None,
    })
    .expect("serialize snapshot");

    let response = app
        .clone()
        .oneshot(
            Request::post("/api/v3/snapshots")
                .header(header::AUTHORIZATION, auth_header())
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(create_body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let bytes = body::to_bytes(response.into_body(), 64 * 1024).await.unwrap();
    let created: SnapshotResponse = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(created.name.as_deref(), Some(name.as_str()));

    let response = app
        .clone()
        .oneshot(
            Request::get(&format!("/api/v3/snapshots/{}", created.id))
                .header(header::AUTHORIZATION, auth_header())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let response = app
        .oneshot(
            Request::delete(&format!("/api/v3/snapshots/{}", created.id))
                .header(header::AUTHORIZATION, auth_header())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
#[ignore = "Requires running test database - run with: ./scripts/db-manager.sh mysql"]
async fn test_api_streaming_routes_require_auth() {
    let db = get_test_db().await.expect("Failed to connect to test database");
    let app = build_app(db);

    let response = app
        .clone()
        .oneshot(
            Request::get("/api/v3/streams/1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let response = app
        .clone()
        .oneshot(
            Request::get("/api/v3/mse/streams")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let response = app
        .oneshot(
            Request::get("/api/v3/webrtc/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
