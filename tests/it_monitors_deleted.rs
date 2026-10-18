//! Integration tests for soft-deleted monitors on `GET /monitors` (GH #137).
//!
//! ZoneMinder's web UI deletes a monitor by setting `Deleted = 1` and leaving
//! the row until its events are gone. The list hides those rows unless asked.
//!
//! Requires the test database — run with:
//!   APP_PROFILE=test-db cargo test --test it_monitors_deleted -- --include-ignored

mod common;

use axum::http::StatusCode;
use common::assertions::assert_status;
use common::fixtures::{insert_monitor, RowGuard};
use common::harness::{superuser_token, TestApp};
use sea_orm::{ActiveModelTrait, IntoActiveModel, Set};
use zm_api::dto::response::{MonitorResponse, PaginatedMonitorsResponse};

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn list_hides_soft_deleted_monitors_unless_asked() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    let live = insert_monitor(&app.db, "DelLive").await.expect("monitor");
    let _live = RowGuard::monitor(live.id);
    let gone = insert_monitor(&app.db, "DelGone").await.expect("monitor");
    let _gone = RowGuard::monitor(gone.id);
    let mut am = gone.clone().into_active_model();
    am.deleted = Set(1);
    am.update(&app.db).await.expect("soft-delete monitor");

    let list = |qs: &'static str| {
        let app = &app;
        let token = &token;
        async move {
            let resp = app
                .get(&format!("/api/v3/monitors?page_size=1000{qs}"), token)
                .await;
            assert_status(&resp, StatusCode::OK);
            resp.json::<PaginatedMonitorsResponse>().items
        }
    };

    let default = list("").await;
    assert!(default.iter().any(|m| m.id == live.id));
    assert!(
        default.iter().all(|m| m.id != gone.id),
        "a soft-deleted monitor must not appear by default"
    );

    let explicit_off = list("&include_deleted=false").await;
    assert!(explicit_off.iter().all(|m| m.id != gone.id));

    let all = list("&include_deleted=true").await;
    assert!(all.iter().any(|m| m.id == live.id));
    let found = all
        .iter()
        .find(|m| m.id == gone.id)
        .expect("include_deleted=true lists the soft-deleted monitor");
    assert!(found.deleted);
}

/// A deleted monitor's page must still load while its events exist.
#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn get_by_id_still_returns_a_soft_deleted_monitor() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    let gone = insert_monitor(&app.db, "DelGet").await.expect("monitor");
    let _gone = RowGuard::monitor(gone.id);
    let mut am = gone.clone().into_active_model();
    am.deleted = Set(1);
    am.update(&app.db).await.expect("soft-delete monitor");

    let resp = app
        .get(&format!("/api/v3/monitors/{}", gone.id), &token)
        .await;
    assert_status(&resp, StatusCode::OK);
    let body: MonitorResponse = resp.json();
    assert!(body.deleted);
}
