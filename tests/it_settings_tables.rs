//! EncoderTemplates and Menu_Items (#144).
//!
//! Requires the test database:
//!   APP_PROFILE=test-db cargo nextest run --test it_settings_tables --run-ignored all

mod common;

use axum::http::StatusCode;
use common::fixtures::{unique_name, RowGuard};
use common::harness::{superuser_token, token_with, TestApp};
use serde_json::json;
use zm_api::dto::response::settings_tables::{EncoderTemplateResponse, MenuItemResponse};
use zm_api::util::authz::UserPermissions;

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn encoder_templates_round_trip_and_the_seeds_are_listed() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let list: Vec<EncoderTemplateResponse> =
        app.get("/api/v3/encoder-templates", &token).await.json();
    assert!(
        list.iter().any(|t| t.encoder == "libx264"),
        "seeded templates listed"
    );

    let resp = app
        .post_json(
            "/api/v3/encoder-templates",
            &token,
            &json!({ "encoder": "libx265", "name": unique_name("Tmpl"), "params": "crf=28" }),
        )
        .await;
    assert_eq!(resp.status(), StatusCode::CREATED, "{}", resp.text());
    let t: EncoderTemplateResponse = resp.json();
    let id = t.id;
    let _g = RowGuard::new(format!("EncoderTemplates#{id}"), move |db| async move {
        use sea_orm::EntityTrait;
        let _ = zm_api::entity::encoder_templates::Entity::delete_by_id(id)
            .exec(&db)
            .await;
    });
    let resp = app
        .put_json(
            &format!("/api/v3/encoder-templates/{id}"),
            &token,
            &json!({ "params": "crf=30" }),
        )
        .await;
    assert_eq!(resp.json::<EncoderTemplateResponse>().params, "crf=30");
    let del = app
        .delete(&format!("/api/v3/encoder-templates/{id}"), &token)
        .await;
    assert_eq!(del.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn the_navbar_shows_enabled_items_in_order_to_any_user() {
    let app = TestApp::spawn().await;
    let admin = superuser_token();
    let key = unique_name("Nav");
    let key = &key[key.len().saturating_sub(32)..];
    let resp = app
        .post_json(
            "/api/v3/menu-items",
            &admin,
            &json!({ "menu_key": key, "enabled": false, "sort_order": -100, "label": "Hidden" }),
        )
        .await;
    assert_eq!(resp.status(), StatusCode::CREATED, "{}", resp.text());
    let item: MenuItemResponse = resp.json();
    let id = item.id;
    let _g = RowGuard::new(format!("Menu_Items#{id}"), move |db| async move {
        use sea_orm::EntityTrait;
        let _ = zm_api::entity::menu_items::Entity::delete_by_id(id)
            .exec(&db)
            .await;
    });

    // A user with no permissions at all still gets the navbar.
    let nobody = token_with(UserPermissions::default());
    let nav = app.get("/api/v3/system/menu", &nobody).await;
    assert_eq!(nav.status(), StatusCode::OK, "{}", nav.text());
    let nav: Vec<MenuItemResponse> = nav.json();
    assert!(
        !nav.iter().any(|m| m.id == id),
        "disabled items are left out"
    );
    assert!(nav.windows(2).all(|w| w[0].sort_order <= w[1].sort_order));
    // ...but not the admin CRUD.
    assert_eq!(
        app.get("/api/v3/menu-items", &nobody).await.status(),
        StatusCode::FORBIDDEN
    );

    let resp = app
        .put_json(
            &format!("/api/v3/menu-items/{id}"),
            &admin,
            &json!({ "enabled": true, "label": null }),
        )
        .await;
    let updated: MenuItemResponse = resp.json();
    assert!(updated.enabled && updated.label.is_none());
    let nav: Vec<MenuItemResponse> = app.get("/api/v3/system/menu", &nobody).await.json();
    assert_eq!(
        nav.first().map(|m| m.id),
        Some(id),
        "sort_order -100 comes first"
    );

    let bad = app
        .put_json(
            &format!("/api/v3/menu-items/{id}"),
            &admin,
            &json!({ "icon_type": "emoji" }),
        )
        .await;
    assert!(bad.status().is_client_error());
}

/// Each storage reports the filesystem its path is on (#145), and null when
/// the path can't be read.
#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn storage_reports_its_filesystem() {
    use common::fixtures::insert_storage;
    use zm_api::entity::sea_orm_active_enums::Scheme;
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let dir = tempfile::tempdir().unwrap();
    let real = insert_storage(
        &app.db,
        "FsReal",
        dir.path().to_str().unwrap(),
        Scheme::Medium,
    )
    .await
    .unwrap();
    let _r = RowGuard::storage(real.id);
    let gone = insert_storage(&app.db, "FsGone", "/no/such/zm/storage", Scheme::Medium)
        .await
        .unwrap();
    let _g = RowGuard::storage(gone.id);

    let body: serde_json::Value = app
        .get(&format!("/api/v3/storage/{}", real.id), &token)
        .await
        .json();
    let fs = &body["filesystem"];
    assert!(fs["total_bytes"].as_u64().unwrap() > 0, "{body}");
    assert!(fs["used_bytes"].as_u64().unwrap() <= fs["total_bytes"].as_u64().unwrap());

    let body: serde_json::Value = app
        .get(&format!("/api/v3/storage/{}", gone.id), &token)
        .await
        .json();
    assert!(body["filesystem"].is_null(), "{body}");
}

/// `PATCH /storage/{id}` changes `do_delete` (#185).
#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn storage_do_delete_can_be_changed() {
    use common::fixtures::insert_storage;
    use zm_api::entity::sea_orm_active_enums::Scheme;
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let s = insert_storage(&app.db, "DoDel", "/tmp", Scheme::Medium)
        .await
        .unwrap();
    let _g = RowGuard::storage(s.id);
    let r = app
        .patch_json(
            &format!("/api/v3/storage/{}", s.id),
            &token,
            &json!({ "do_delete": 0 }),
        )
        .await;
    assert_eq!(r.status(), StatusCode::OK, "{}", r.text());
    assert_eq!(r.json::<serde_json::Value>()["do_delete"], 0);
    let bad = app
        .patch_json(
            &format!("/api/v3/storage/{}", s.id),
            &token,
            &json!({ "do_delete": 7 }),
        )
        .await;
    assert!(bad.status().is_client_error());
}

/// `POST /system/privacy` records the answer as legacy's privacy page does
/// (#189), and needs System edit.
#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn the_privacy_answer_is_recorded() {
    use sea_orm::{ActiveModelTrait, EntityTrait, Set};
    let app = TestApp::spawn().await;
    // The test database may not carry these Config rows; add any missing.
    let mut added = Vec::new();
    for (name, value) in [("ZM_SHOW_PRIVACY", "1"), ("ZM_TELEMETRY_DATA", "0")] {
        if zm_api::entity::config::Entity::find_by_id(name.to_string())
            .one(&app.db)
            .await
            .unwrap()
            .is_none()
        {
            zm_api::entity::config::ActiveModel {
                id: Set(0),
                name: Set(name.into()),
                value: Set(value.into()),
                r#type: Set("boolean".into()),
                category: Set("system".into()),
                readonly: Set(1),
                private: Set(0),
                system: Set(1),
                ..Default::default()
            }
            .insert(&app.db)
            .await
            .unwrap();
            added.push(name.to_string());
        }
    }
    let _g = RowGuard::new("privacy config rows", move |db| async move {
        for name in added {
            let _ = zm_api::entity::config::Entity::delete_by_id(name)
                .exec(&db)
                .await;
        }
    });

    let viewer = token_with(UserPermissions {
        system: zm_api::util::authz::Level::View,
        ..Default::default()
    });
    let r = app
        .post_json(
            "/api/v3/system/privacy",
            &viewer,
            &json!({ "accept": true }),
        )
        .await;
    assert_eq!(r.status(), StatusCode::FORBIDDEN);

    let r = app
        .post_json(
            "/api/v3/system/privacy",
            &superuser_token(),
            &json!({ "accept": true }),
        )
        .await;
    assert_eq!(r.status(), StatusCode::OK, "{}", r.text());
    let body: serde_json::Value = r.json();
    assert_eq!(body["show_privacy"], false);
    assert_eq!(body["telemetry"], true);
    let r = app
        .post_json(
            "/api/v3/system/privacy",
            &superuser_token(),
            &json!({ "accept": false }),
        )
        .await;
    assert_eq!(r.json::<serde_json::Value>()["telemetry"], false);
}
