// Integration tests with real database
// Run with: cargo test --test handlers_devices_integration -- --include-ignored

mod common;

use axum::http::{Request, StatusCode, header};
use axum::body::{self, Body};
use tower::ServiceExt;
use zm_api::entity::devices::Model as DeviceModel;
use zm_api::entity::sea_orm_active_enums::DeviceType;
use zm_api::dto::response::DeviceResponse;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use common::test_db::get_test_db;
use common::test_db::{cleanup_by_prefix, test_prefix};

fn auth_header() -> String {
    let token = zm_api::service::token::generate_tokens("tester".to_string())
        .expect("token")
        .access_token;
    format!("Bearer {}", token)
}

async fn create_device_db(db: &sea_orm::DatabaseConnection) -> Result<DeviceModel, sea_orm::DbErr> {
    use zm_api::entity::devices::ActiveModel;
    
    let device = ActiveModel {
        name: Set(format!("{}X10_Controller", test_prefix())),
        r#type: Set(DeviceType::X10),
        key_string: Set("A1".to_string()),
        ..Default::default()
    };
    
    device.insert(db).await
}

async fn cleanup_devices_db(db: &sea_orm::DatabaseConnection) -> Result<(), sea_orm::DbErr> {
    use zm_api::entity::devices::Entity as Device;
    
    let _ = Device::find().one(db).await?;
    cleanup_by_prefix(db, "Devices", "Name", &test_prefix()).await?;
    
    Ok(())
}

#[tokio::test]
#[ignore = "Requires running test database - run with: ./scripts/test-db.sh start"]
async fn test_list_devices_real_db() {
    let db = get_test_db().await.expect("Failed to connect to test database");
    
    // Setup: Create a test device
    let device = create_device_db(&db).await.expect("Failed to create test device");
    
    // Test: List devices
    let state = zm_api::server::state::AppState::for_test_with_db(db);
    let app = zm_api::routes::create_router_app(state);

    let response = app
        .oneshot(
            Request::get("/api/v3/devices")
                .header(header::AUTHORIZATION, auth_header())
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = body::to_bytes(response.into_body(), 64 * 1024).await.unwrap();
    let body: Vec<DeviceResponse> = serde_json::from_slice(&bytes).unwrap();
    
    // Verify our test device is in the list
    assert!(body.iter().any(|d| d.id == device.id));
    
    // Note: Cleanup is done in a separate connection since we consumed db
    let cleanup_db = get_test_db().await.expect("Failed to get cleanup connection");
    cleanup_devices_db(&cleanup_db).await.expect("Failed to cleanup");
}

#[tokio::test]
#[ignore = "Requires running test database - run with: ./scripts/test-db.sh start"]
async fn test_get_device_by_id_real_db() {
    let db = get_test_db().await.expect("Failed to connect to test database");
    
    // Setup: Create a test device
    let device = create_device_db(&db).await.expect("Failed to create test device");
    
    // Test: Get device by ID
    let state = zm_api::server::state::AppState::for_test_with_db(db);
    let app = zm_api::routes::create_router_app(state);

    let response = app
        .oneshot(
            Request::get(&format!("/api/v3/devices/{}", device.id))
                .header(header::AUTHORIZATION, auth_header())
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = body::to_bytes(response.into_body(), 64 * 1024).await.unwrap();
    let body: DeviceResponse = serde_json::from_slice(&bytes).unwrap();
    
    assert_eq!(body.id, device.id);
    assert_eq!(body.name, format!("{}X10_Controller", test_prefix()));
    assert_eq!(body.key_string, "A1");
    
    // Cleanup
    let cleanup_db = get_test_db().await.expect("Failed to get cleanup connection");
    cleanup_devices_db(&cleanup_db).await.expect("Failed to cleanup");
}

#[tokio::test]
#[ignore = "Requires running test database - run with: ./scripts/test-db.sh start"]
async fn test_create_and_delete_device_real_db() {
    let db = get_test_db().await.expect("Failed to connect to test database");
    
    // Create a device through the API
    // Note: You'll need to implement the POST endpoint for this test
    // For now, we'll create directly and test delete
    
    let device = create_device_db(&db).await.expect("Failed to create test device");
    let device_id = device.id;
    
    // Test: Delete device
    let state = zm_api::server::state::AppState::for_test_with_db(db);
    let app = zm_api::routes::create_router_app(state);

    let response = app
        .oneshot(
            Request::delete(&format!("/api/v3/devices/{}", device_id))
                .header(header::AUTHORIZATION, auth_header())
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    
    // Verify it's actually deleted
    use zm_api::entity::devices::{Entity as Device};
    let verify_db = get_test_db().await.expect("Failed to get verify connection");
    let found = Device::find_by_id(device_id).one(&verify_db).await.unwrap();
    assert!(found.is_none(), "Device should be deleted");
}
