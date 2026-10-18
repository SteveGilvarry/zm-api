//! `Private` config values are for System editors only (#184).
//!
//! Requires the test database:
//!   APP_PROFILE=test-db cargo nextest run --test it_config_private --run-ignored all

mod common;

use axum::http::StatusCode;
use common::fixtures::RowGuard;
use common::harness::{token_with, TestApp};
use sea_orm::{ActiveModelTrait, Set};
use zm_api::util::authz::{Level, UserPermissions};

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn private_values_are_blank_below_system_edit() {
    let app = TestApp::spawn().await;
    let name = format!("ZM_TEST_PRIVATE_{}", std::process::id());
    zm_api::entity::config::ActiveModel {
        id: Set(0),
        name: Set(name.clone()),
        value: Set("s3cret-signing-key".into()),
        r#type: Set("string".into()),
        category: Set("hidden".into()),
        readonly: Set(0),
        private: Set(1),
        system: Set(1),
        ..Default::default()
    }
    .insert(&app.db)
    .await
    .expect("private config fixture");
    let n = name.clone();
    let _g = RowGuard::new(format!("Config#{name}"), move |db| async move {
        use sea_orm::EntityTrait;
        let _ = zm_api::entity::config::Entity::delete_by_id(n)
            .exec(&db)
            .await;
    });

    let viewer = token_with(UserPermissions {
        system: Level::View,
        ..Default::default()
    });
    let editor = token_with(UserPermissions {
        system: Level::Edit,
        ..Default::default()
    });

    let path = format!("/api/v3/configs/{name}");
    let r = app.get(&path, &viewer).await;
    assert_eq!(r.status(), StatusCode::OK, "{}", r.text());
    assert_eq!(
        r.json::<serde_json::Value>()["value"],
        "",
        "System=View sees a blank"
    );

    let list = app
        .get(
            &format!("/api/v3/configs?page_size=1000&search={name}"),
            &viewer,
        )
        .await;
    assert!(
        list.text().contains(&name),
        "the row is listed: {}",
        list.text()
    );
    assert!(
        !list.text().contains("s3cret-signing-key"),
        "but not its value"
    );

    let r = app.get(&path, &editor).await;
    assert_eq!(r.json::<serde_json::Value>()["value"], "s3cret-signing-key");
}
