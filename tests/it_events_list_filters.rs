//! Integration tests for the `GET /events` additions of GH #136: the "no tag"
//! and "any tag" filters, `tag_op`, three more sort fields, and result totals.
//!
//! Every test lists with `monitor_id` of a monitor it just made, so it only
//! sees its own events on a shared database.
//!
//! Requires the test database — run with:
//!   APP_PROFILE=test-db cargo test --test it_events_list_filters -- --include-ignored

mod common;

use axum::http::StatusCode;
use common::fixtures::{insert_monitor, unique_name, RowGuard};
use common::harness::{superuser_token, TestApp, TestResponse};
use rust_decimal::Decimal;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use zm_api::dto::response::PaginatedEventsResponse;

#[derive(Default)]
struct EventSpec {
    length_secs: i64,
    disk_space: Option<u64>,
    archived: u8,
    emailed: u8,
}

async fn insert_event(db: &sea_orm::DatabaseConnection, monitor_id: u32, spec: EventSpec) -> u64 {
    zm_api::entity::events::ActiveModel {
        monitor_id: Set(monitor_id),
        state_id: Set(1),
        name: Set(unique_name("EvList")),
        length: Set(Decimal::new(spec.length_secs, 0)),
        disk_space: Set(spec.disk_space),
        archived: Set(spec.archived),
        emailed: Set(spec.emailed),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("insert event fixture")
    .id
}

async fn insert_tag(db: &sea_orm::DatabaseConnection, label: &str) -> u64 {
    zm_api::entity::tags::ActiveModel {
        name: Set(unique_name(label)),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("insert tag fixture")
    .id
}

async fn tag_event(db: &sea_orm::DatabaseConnection, tag_id: u64, event_id: u64) -> RowGuard {
    zm_api::entity::events_tags::ActiveModel {
        tag_id: Set(tag_id),
        event_id: Set(event_id),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("insert events_tags fixture");
    RowGuard::new(
        format!("Events_Tags#({tag_id},{event_id})"),
        move |db| async move {
            let _ = zm_api::entity::events_tags::Entity::delete_by_id((tag_id, event_id))
                .exec(&db)
                .await;
        },
    )
}

fn guard_event(id: u64) -> RowGuard {
    RowGuard::new(format!("Events#{id}"), move |db| async move {
        let _ = zm_api::entity::events::Entity::delete_by_id(id)
            .exec(&db)
            .await;
    })
}

fn ids_in_order(resp: &TestResponse) -> Vec<u64> {
    assert_eq!(resp.status(), StatusCode::OK, "body: {}", resp.text());
    resp.json::<PaginatedEventsResponse>()
        .items
        .iter()
        .map(|e| e.id)
        .collect()
}

fn sorted(mut ids: Vec<u64>) -> Vec<u64> {
    ids.sort_unstable();
    ids
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn tag_filter_knows_no_tag_any_tag_and_not_in() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let monitor = insert_monitor(&app.db, "EvTagFilter").await.expect("mon");
    let _mon = RowGuard::monitor(monitor.id);

    let untagged = insert_event(&app.db, monitor.id, EventSpec::default()).await;
    let with_a = insert_event(&app.db, monitor.id, EventSpec::default()).await;
    let with_b = insert_event(&app.db, monitor.id, EventSpec::default()).await;
    let _events = [untagged, with_a, with_b].map(guard_event);
    let tag_a = insert_tag(&app.db, "EvListTagA").await;
    let tag_b = insert_tag(&app.db, "EvListTagB").await;
    let _tags = [RowGuard::tag(tag_a), RowGuard::tag(tag_b)];
    let _links = [
        tag_event(&app.db, tag_a, with_a).await,
        tag_event(&app.db, tag_b, with_b).await,
    ];

    let list = |qs: String| {
        let app = &app;
        let token = &token;
        let mid = monitor.id;
        async move {
            app.get(&format!("/api/v3/events?monitor_id={mid}&{qs}"), token)
                .await
        }
    };

    let resp = list("tag_id=none".into()).await;
    assert_eq!(ids_in_order(&resp), vec![untagged]);

    let resp = list("tag_id=any".into()).await;
    assert_eq!(sorted(ids_in_order(&resp)), vec![with_a, with_b]);

    // Legacy's sentinels for the same two, which zm-web's filter model carries.
    let resp = list("tag_id=0".into()).await;
    assert_eq!(ids_in_order(&resp), vec![untagged]);
    let resp = list("tag_id=-1".into()).await;
    assert_eq!(sorted(ids_in_order(&resp)), vec![with_a, with_b]);

    // not_in: everything that doesn't carry tag A, untagged events included.
    let resp = list(format!("tag_id={tag_a}&tag_op=not_in")).await;
    assert_eq!(sorted(ids_in_order(&resp)), vec![untagged, with_b]);

    // Plain ids still mean "any of these".
    let resp = list(format!("tag_id={tag_a},{tag_b}")).await;
    assert_eq!(sorted(ids_in_order(&resp)), vec![with_a, with_b]);

    let resp = list("tag_id=bogus".into()).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST, "{}", resp.text());
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn sorts_on_archived_emailed_and_disk_space() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let monitor = insert_monitor(&app.db, "EvSort").await.expect("mon");
    let _mon = RowGuard::monitor(monitor.id);

    let small = insert_event(
        &app.db,
        monitor.id,
        EventSpec {
            disk_space: Some(10),
            archived: 1,
            ..Default::default()
        },
    )
    .await;
    let big = insert_event(
        &app.db,
        monitor.id,
        EventSpec {
            disk_space: Some(999),
            emailed: 1,
            ..Default::default()
        },
    )
    .await;
    let _events = [small, big].map(guard_event);

    let first = |sort: &'static str| {
        let app = &app;
        let token = &token;
        let mid = monitor.id;
        async move {
            let resp = app
                .get(
                    &format!("/api/v3/events?monitor_id={mid}&sort={sort}&direction=desc"),
                    token,
                )
                .await;
            ids_in_order(&resp)[0]
        }
    };

    assert_eq!(first("disk_space").await, big);
    assert_eq!(first("archived").await, small);
    assert_eq!(first("emailed").await, big);
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn totals_cover_the_whole_result_not_the_page() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let monitor = insert_monitor(&app.db, "EvTotals").await.expect("mon");
    let _mon = RowGuard::monitor(monitor.id);

    let mut guards = Vec::new();
    for (len, disk) in [(10, Some(100)), (20, Some(200)), (30, None)] {
        let id = insert_event(
            &app.db,
            monitor.id,
            EventSpec {
                length_secs: len,
                disk_space: disk,
                ..Default::default()
            },
        )
        .await;
        guards.push(guard_event(id));
    }

    // One row per page: the sums must still cover all three.
    let resp = app
        .get(
            &format!(
                "/api/v3/events?monitor_id={}&page_size=1&totals=true",
                monitor.id
            ),
            &token,
        )
        .await;
    assert_eq!(resp.status(), StatusCode::OK, "body: {}", resp.text());
    let body: serde_json::Value = resp.json();
    assert_eq!(body["total"], 3);
    assert_eq!(body["items"].as_array().map(Vec::len), Some(1));
    assert_eq!(body["total_length_seconds"].as_f64(), Some(60.0));
    assert_eq!(body["total_disk_space"].as_u64(), Some(300));

    // Not asked for, not computed.
    let resp = app
        .get(&format!("/api/v3/events?monitor_id={}", monitor.id), &token)
        .await;
    let body: serde_json::Value = resp.json();
    assert!(body.get("total_length_seconds").is_none_or(|v| v.is_null()));
    assert!(body.get("total_disk_space").is_none_or(|v| v.is_null()));
}
