//! `VectorStore` abstraction + runtime backend selection.
//!
//! One trait, several engine-specific impls (each hides its non-portable vector
//! SQL). The backend is chosen at startup by a **functional capability probe**
//! (not a version string), so a fork/patched server is detected correctly.
//! Today the box is MySQL 8.0 (no native `VECTOR`), so `auto` resolves to
//! sqlite-vec; MariaDB-native / pgvector are future backends behind the trait.

use async_trait::async_trait;
use sea_orm::{ConnectionTrait, DatabaseConnection, Statement};

use super::SearchResult;
use crate::configure::search::BackendPref;

/// A dense embedding vector.
#[derive(Debug, Clone, PartialEq)]
pub struct Embedding(pub Vec<f32>);

/// Whether an embedding came from text metadata or an image crop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmbedKind {
    Text,
    Image,
}

impl EmbedKind {
    pub fn as_str(self) -> &'static str {
        match self {
            EmbedKind::Text => "text",
            EmbedKind::Image => "image",
        }
    }
}

/// One row to persist: the embedding plus the metadata needed to pre-filter and
/// to do lexical (FTS) search.
#[derive(Debug, Clone)]
pub struct UpsertItem {
    pub event_id: u64,
    pub monitor_id: u32,
    /// Event time, epoch seconds (indexed for time pre-filtering).
    pub ts: i64,
    pub kind: EmbedKind,
    pub vec: Embedding,
    /// Object/class labels for class pre-filtering.
    pub classes: Vec<String>,
    /// Embedded text, kept for FTS/BM25 and as the result snippet.
    pub text: String,
}

/// Metadata pre-filter applied **before** the ANN — the decisive property for
/// selective surveillance queries.
#[derive(Debug, Clone, Default)]
pub struct Filter {
    pub monitor_ids: Vec<u32>,
    pub from: Option<i64>,
    pub to: Option<i64>,
    pub classes: Vec<String>,
}

/// One retrieved event.
#[derive(Debug, Clone, PartialEq)]
pub struct Hit {
    pub event_id: u64,
    pub score: f32,
    pub ts: i64,
    pub snippet: String,
}

/// A pluggable vector index. Impls keep all engine-specific SQL internal.
#[async_trait]
pub trait VectorStore: Send + Sync {
    /// Create the index schema if absent (idempotent).
    async fn ensure_schema(&self) -> SearchResult<()>;
    /// Insert/replace embedding rows.
    async fn upsert(&self, items: &[UpsertItem]) -> SearchResult<()>;
    /// Vector ANN over the pre-filtered set, best-first, up to `k`.
    async fn search(&self, q: &Embedding, f: &Filter, k: usize) -> SearchResult<Vec<Hit>>;
    /// Lexical (FTS/BM25) search over the same pre-filtered set.
    async fn fts(&self, query: &str, f: &Filter, k: usize) -> SearchResult<Vec<Hit>>;
    /// Re-embed from `Events`/descriptions (used after a backend switch — the
    /// index is derived, so switching backends is a rebuild, not a migration).
    async fn rebuild(&self) -> SearchResult<()>;
    /// Backend name for logs / `/search` diagnostics.
    fn backend(&self) -> &'static str;
}

/// The resolved backend after capability detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    Mariadb,
    Sqlite,
    Postgres,
    /// No usable backend — search disabled.
    None,
}

impl Backend {
    pub fn as_str(self) -> &'static str {
        match self {
            Backend::Mariadb => "mariadb",
            Backend::Sqlite => "sqlite",
            Backend::Postgres => "postgres",
            Backend::None => "none",
        }
    }
}

/// Pure backend decision given the configured preference and probe results.
/// Separated from the async probe so it is unit-testable without a DB.
pub fn choose_backend(pref: BackendPref, mariadb_native: bool) -> Backend {
    match pref {
        BackendPref::None => Backend::None,
        BackendPref::Sqlite => Backend::Sqlite,
        // Strict requests disable search when the capability is absent.
        BackendPref::Mariadb => {
            if mariadb_native {
                Backend::Mariadb
            } else {
                Backend::None
            }
        }
        // Postgres is a future backend; treat a strict request as not-yet-usable.
        BackendPref::Postgres => Backend::None,
        // Auto: prefer native MariaDB `VECTOR`, else the universal sqlite floor.
        BackendPref::Auto => {
            if mariadb_native {
                Backend::Mariadb
            } else {
                Backend::Sqlite
            }
        }
    }
}

/// Functionally probe the already-connected DB for native `VECTOR` support
/// (MariaDB 11.8+ / MySQL 9+). Tests the native vector **functions** with a
/// self-contained `SELECT` rather than a version string (so forks/distro patches
/// are detected) — and rather than a temp table, which is unreliable here because
/// sea-orm's `DatabaseConnection` is a *pool*: `CREATE`/`DROP` can land on
/// different connections, leaving a lingering connection-scoped table that breaks
/// the next probe. A pure `SELECT` is idempotent and pool-safe.
pub async fn probe_mariadb_vector(db: &DatabaseConnection) -> bool {
    db.query_one(Statement::from_string(
        db.get_database_backend(),
        "SELECT VEC_DISTANCE_COSINE(VEC_FromText('[1,2,3]'), VEC_FromText('[1,2,3]')) AS d",
    ))
    .await
    .is_ok()
}

/// Resolve the backend: probe the DB (when `auto`/`mariadb`) and apply
/// [`choose_backend`].
pub async fn detect_backend(db: &DatabaseConnection, pref: BackendPref) -> Backend {
    let needs_probe = matches!(pref, BackendPref::Auto | BackendPref::Mariadb);
    let mariadb_native = needs_probe && probe_mariadb_vector(db).await;
    choose_backend(pref, mariadb_native)
}

/// The disabled store: every operation is a successful no-op. Used when search
/// is off, when no backend is available, or until a backend impl is wired.
pub struct NullVectorStore;

#[async_trait]
impl VectorStore for NullVectorStore {
    async fn ensure_schema(&self) -> SearchResult<()> {
        Ok(())
    }
    async fn upsert(&self, _items: &[UpsertItem]) -> SearchResult<()> {
        Ok(())
    }
    async fn search(&self, _q: &Embedding, _f: &Filter, _k: usize) -> SearchResult<Vec<Hit>> {
        Ok(Vec::new())
    }
    async fn fts(&self, _query: &str, _f: &Filter, _k: usize) -> SearchResult<Vec<Hit>> {
        Ok(Vec::new())
    }
    async fn rebuild(&self) -> SearchResult<()> {
        Ok(())
    }
    fn backend(&self) -> &'static str {
        "none"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn choose_backend_honours_preference() {
        // Auto prefers native MariaDB, else falls to the sqlite floor.
        assert_eq!(choose_backend(BackendPref::Auto, true), Backend::Mariadb);
        assert_eq!(choose_backend(BackendPref::Auto, false), Backend::Sqlite);
        // Strict requests disable search when the capability is absent.
        assert_eq!(choose_backend(BackendPref::Mariadb, true), Backend::Mariadb);
        assert_eq!(choose_backend(BackendPref::Mariadb, false), Backend::None);
        // Sqlite is always available; None is explicit off.
        assert_eq!(choose_backend(BackendPref::Sqlite, false), Backend::Sqlite);
        assert_eq!(choose_backend(BackendPref::None, true), Backend::None);
        // Postgres is a future backend.
        assert_eq!(choose_backend(BackendPref::Postgres, true), Backend::None);
    }

    #[tokio::test]
    async fn null_store_is_a_clean_noop() {
        let store = NullVectorStore;
        assert_eq!(store.backend(), "none");
        store.ensure_schema().await.unwrap();
        store
            .upsert(&[UpsertItem {
                event_id: 1,
                monitor_id: 2,
                ts: 0,
                kind: EmbedKind::Text,
                vec: Embedding(vec![0.0; 4]),
                classes: vec![],
                text: "x".into(),
            }])
            .await
            .unwrap();
        assert!(store
            .search(&Embedding(vec![0.0; 4]), &Filter::default(), 5)
            .await
            .unwrap()
            .is_empty());
        assert!(store
            .fts("car", &Filter::default(), 5)
            .await
            .unwrap()
            .is_empty());
    }
}
