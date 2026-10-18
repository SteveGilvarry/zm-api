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

/// `q` searches legacy's column set: a name substring, a cause, or an id as
/// text all find the event, and a miss finds nothing (#136).
#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn q_searches_across_columns() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let monitor = insert_monitor(&app.db, "EvListQ").await.expect("monitor");
    let _m = RowGuard::monitor(monitor.id);
    let event = zm_api::entity::events::ActiveModel {
        monitor_id: Set(monitor.id),
        state_id: Set(1),
        name: Set(unique_name("Driveway")),
        cause: Set(Some("Linked".to_string())),
        ..Default::default()
    }
    .insert(&app.db)
    .await
    .expect("event");
    let _e = guard_event(event.id);
    let other = insert_event(&app.db, monitor.id, EventSpec::default()).await;
    let _o = guard_event(other);

    let found = |q: String| {
        let app = &app;
        let token = token.clone();
        async move {
            let r = app
                .get(
                    &format!(
                        "/api/v3/events?monitor_id={}&q={}",
                        monitor.id,
                        urlencoding::encode(&q)
                    ),
                    &token,
                )
                .await;
            assert_eq!(r.status(), StatusCode::OK, "{}", r.text());
            r.json::<PaginatedEventsResponse>()
                .items
                .into_iter()
                .map(|e| e.id)
                .collect::<Vec<_>>()
        }
    };
    assert_eq!(found("Driveway".into()).await, vec![event.id]);
    assert_eq!(found("Linke".into()).await, vec![event.id]);
    assert!(found(event.id.to_string()).await.contains(&event.id));
    assert!(found("no-such-text-anywhere".into()).await.is_empty());
}

/// `GET /events/{id}` carries the event's directory as the server resolves
/// it; lists don't (#140).
#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn a_single_event_carries_its_directory() {
    use zm_api::entity::sea_orm_active_enums::Scheme;
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let monitor = insert_monitor(&app.db, "EvPath").await.expect("monitor");
    let _m = RowGuard::monitor(monitor.id);
    let event = zm_api::entity::events::ActiveModel {
        monitor_id: Set(monitor.id),
        state_id: Set(1),
        name: Set(unique_name("EvPath")),
        scheme: Set(Scheme::Shallow),
        ..Default::default()
    }
    .insert(&app.db)
    .await
    .expect("event");
    let _e = guard_event(event.id);

    let r = app
        .get(&format!("/api/v3/events/{}", event.id), &token)
        .await;
    assert_eq!(r.status(), StatusCode::OK, "{}", r.text());
    let body: serde_json::Value = r.json();
    let path = body["path"].as_str().expect("path present");
    assert!(
        path.ends_with(&format!("/{}/{}", monitor.id, event.id)),
        "shallow layout: {path}"
    );

    let r = app
        .get(&format!("/api/v3/events?monitor_id={}", monitor.id), &token)
        .await;
    let body: serde_json::Value = r.json();
    assert!(body["items"][0].get("path").is_none());
}

/// `GET /events/counts/{hours}?monitor_id=` is one monitor's hourly
/// histogram (#36).
#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn hourly_counts_can_be_for_one_monitor() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let a = insert_monitor(&app.db, "HistA").await.expect("monitor a");
    let _ma = RowGuard::monitor(a.id);
    let b = insert_monitor(&app.db, "HistB").await.expect("monitor b");
    let _mb = RowGuard::monitor(b.id);
    let now = chrono::Utc::now().naive_utc();
    let mut guards = Vec::new();
    for (mid, n) in [(a.id, 2), (b.id, 3)] {
        for _ in 0..n {
            let e = zm_api::entity::events::ActiveModel {
                monitor_id: Set(mid),
                state_id: Set(1),
                name: Set(unique_name("Hist")),
                start_date_time: Set(Some(now)),
                ..Default::default()
            }
            .insert(&app.db)
            .await
            .expect("event");
            guards.push(guard_event(e.id));
        }
    }
    let total = |mid: u32| {
        let app = &app;
        let token = token.clone();
        async move {
            let r = app
                .get(&format!("/api/v3/events/counts/2?monitor_id={mid}"), &token)
                .await;
            assert_eq!(r.status(), StatusCode::OK, "{}", r.text());
            r.json::<serde_json::Value>()["counts"]
                .as_array()
                .unwrap()
                .iter()
                .map(|c| c["count"].as_u64().unwrap())
                .sum::<u64>()
        }
    };
    assert_eq!(total(a.id).await, 2);
    assert_eq!(total(b.id).await, 3);
}
