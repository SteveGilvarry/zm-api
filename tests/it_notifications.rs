//! Push token registrations (#175): each user sees and edits their own, a
//! POST with a known token updates it, and a System editor sees everyone's.
//!
//! Requires the test database:
//!   APP_PROFILE=test-db cargo nextest run --test it_notifications --run-ignored all

mod common;

use axum::http::{Method, StatusCode};
use common::fixtures::{insert_user_with_id, RowGuard};
use common::harness::{token_for, TestApp};
use serde_json::json;
use zm_api::util::authz::UserPermissions;

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn users_manage_their_own_push_tokens() {
    let app = TestApp::spawn().await;
    // Per-process ids: nextest runs each test in its own process.
    let alice = 970_000 + std::process::id() % 10_000;
    let bob = alice + 10_000;
    for uid in [alice, bob] {
        insert_user_with_id(&app.db, uid, &format!("push{uid}"))
            .await
            .unwrap();
    }
    // The foreign key cascades: deleting the users drops their rows.
    let (_a, _b) = (RowGuard::user(alice), RowGuard::user(bob));
    let (ta, tb) = (
        token_for(alice, UserPermissions::default()),
        token_for(bob, UserPermissions::default()),
    );
    let token = format!("fcm-{alice}");

    let r = app
        .post_json(
            "/api/v3/notifications",
            &ta,
            &json!({ "token": token, "platform": "android", "monitor_list": "1,2",
                     "user_id": bob }),
        )
        .await;
    assert_eq!(r.status(), StatusCode::CREATED, "{}", r.text());
    let row: serde_json::Value = r.json();
    assert_eq!(row["user_id"], alice, "only an admin picks the owner");
    assert_eq!(row["push_state"], "enabled");
    let id = row["id"].as_u64().unwrap();

    // The same token again updates the row.
    let r = app
        .post_json(
            "/api/v3/notifications",
            &ta,
            &json!({ "token": token, "platform": "android", "app_version": "2.0" }),
        )
        .await;
    assert_eq!(r.status(), StatusCode::OK, "{}", r.text());
    assert_eq!(r.json::<serde_json::Value>()["id"], id);

    // Bob can't see, change or claim it.
    let list: serde_json::Value = app.get("/api/v3/notifications", &tb).await.json();
    assert!(
        !list.as_array().unwrap().iter().any(|n| n["id"] == id),
        "{list}"
    );
    let r = app.get(&format!("/api/v3/notifications/{id}"), &tb).await;
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
    let r = app
        .post_json(
            "/api/v3/notifications",
            &tb,
            &json!({ "token": token, "platform": "ios" }),
        )
        .await;
    assert_eq!(r.status(), StatusCode::FORBIDDEN, "{}", r.text());

    // After a push: the throttle fields, in UTC.
    let r = app
        .request(Method::PATCH, &format!("/api/v3/notifications/{id}"))
        .bearer(&ta)
        .json(&json!({ "badge_count": 3, "last_notified_at": "2026-10-18T12:00:00Z" }))
        .send()
        .await;
    assert_eq!(r.status(), StatusCode::OK, "{}", r.text());
    let row: serde_json::Value = r.json();
    assert_eq!(row["badge_count"], 3);
    assert!(
        row["last_notified_at"]
            .as_str()
            .unwrap()
            .starts_with("2026-10-18T12:00:00"),
        "{row}"
    );

    // A System editor sees everyone's.
    let admin = token_for(bob, UserPermissions::superuser());
    let all: serde_json::Value = app.get("/api/v3/notifications", &admin).await.json();
    assert!(all.as_array().unwrap().iter().any(|n| n["id"] == id));

    let r = app
        .request(Method::DELETE, &format!("/api/v3/notifications/{id}"))
        .bearer(&tb)
        .send()
        .await;
    assert_eq!(r.status(), StatusCode::NOT_FOUND, "not bob's to delete");
    let r = app
        .request(Method::DELETE, &format!("/api/v3/notifications/{id}"))
        .bearer(&ta)
        .send()
        .await;
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}
