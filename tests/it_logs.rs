//! Integration tests for the logs API (GH #21): inverted-scale `min_level`,
//! `search`, and `DELETE` clear-logs.
//!
//! Requires the test database — run with:
//!   APP_PROFILE=test-db cargo test --test it_logs -- --include-ignored

mod common;

use axum::http::{Method, StatusCode};
use common::fixtures::RowGuard;
use common::harness::{superuser_token, TestApp};
use common::test_db::{get_test_db, test_run_id};
use rust_decimal::Decimal;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use zm_api::dto::response::logs::PaginatedLogsResponse;

async fn insert_log(db: &DatabaseConnection, component: &str, level: i8, message: &str, t: i64) {
    zm_api::entity::logs::ActiveModel {
        time_key: Set(Decimal::new(t, 0)),
        component: Set(component.to_string()),
        server_id: Set(None),
        pid: Set(None),
        level: Set(level),
        code: Set("TST".to_string()),
        message: Set(message.to_string()),
        file: Set(None),
        line: Set(None),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("insert log");
}

/// Delete every log row for a component (test cleanup).
fn logs_guard(component: String) -> RowGuard {
    RowGuard::new(
        format!("Logs(component={component})"),
        move |db| async move {
            let _ = zm_api::entity::logs::Entity::delete_many()
                .filter(zm_api::entity::logs::Column::Component.eq(component))
                .exec(&db)
                .await;
        },
    )
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn min_level_returns_that_severity_or_worse() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    // Unique component so this test only sees its own rows on a shared DB.
    let rid = test_run_id();
    let component = format!("mlog{}", &rid[rid.len().saturating_sub(12)..]);
    let _g = logs_guard(component.clone());

    // Info(0), Warning(-1), Error(-2), Fatal(-3).
    insert_log(&app.db, &component, 0, "info msg", 1_000).await;
    insert_log(&app.db, &component, -1, "warning msg", 1_001).await;
    insert_log(&app.db, &component, -2, "error msg alpha", 1_002).await;
    insert_log(&app.db, &component, -3, "fatal msg", 1_003).await;

    // min_level=error → only Error + Fatal (the inverted-scale fix; the old
    // code's >= would have returned Info+Warning+Error).
    let resp = app
        .get(
            &format!("/api/v3/logs?component={component}&min_level=error&page_size=100"),
            &token,
        )
        .await;
    assert_eq!(resp.status(), StatusCode::OK, "body: {}", resp.text());
    let body: PaginatedLogsResponse = resp.json();
    assert_eq!(body.total, 2, "error+fatal only");
    assert!(
        body.items.iter().all(|l| l.level <= -2),
        "every returned row is Error or worse: {:?}",
        body.items.iter().map(|l| l.level).collect::<Vec<_>>()
    );

    // search filters by message substring.
    let resp = app
        .get(
            &format!("/api/v3/logs?component={component}&search=alpha&page_size=100"),
            &token,
        )
        .await;
    let body: PaginatedLogsResponse = resp.json();
    assert_eq!(body.total, 1, "only the 'error msg alpha' row");
    assert!(body.items[0].message.contains("alpha"));
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn clear_logs_deletes_matching_rows() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let rid = test_run_id();
    let component = format!("clog{}", &rid[rid.len().saturating_sub(12)..]);
    let _g = logs_guard(component.clone());

    for (i, lvl) in [0i8, -1, -2, -3].into_iter().enumerate() {
        insert_log(&app.db, &component, lvl, "clear me", 2_000 + i as i64).await;
    }

    // DELETE scoped to this component clears all four.
    let del = app
        .request(
            Method::DELETE,
            &format!("/api/v3/logs?component={component}"),
        )
        .bearer(&token)
        .send()
        .await;
    assert_eq!(del.status(), StatusCode::OK, "body: {}", del.text());

    // A verification connection: nothing left for this component.
    let db = get_test_db().await.expect("db");
    let remaining = zm_api::entity::logs::Entity::find()
        .filter(zm_api::entity::logs::Column::Component.eq(component.as_str()))
        .all(&db)
        .await
        .unwrap();
    assert!(
        remaining.is_empty(),
        "clear should remove all matching rows"
    );
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn zero_and_oversized_page_size_are_rejected_not_panics() {
    // LogQueryParams declared range(min = 1, max = 1000) but the handler never
    // called validate(), so the bounds were inert. page_size=0 reached
    // `total.div_ceil(0)` and SeaORM's `paginate(0)` — both panic on zero.
    let app = TestApp::spawn().await;
    let token = superuser_token();

    for qs in ["page_size=0", "page_size=100000", "page=0"] {
        let resp = app.get(&format!("/api/v3/logs?{qs}"), &token).await;
        assert_eq!(
            resp.status(),
            StatusCode::BAD_REQUEST,
            "{qs} must be a 400, not a panic or an unbounded query; body: {}",
            resp.text()
        );
    }

    // A value inside the declared range still works.
    let resp = app.get("/api/v3/logs?page_size=10&page=1", &token).await;
    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "an in-range page_size must still succeed; body: {}",
        resp.text()
    );
}
