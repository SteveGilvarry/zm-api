//! Integration tests for the structured filter AST: create-with-AST round-trip
//! and the `/filters/preview` native execution.
//!
//! Requires the test database — run with:
//!   APP_PROFILE=test-db cargo test --test it_filters_ast -- --include-ignored

mod common;

use axum::http::StatusCode;
use common::assertions::assert_status;
use common::fixtures::{insert_monitor, unique_name, RowGuard};
use common::harness::{superuser_token, TestApp};
use sea_orm::{ActiveModelTrait, Set};
use serde_json::{json, Value};

/// Insert an event on `monitor_id` with the given max score; returns its id.
async fn insert_event(db: &sea_orm::DatabaseConnection, monitor_id: u32, max_score: u16) -> u64 {
    zm_api::entity::events::ActiveModel {
        monitor_id: Set(monitor_id),
        state_id: Set(1),
        name: Set(unique_name("AstEvent")),
        max_score: Set(Some(max_score)),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("insert event fixture")
    .id
}

fn guard_event(id: u64) -> RowGuard {
    RowGuard::new(format!("Events#{id}"), move |db| async move {
        let _ = <zm_api::entity::events::Entity as sea_orm::EntityTrait>::delete_by_id(id)
            .exec(&db)
            .await;
    })
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn create_filter_with_ast_stores_zm_json_and_round_trips() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    let body = json!({
        "name": unique_name("AstFilter"),
        "filter": {
            "where": {
                "match": "all",
                "rules": [
                    { "field": "monitor_id", "op": "eq", "value": 7 },
                    { "match": "any", "rules": [
                        { "field": "name", "op": "like", "value": "%front%" },
                        { "field": "max_score", "op": "gte", "value": 80 }
                    ]}
                ]
            },
            "sort": { "field": "start_time", "dir": "desc" },
            "limit": 100
        }
    });
    let create = app.post_json("/api/v3/filters", &token, &body).await;
    assert!(
        create.status().is_success(),
        "create should succeed; body: {}",
        create.text()
    );
    let created: Value = create.json();
    let id = created["id"].as_u64().expect("filter id") as u32;
    let _guard = RowGuard::filter(id);

    // The stored query_json must be ZoneMinder's flat terms format.
    let qj = created["query_json"].as_str().expect("query_json string");
    let qj: Value = serde_json::from_str(qj).expect("query_json is JSON");
    let terms = qj["terms"].as_array().expect("terms array");
    assert_eq!(terms.len(), 3, "three leaf conditions");
    assert_eq!(terms[0]["attr"], "MonitorId");
    assert_eq!(qj["sort_field"], "StartDateTime");

    // GET returns the structured AST reconstructed from storage.
    let got = app.get(&format!("/api/v3/filters/{id}"), &token).await;
    assert_status(&got, StatusCode::OK);
    let got: Value = got.json();
    assert!(got["filter"].is_object(), "response carries the AST");
    assert_eq!(got["filter"]["where"]["match"], "all");
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn preview_returns_only_matching_events() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    let monitor = insert_monitor(&app.db, "AstPreview")
        .await
        .expect("insert monitor");
    let _mon = RowGuard::monitor(monitor.id);
    let hi = insert_event(&app.db, monitor.id, 90).await;
    let _hi = guard_event(hi);
    let lo = insert_event(&app.db, monitor.id, 10).await;
    let _lo = guard_event(lo);

    // monitor_id = N AND max_score >= 80  -> only the high-score event.
    let body = json!({
        "where": {
            "match": "all",
            "rules": [
                { "field": "monitor_id", "op": "eq", "value": monitor.id },
                { "field": "max_score", "op": "gte", "value": 80 }
            ]
        }
    });
    let resp = app
        .post_json(
            "/api/v3/filters/preview?page=1&page_size=100",
            &token,
            &body,
        )
        .await;
    assert!(
        resp.status().is_success(),
        "preview should succeed; body: {}",
        resp.text()
    );
    let resp: Value = resp.json();
    let items = resp["items"].as_array().expect("items array");
    let ids: Vec<u64> = items
        .iter()
        .map(|e| e["id"].as_u64().expect("event id"))
        .collect();
    assert!(ids.contains(&hi), "high-score event should match");
    assert!(!ids.contains(&lo), "low-score event should be excluded");
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn preview_rejects_type_mismatch() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    // max_score is numeric; a string value must be rejected before any SQL.
    let body = json!({
        "where": {
            "match": "all",
            "rules": [ { "field": "max_score", "op": "gte", "value": "not-a-number" } ]
        }
    });
    let resp = app
        .post_json("/api/v3/filters/preview", &token, &body)
        .await;
    assert!(
        resp.status().is_client_error(),
        "type mismatch should be a 4xx, got {}; body: {}",
        resp.status(),
        resp.text()
    );
}
