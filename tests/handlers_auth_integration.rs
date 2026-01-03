// Integration tests for auth endpoints with a real database
// Run with: cargo test --test handlers_auth_integration -- --include-ignored

mod common;

use axum::body::{self, Body};
use axum::http::{header, Request, StatusCode};
use common::test_db::{cleanup_by_prefix, get_test_db, test_prefix};
use sea_orm::{ActiveModelTrait, Set};
use tower::ServiceExt;
use zm_api::dto::request::{LoginRequest, RefreshTokenRequest};
use zm_api::dto::response::{MessageResponse, TokenResponse};

async fn create_user_db(
    db: &sea_orm::DatabaseConnection,
    username: &str,
    password: &str,
) -> Result<(), sea_orm::DbErr> {
    use zm_api::entity::users::ActiveModel;

    let hashed = zm_api::util::password::hash(password.to_string())
        .await
        .expect("hash password");
    let user = ActiveModel {
        username: Set(username.to_string()),
        password: Set(hashed),
        enabled: Set(1),
        ..Default::default()
    };
    user.insert(db).await?;
    Ok(())
}

async fn cleanup_users_db(db: &sea_orm::DatabaseConnection) -> Result<(), sea_orm::DbErr> {
    cleanup_by_prefix(db, "Users", "Username", &test_prefix()).await?;
    Ok(())
}

#[tokio::test]
#[ignore = "Requires running test database - run with: ./scripts/db-manager.sh mysql"]
async fn test_api_auth_login_refresh_logout() {
    let username = format!("{}auth_user", test_prefix());
    let password = "TestPass123!";

    let setup_db = get_test_db()
        .await
        .expect("Failed to connect to test database");
    create_user_db(&setup_db, &username, password)
        .await
        .expect("Failed to create test user");

    let state = zm_api::server::state::AppState::for_test_with_db(setup_db);
    let app = zm_api::routes::create_router_app(state);

    let login_body = serde_json::to_vec(&LoginRequest {
        username: username.clone(),
        password: password.to_string(),
    })
    .expect("serialize login");

    let response = app
        .clone()
        .oneshot(
            Request::post("/api/v3/auth/login")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(login_body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = body::to_bytes(response.into_body(), 64 * 1024)
        .await
        .unwrap();
    let token: TokenResponse = serde_json::from_slice(&bytes).unwrap();
    assert!(!token.access_token.is_empty());
    assert!(!token.refresh_token.is_empty());

    let refresh_body = serde_json::to_vec(&RefreshTokenRequest {
        token: token.refresh_token.clone(),
    })
    .expect("serialize refresh");

    let response = app
        .clone()
        .oneshot(
            Request::post("/api/v3/auth/refresh")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(refresh_body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = body::to_bytes(response.into_body(), 64 * 1024)
        .await
        .unwrap();
    let refreshed: TokenResponse = serde_json::from_slice(&bytes).unwrap();
    assert!(!refreshed.access_token.is_empty());

    let response = app
        .oneshot(
            Request::get("/api/v3/auth/logout")
                .header(
                    header::AUTHORIZATION,
                    format!("Bearer {}", token.access_token),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = body::to_bytes(response.into_body(), 64 * 1024)
        .await
        .unwrap();
    let body: MessageResponse = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body.message(), "Logout successful");

    let cleanup_db = get_test_db()
        .await
        .expect("Failed to get cleanup connection");
    cleanup_users_db(&cleanup_db)
        .await
        .expect("Failed to cleanup users");
}
