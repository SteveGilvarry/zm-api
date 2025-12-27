// Integration tests with real database
// Run with: cargo test --test handlers_devices_integration -- --include-ignored

mod common;

use axum::http::{Request, StatusCode, header};
use axum::body::{self, Body};
use tower::ServiceExt;
use zm_api::entity::devices::Model as DeviceModel;
use zm_api::entity::sea_orm_active_enums::DeviceType;
use zm_api::dto::response::DeviceResponse;
use sea_orm::{EntityTrait, ActiveModelTrait, Set};
use common::test_db::get_test_db;

fn auth_header() -> String {
    let token = zm_api::service::token::generate_tokens("tester".to_string())
        .expect("token")
        .access_token;
    format!("Bearer {}", token)
}

async fn create_test_device(db: &sea_orm::DatabaseConnection) -> Result<DeviceModel, sea_orm::DbErr> {
    use zm_api::entity::devices::{self, ActiveModel};
    
    let device = ActiveModel {
        name: Set("Test X10 Controller".to_string()),
        r#type: Set(DeviceType::X10),
        key_string: Set("A1".to_string()),
        ..Default::default()
    };
    
    device.insert(db).await
}

async fn cleanup_test_devices(db: &sea_orm::DatabaseConnection) -> Result<(), sea_orm::DbErr> {
    use zm_api::entity::devices::{self, Entity as Device};
    use sea_orm::QueryFilter;
    use sea_orm::sea_query::Expr;
    use sea_orm::ColumnTrait;
    
    // Delete test devices (ones created by tests)
    Device::delete_many()
        .filter(devices::Column::Name.like("%Test%"))
        .exec(db)
        .await?;
    
    Ok(())
}

#[tokio::test]
#[ignore = "Requires running test database - run with: ./scripts/test-db.sh start"]
async fn test_list_devices_real_db() {
    let db = get_test_db().await.expect("Failed to connect to test database");
    
    // Setup: Create a test device
    let device = create_test_device(&db).await.expect("Failed to create test device");
    
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
    cleanup_test_devices(&cleanup_db).await.expect("Failed to cleanup");
}

#[tokio::test]
#[ignore = "Requires running test database - run with: ./scripts/test-db.sh start"]
async fn test_get_device_by_id_real_db() {
    let db = get_test_db().await.expect("Failed to connect to test database");
    
    // Setup: Create a test device
    let device = create_test_device(&db).await.expect("Failed to create test device");
    
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
    assert_eq!(body.name, "Test X10 Controller");
    assert_eq!(body.key_string, "A1");
    
    // Cleanup
    let cleanup_db = get_test_db().await.expect("Failed to get cleanup connection");
    cleanup_test_devices(&cleanup_db).await.expect("Failed to cleanup");
}

#[tokio::test]
#[ignore = "Requires running test database - run with: ./scripts/test-db.sh start"]
async fn test_create_and_delete_device_real_db() {
    let db = get_test_db().await.expect("Failed to connect to test database");
    
    // Create a device through the API
    // Note: You'll need to implement the POST endpoint for this test
    // For now, we'll create directly and test delete
    
    let device = create_test_device(&db).await.expect("Failed to create test device");
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
