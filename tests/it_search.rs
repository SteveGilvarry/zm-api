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

use async_trait::async_trait;
use common::test_db::get_test_db;
use sea_orm::{ConnectionTrait, Statement};
use zm_api::configure::search::{BackendPref, SearchConfig, SearchEnabled};
use zm_api::service::search::mariadb::MariaDbVectorStore;
use zm_api::service::search::provider::{ChatProvider, EmbeddingProvider, RerankProvider};
use zm_api::service::search::store::{
    detect_backend, probe_mariadb_vector, Backend, EmbedKind, Embedding, Filter, UpsertItem,
    VectorStore,
};
use zm_api::service::search::{SearchResult, SearchService};

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

/// Serializes the two tests that mutate the shared `zmnext_event_vectors` table.
/// They run in one binary (so concurrently by default) and both write/replace
/// rows + maintain the vector index; concurrent writers can InnoDB-deadlock, so
/// we take this lock for the duration of each.
static TABLE_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

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
    let _guard = TABLE_LOCK.lock().await;
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

// ---------------------------------------------------------------------------
// Full-stack SearchService round trip (embed → upsert → ANN+FTS → RRF → rerank,
// plus the count tool and `similar`). Uses a deterministic in-test embedder so
// retrieval order is predictable without a live inference endpoint. Distinct
// event ids / monitor from the store test above (and the same VECTOR(4) schema)
// so the two can run concurrently against the shared table.
// ---------------------------------------------------------------------------

const SVC_EVENT_IDS: [u64; 4] = [992_001, 992_002, 992_003, 992_004];
const SVC_MONITOR: u32 = 6;

async fn cleanup_svc_vectors(db: &sea_orm::DatabaseConnection) {
    let ids = SVC_EVENT_IDS.map(|i| i.to_string()).join(",");
    let _ = db
        .execute(Statement::from_string(
            db.get_database_backend(),
            format!("DELETE FROM zmnext_event_vectors WHERE event_id IN ({ids})"),
        ))
        .await;
}

/// Deterministic bag-of-words embedder: identical text → identical vector, so an
/// exact-text query is guaranteed nearest. Width matches the VECTOR(dim) schema.
struct TestEmbed {
    dim: usize,
}

#[async_trait]
impl EmbeddingProvider for TestEmbed {
    async fn embed(&self, texts: &[String]) -> SearchResult<Vec<Embedding>> {
        Ok(texts
            .iter()
            .map(|t| {
                let mut v = vec![0.0f32; self.dim];
                for w in t.to_lowercase().split_whitespace() {
                    let h = w
                        .bytes()
                        .fold(0u64, |a, b| a.wrapping_mul(131).wrapping_add(b as u64));
                    v[(h as usize) % self.dim] += 1.0;
                }
                Embedding(v)
            })
            .collect())
    }
}

/// Reranks by query word-overlap (no network).
struct TestRerank;

#[async_trait]
impl RerankProvider for TestRerank {
    async fn rerank(&self, query: &str, docs: &[String]) -> SearchResult<Vec<f32>> {
        let q: std::collections::HashSet<&str> = query.split_whitespace().collect();
        Ok(docs
            .iter()
            .map(|d| d.split_whitespace().filter(|w| q.contains(w)).count() as f32)
            .collect())
    }
}

/// No router LLM in tests → answers degrade to citations only.
struct TestChat;

#[async_trait]
impl ChatProvider for TestChat {
    async fn complete(&self, _system: &str, _user: &str) -> SearchResult<String> {
        Ok(String::new())
    }
    fn available(&self) -> bool {
        false
    }
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn search_service_round_trip() {
    let _guard = TABLE_LOCK.lock().await;
    let db = get_test_db().await.expect("test db");
    if !probe_mariadb_vector(&db).await {
        eprintln!(
            "skipping search_service_round_trip: DB lacks native VECTOR (need MariaDB 11.8+)"
        );
        return;
    }

    let dim = 4u32;
    let store = Arc::new(MariaDbVectorStore::new(
        Arc::new(get_test_db().await.unwrap()),
        dim,
    ));
    store.ensure_schema().await.expect("ensure_schema");
    cleanup_svc_vectors(&db).await;

    let cfg = SearchConfig {
        enabled: SearchEnabled::On,
        embed_dim: dim,
        rerank: true,
        ..Default::default()
    };
    let svc = SearchService::with_components(
        cfg,
        Backend::Mariadb,
        store.clone(),
        Arc::new(TestEmbed { dim: dim as usize }),
        Arc::new(TestRerank),
        Arc::new(TestChat),
    );
    assert!(svc.enabled(), "On + Mariadb backend → enabled");

    // Embed-at-ingest: index four events on monitor 6.
    let docs: [(u64, &str, &[&str]); 4] = [
        (992_001, "person at the door", &["person"]),
        (992_002, "red car in the driveway", &["car"]),
        (992_003, "a cat on the lawn", &["animal"]),
        (992_004, "delivery van outside", &["vehicle"]),
    ];
    for (id, text, classes) in docs {
        svc.index_text(
            id,
            SVC_MONITOR,
            1_700_000_000 + id as i64,
            classes.iter().map(|s| s.to_string()).collect(),
            text.to_string(),
        )
        .await
        .expect("index_text");
    }

    let mine = Filter {
        monitor_ids: vec![SVC_MONITOR],
        ..Default::default()
    };

    // Hybrid retrieval: an exact-text query surfaces its event first.
    let hits = svc
        .search("person at the door", &mine, 5)
        .await
        .expect("search");
    assert_eq!(
        hits[0].event_id,
        992_001,
        "exact-text match ranks first; got {:?}",
        hits.iter().map(|h| h.event_id).collect::<Vec<_>>()
    );

    // Class pre-filter narrows to the matching events.
    let cars = svc
        .search(
            "vehicle",
            &Filter {
                monitor_ids: vec![SVC_MONITOR],
                classes: vec!["car".to_string()],
                ..Default::default()
            },
            5,
        )
        .await
        .expect("class-filtered search");
    assert!(
        cars.iter().all(|h| h.event_id == 992_002),
        "class=car returns only the car event; got {:?}",
        cars.iter().map(|h| h.event_id).collect::<Vec<_>>()
    );

    // Monitor pre-filter: a monitor with no rows yields nothing.
    let none = svc
        .search(
            "person at the door",
            &Filter {
                monitor_ids: vec![9999],
                ..Default::default()
            },
            5,
        )
        .await
        .expect("filtered search");
    assert!(none.is_empty(), "monitor pre-filter excludes all rows");

    // Count tool fires on "how many"; no LLM → no grounded answer.
    let out = svc
        .answer("how many events on this camera", &mine, 5)
        .await
        .expect("answer");
    assert_eq!(out.count, Some(4), "four distinct events indexed");
    assert!(out.answer.is_none(), "no router LLM → citations only");

    // "More like this": nearest neighbours of an event exclude itself.
    let similar = svc.similar(992_001, &mine, 5).await.expect("similar");
    assert!(
        similar.iter().all(|h| h.event_id != 992_001),
        "similar excludes the source event"
    );

    cleanup_svc_vectors(&db).await;
}
