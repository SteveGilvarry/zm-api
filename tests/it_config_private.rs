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

/// An option zm.conf or conf.d sets reports the file and its value, a
/// private one never its value, and a PUT to it is a 409 (#183).
#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn options_set_in_a_config_file_say_so() {
    let dir = tempfile::tempdir().unwrap();
    let pid = std::process::id();
    let (plain, secret) = (
        format!("ZM_TEST_FILE_{pid}"),
        format!("ZM_TEST_FILE_SECRET_{pid}"),
    );
    std::fs::create_dir(dir.path().join("conf.d")).unwrap();
    std::fs::write(
        dir.path().join("conf.d/99-test.conf"),
        format!("{plain}=from-file\n{secret}=file-secret\n"),
    )
    .unwrap();
    // nextest runs each test in its own process, so this reaches only us.
    std::env::set_var("ZM_CONF_DIR", dir.path());

    let app = TestApp::spawn().await;
    let mut guards = Vec::new();
    for (name, private) in [(&plain, 0), (&secret, 1)] {
        zm_api::entity::config::ActiveModel {
            id: Set(0),
            name: Set(name.clone()),
            value: Set("from-db".into()),
            r#type: Set("string".into()),
            category: Set("hidden".into()),
            readonly: Set(0),
            private: Set(private),
            system: Set(1),
            requires: Set(Some("ZM_OPT_USE_AUTH=1".into())),
            ..Default::default()
        }
        .insert(&app.db)
        .await
        .unwrap();
        let n = name.clone();
        guards.push(RowGuard::new(
            format!("Config#{name}"),
            move |db| async move {
                use sea_orm::EntityTrait;
                let _ = zm_api::entity::config::Entity::delete_by_id(n)
                    .exec(&db)
                    .await;
            },
        ));
    }
    let admin = token_with(UserPermissions::superuser());

    let r: serde_json::Value = app
        .get(&format!("/api/v3/configs/{plain}"), &admin)
        .await
        .json();
    assert_eq!(r["source"], "file", "{r}");
    assert!(r["source_file"]
        .as_str()
        .unwrap()
        .ends_with("conf.d/99-test.conf"));
    assert_eq!(r["value"], "from-file", "the value in effect");
    assert_eq!(r["requires"], "ZM_OPT_USE_AUTH=1");

    let r: serde_json::Value = app
        .get(&format!("/api/v3/configs/{secret}"), &admin)
        .await
        .json();
    assert_eq!(
        r["value"], "",
        "a secret set in a file never leaves the server"
    );

    let put = app
        .request(axum::http::Method::PUT, &format!("/api/v3/configs/{plain}"))
        .bearer(&admin)
        .json(&serde_json::json!({ "value": "x" }))
        .send()
        .await;
    assert_eq!(put.status(), StatusCode::CONFLICT, "{}", put.text());
}
