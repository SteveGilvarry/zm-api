//! Integration tests for NL/semantic event search (Phase 0).
//!
//! Verifies the *functional* vector-capability probe against the real DB: on a
//! server without native `VECTOR` (the box is MySQL 8.0), `auto` must resolve to
//! the sqlite floor and a strict `mariadb` request must disable search.
//!
//! Requires the test database — run with:
//!   APP_PROFILE=test-db cargo test --test it_search -- --include-ignored

mod common;

use std::sync::Arc;

use common::test_db::get_test_db;
use sea_orm::{ConnectionTrait, Statement};
use zm_api::configure::search::BackendPref;
use zm_api::service::search::mariadb::MariaDbVectorStore;
use zm_api::service::search::store::{
    detect_backend, probe_mariadb_vector, Backend, EmbedKind, Embedding, Filter, UpsertItem,
    VectorStore,
};

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn capability_probe_resolves_backend() {
    let db = get_test_db().await.expect("test db");

    // The functional probe (CREATE TEMPORARY TABLE … VECTOR(3)) — true only on a
    // native-vector server (MariaDB 11.8+ / MySQL 9+). The CI/dev box is MySQL
    // 8.0, so this is false.
    let native = probe_mariadb_vector(&db).await;

    if native {
        assert_eq!(
            detect_backend(&db, BackendPref::Auto).await,
            Backend::Mariadb
        );
        assert_eq!(
            detect_backend(&db, BackendPref::Mariadb).await,
            Backend::Mariadb
        );
    } else {
        // auto falls to the universal sqlite floor; strict mariadb disables.
        assert_eq!(
            detect_backend(&db, BackendPref::Auto).await,
            Backend::Sqlite
        );
        assert_eq!(
            detect_backend(&db, BackendPref::Mariadb).await,
            Backend::None
        );
    }

    // Explicit preferences never probe.
    assert_eq!(
        detect_backend(&db, BackendPref::Sqlite).await,
        Backend::Sqlite
    );
    assert_eq!(detect_backend(&db, BackendPref::None).await, Backend::None);
}

/// Test event ids well outside any real range; cleaned up before + after.
const TEST_EVENT_IDS: [u64; 5] = [991_001, 991_002, 991_003, 991_004, 991_005];

async fn cleanup_test_vectors(db: &sea_orm::DatabaseConnection) {
    let ids = TEST_EVENT_IDS.map(|i| i.to_string()).join(",");
    let _ = db
        .execute(Statement::from_string(
            db.get_database_backend(),
            format!("DELETE FROM zmnext_event_vectors WHERE event_id IN ({ids})"),
        ))
        .await;
}

fn item(event_id: u64, vec: Vec<f32>, classes: &[&str], text: &str) -> UpsertItem {
    UpsertItem {
        event_id,
        monitor_id: 5,
        ts: 1_700_000_000 + event_id as i64,
        kind: EmbedKind::Text,
        vec: Embedding(vec),
        classes: classes.iter().map(|s| s.to_string()).collect(),
        text: text.to_string(),
    }
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn mariadb_vector_round_trip() {
    let db = get_test_db().await.expect("test db");

    // Self-skip where there is no native VECTOR (the dev box is MySQL 8.0); this
    // exercises end-to-end only on MariaDB 11.8+ (CI).
    if !probe_mariadb_vector(&db).await {
        eprintln!(
            "skipping mariadb_vector_round_trip: DB lacks native VECTOR (need MariaDB 11.8+)"
        );
        return;
    }

    let store = MariaDbVectorStore::new(Arc::new(get_test_db().await.unwrap()), 4);
    store.ensure_schema().await.expect("ensure_schema");
    cleanup_test_vectors(&db).await;

    // 5 distinct events so FTS natural-language (50%-frequency rule) behaves.
    store
        .upsert(&[
            item(
                991_001,
                vec![1.0, 0.0, 0.0, 0.0],
                &["person"],
                "a person leaves a package at the door",
            ),
            item(
                991_002,
                vec![0.0, 1.0, 0.0, 0.0],
                &["car"],
                "a red car drives past the driveway",
            ),
            item(
                991_003,
                vec![0.9, 0.1, 0.0, 0.0],
                &["person"],
                "two people talking near the gate",
            ),
            item(
                991_004,
                vec![0.0, 0.9, 0.1, 0.0],
                &["vehicle"],
                "a delivery van stops outside",
            ),
            item(
                991_005,
                vec![0.0, 0.0, 1.0, 0.0],
                &["animal"],
                "a cat walks across the lawn",
            ),
        ])
        .await
        .expect("upsert");

    // ANN: query nearest the "person at door" vector → 991_001 ranks first.
    let hits = store
        .search(
            &Embedding(vec![0.97, 0.03, 0.0, 0.0]),
            &Filter::default(),
            5,
        )
        .await
        .expect("search");
    assert!(!hits.is_empty(), "ANN returns hits");
    assert_eq!(
        hits[0].event_id, 991_001,
        "nearest is the person-at-door event"
    );

    // Pre-filter: a monitor we didn't insert yields nothing.
    let other = Filter {
        monitor_ids: vec![999],
        ..Default::default()
    };
    let none = store
        .search(&Embedding(vec![1.0, 0.0, 0.0, 0.0]), &other, 5)
        .await
        .expect("filtered search");
    assert!(none.is_empty(), "monitor pre-filter excludes all test rows");

    // FTS: lexical "car" finds the car event.
    let fts = store.fts("car", &Filter::default(), 5).await.expect("fts");
    assert!(
        fts.iter().any(|h| h.event_id == 991_002),
        "FTS finds the car event; got {:?}",
        fts.iter().map(|h| h.event_id).collect::<Vec<_>>()
    );

    // Upsert replace: re-ingesting an event keeps a single row.
    store
        .upsert(&[item(
            991_001,
            vec![1.0, 0.0, 0.0, 0.0],
            &["person"],
            "updated text",
        )])
        .await
        .expect("re-upsert");
    let row = db
        .query_one(Statement::from_string(
            db.get_database_backend(),
            "SELECT COUNT(*) AS c FROM zmnext_event_vectors WHERE event_id = 991001".to_owned(),
        ))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        row.try_get::<i64>("", "c").unwrap(),
        1,
        "upsert replaced, not duplicated"
    );

    cleanup_test_vectors(&db).await;
}
