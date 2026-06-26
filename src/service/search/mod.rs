//! Natural-language / semantic event search (zm-api side).
//!
//! zm-next emits the analysis (descriptions, detections); zm-api **owns
//! persistence + retrieval + the HTTP API**. Per-event embeddings are indexed
//! into a runtime-selected [`store::VectorStore`] (sqlite-vec today, MariaDB
//! native `VECTOR` / pgvector later behind the same trait). All model inference
//! is **external local HTTP** — embeddings, rerank, and the query-router LLM are
//! chosen by URL, never in-process ONNX.
//!
//! Phase 0 wires the abstraction, the capability probe, config and the disabled
//! [`NullVectorStore`]; the concrete backends + providers land in later phases.

pub mod fusion;
pub mod mariadb;
pub mod provider;
pub mod store;

use std::sync::Arc;

use sea_orm::DatabaseConnection;
use tracing::{info, warn};

use crate::configure::search::SearchConfig;
use mariadb::MariaDbVectorStore;
use provider::{ChatProvider, EmbeddingProvider, HttpInference, RerankProvider};
use store::{Backend, EmbedKind, Embedding, Filter, Hit, NullVectorStore, UpsertItem, VectorStore};

/// Errors surfaced by the search subsystem. The HTTP layer maps these to
/// `AppError`.
#[derive(Debug, thiserror::Error)]
pub enum SearchError {
    /// Search is disabled or no backend is available.
    #[error("event search is disabled")]
    Disabled,
    /// A configured inference endpoint (embed/rerank/router) failed.
    #[error("inference provider error: {0}")]
    Provider(String),
    /// The vector backend failed.
    #[error("vector store error: {0}")]
    Store(String),
    #[error(transparent)]
    Db(#[from] sea_orm::error::DbErr),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Join(#[from] tokio::task::JoinError),
}

pub type SearchResult<T> = std::result::Result<T, SearchError>;

/// The grounded result of a `/search` query.
#[derive(Debug, Clone)]
pub struct SearchOutcome {
    /// A grounded NL answer when the router LLM is reachable; else `None` and the
    /// caller shows the citations directly.
    pub answer: Option<String>,
    /// The retrieved events (always present) — the verifiable ground truth.
    pub citations: Vec<Hit>,
    /// Exact count from the SQL count tool, for "how many …" queries.
    pub count: Option<u64>,
}

/// Holds the resolved vector backend, config, and the inference providers.
/// Constructed once at startup; cheap to share via `Arc`.
pub struct SearchService {
    config: SearchConfig,
    backend: Backend,
    store: Arc<dyn VectorStore>,
    embed: Arc<dyn EmbeddingProvider>,
    rerank: Arc<dyn RerankProvider>,
    chat: Arc<dyn ChatProvider>,
}

impl SearchService {
    /// Assemble from explicit components (lets tests inject canned providers /
    /// stores). Most callers use [`Self::new`].
    pub fn with_components(
        config: SearchConfig,
        backend: Backend,
        store: Arc<dyn VectorStore>,
        embed: Arc<dyn EmbeddingProvider>,
        rerank: Arc<dyn RerankProvider>,
        chat: Arc<dyn ChatProvider>,
    ) -> Self {
        Self {
            config,
            backend,
            store,
            embed,
            rerank,
            chat,
        }
    }

    /// Resolve the backend (probing the DB when `auto`/`mariadb`), build the
    /// matching store + HTTP inference providers, and create the schema.
    /// Off-by-default: when disabled, no probe runs and the store is the no-op
    /// [`NullVectorStore`]. Schema-creation failure degrades to disabled rather
    /// than failing startup.
    pub async fn new(
        client: reqwest::Client,
        db: Arc<DatabaseConnection>,
        config: SearchConfig,
    ) -> Self {
        let backend = if config.is_enabled() {
            store::detect_backend(&db, config.backend).await
        } else {
            Backend::None
        };

        let store: Arc<dyn VectorStore> = match backend {
            Backend::Mariadb => Arc::new(MariaDbVectorStore::new(db.clone(), config.embed_dim)),
            // sqlite-vec / pgvector are future backends; None = disabled.
            _ => Arc::new(NullVectorStore),
        };

        let backend = if !matches!(backend, Backend::None) {
            match store.ensure_schema().await {
                Ok(()) => backend,
                Err(e) => {
                    warn!(
                        "event search: ensure_schema failed on {} backend ({e}); disabling",
                        backend.as_str()
                    );
                    Backend::None
                }
            }
        } else {
            backend
        };
        let store: Arc<dyn VectorStore> = if matches!(backend, Backend::None) {
            Arc::new(NullVectorStore)
        } else {
            store
        };

        let http = Arc::new(HttpInference::new(client, &config));
        info!(
            "event search: enabled={:?} → backend={}",
            config.enabled,
            backend.as_str()
        );
        Self::with_components(config, backend, store, http.clone(), http.clone(), http)
    }

    /// Build a disabled service synchronously (no DB probe) — for tests and the
    /// off path where no async resolution is needed.
    pub fn disabled(config: SearchConfig) -> Self {
        let http = Arc::new(HttpInference::new(reqwest::Client::new(), &config));
        Self::with_components(
            config,
            Backend::None,
            Arc::new(NullVectorStore),
            http.clone(),
            http.clone(),
            http,
        )
    }

    /// True only when search is configured on *and* a usable backend resolved.
    pub fn enabled(&self) -> bool {
        self.config.is_enabled() && self.backend != Backend::None
    }

    pub fn backend(&self) -> Backend {
        self.backend
    }

    pub fn config(&self) -> &SearchConfig {
        &self.config
    }

    pub fn store(&self) -> &Arc<dyn VectorStore> {
        &self.store
    }

    /// Embed an event's text and upsert it into the index (embed-at-ingest).
    /// No-op when disabled or the text is empty.
    pub async fn index_text(
        &self,
        event_id: u64,
        monitor_id: u32,
        ts: i64,
        classes: Vec<String>,
        text: String,
    ) -> SearchResult<()> {
        if !self.enabled() || text.trim().is_empty() {
            return Ok(());
        }
        let vec = self
            .embed
            .embed(std::slice::from_ref(&text))
            .await?
            .into_iter()
            .next()
            .ok_or_else(|| SearchError::Provider("embed returned no vector".into()))?;
        self.store
            .upsert(&[UpsertItem {
                event_id,
                monitor_id,
                ts,
                kind: EmbedKind::Text,
                vec,
                classes,
                text,
            }])
            .await
    }

    /// Hybrid retrieval: embed the query, run vector ANN + lexical FTS over the
    /// pre-filtered set, fuse with RRF, and cross-encoder rerank the top-M.
    pub async fn search(&self, query: &str, filter: &Filter, k: usize) -> SearchResult<Vec<Hit>> {
        if !self.enabled() {
            return Err(SearchError::Disabled);
        }
        let qvec = self.embed_query(query).await?;
        let pool = (k * 4).max(20);
        let ann = self.store.search(&qvec, filter, pool).await?;
        let fts = self.store.fts(query, filter, pool).await?;

        let mut fused = fusion::reciprocal_rank_fusion(&[ann, fts], fusion::RRF_K);
        fused.truncate((k * 4).max(k)); // rerank window

        // Cross-encoder rerank (best-effort: keep RRF order if the endpoint fails).
        if self.config.rerank && fused.len() > 1 {
            let docs: Vec<String> = fused.iter().map(|h| h.snippet.clone()).collect();
            if let Ok(scores) = self.rerank.rerank(query, &docs).await {
                for (h, s) in fused.iter_mut().zip(scores) {
                    h.score = s;
                }
                fused.sort_by(|a, b| {
                    b.score
                        .partial_cmp(&a.score)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            }
        }
        fused.truncate(k);
        Ok(fused)
    }

    /// "More like this" — events nearest the given event's stored embedding.
    pub async fn similar(
        &self,
        event_id: u64,
        filter: &Filter,
        k: usize,
    ) -> SearchResult<Vec<Hit>> {
        if !self.enabled() {
            return Err(SearchError::Disabled);
        }
        self.store.similar(event_id, filter, k).await
    }

    /// Answer a query: hybrid retrieval for citations, the SQL count tool for
    /// "how many …" intents, and (when the router LLM is reachable) a grounded
    /// NL answer over the citations. Degrades gracefully without the LLM.
    pub async fn answer(
        &self,
        query: &str,
        filter: &Filter,
        k: usize,
    ) -> SearchResult<SearchOutcome> {
        let citations = self.search(query, filter, k).await?;
        let count = if is_count_query(query) {
            Some(self.store.count(filter).await?)
        } else {
            None
        };
        let answer = if self.chat.available() && !citations.is_empty() {
            self.grounded_answer(query, &citations, count).await.ok()
        } else {
            None
        };
        Ok(SearchOutcome {
            answer,
            citations,
            count,
        })
    }

    async fn embed_query(&self, query: &str) -> SearchResult<Embedding> {
        self.embed
            .embed(&[query.to_string()])
            .await?
            .into_iter()
            .next()
            .ok_or_else(|| SearchError::Provider("embed returned no vector".into()))
    }

    /// Closed-book, strict-citation prompt over the retrieved events.
    async fn grounded_answer(
        &self,
        query: &str,
        citations: &[Hit],
        count: Option<u64>,
    ) -> SearchResult<String> {
        let system = "You answer surveillance-event questions ONLY from the EVENTS provided. \
             Cite the event id (e.g. [event 512]) for every claim. If the events don't answer \
             the question, say so. For 'how many' questions use the COUNT provided verbatim — \
             never tally the events yourself.";
        let mut ctx = String::new();
        if let Some(c) = count {
            ctx.push_str(&format!("COUNT: {c}\n"));
        }
        ctx.push_str("EVENTS:\n");
        for h in citations {
            ctx.push_str(&format!("[event {}] {}\n", h.event_id, h.snippet));
        }
        let user = format!("{ctx}\nQUESTION: {query}");
        self.chat.complete(system, &user).await
    }
}

/// Heuristic count-intent detector (the LLM router can refine this later).
fn is_count_query(query: &str) -> bool {
    let q = query.to_lowercase();
    q.contains("how many") || q.contains("number of") || q.starts_with("count ")
}

#[cfg(test)]
mod tests {
    use super::provider::mock::MockInference;
    use super::store::{Embedding, Filter, Hit, UpsertItem, VectorStore};
    use super::*;
    use async_trait::async_trait;
    use std::sync::Arc;

    /// A canned store: fixed ANN / FTS lists and a fixed count, so we can test
    /// the service's fuse → rerank → truncate orchestration in isolation.
    struct CannedStore {
        ann: Vec<Hit>,
        fts: Vec<Hit>,
        count: u64,
    }

    fn hit(id: u64, snippet: &str) -> Hit {
        Hit {
            event_id: id,
            score: 0.0,
            ts: id as i64,
            snippet: snippet.to_string(),
        }
    }

    #[async_trait]
    impl VectorStore for CannedStore {
        async fn ensure_schema(&self) -> SearchResult<()> {
            Ok(())
        }
        async fn upsert(&self, _items: &[UpsertItem]) -> SearchResult<()> {
            Ok(())
        }
        async fn search(&self, _q: &Embedding, _f: &Filter, _k: usize) -> SearchResult<Vec<Hit>> {
            Ok(self.ann.clone())
        }
        async fn fts(&self, _q: &str, _f: &Filter, _k: usize) -> SearchResult<Vec<Hit>> {
            Ok(self.fts.clone())
        }
        async fn similar(&self, _id: u64, _f: &Filter, _k: usize) -> SearchResult<Vec<Hit>> {
            Ok(self.ann.clone())
        }
        async fn count(&self, _f: &Filter) -> SearchResult<u64> {
            Ok(self.count)
        }
        async fn rebuild(&self) -> SearchResult<()> {
            Ok(())
        }
        fn backend(&self) -> &'static str {
            "canned"
        }
    }

    fn service(store: CannedStore) -> SearchService {
        let cfg = SearchConfig {
            enabled: crate::configure::search::SearchEnabled::On,
            ..Default::default()
        };
        let mock = Arc::new(MockInference { dim: 8 });
        SearchService::with_components(
            cfg,
            Backend::Mariadb,
            Arc::new(store),
            mock.clone(),
            mock.clone(),
            mock,
        )
    }

    #[tokio::test]
    async fn search_fuses_then_reranks_by_query_overlap() {
        // event 1 is only in ANN, event 2 in both, event 3 only in FTS. The mock
        // reranker scores by query word-overlap, so the doc matching the query
        // text must surface first regardless of RRF order.
        let store = CannedStore {
            ann: vec![hit(1, "car on driveway"), hit(2, "person at door")],
            fts: vec![hit(2, "person at door"), hit(3, "empty scene")],
            count: 7,
        };
        let svc = service(store);
        let hits = svc
            .search("person at door", &Filter::default(), 5)
            .await
            .unwrap();
        assert_eq!(hits[0].event_id, 2, "best query overlap ranks first");
        assert_eq!(hits.len(), 3, "deduped union of ANN + FTS");
    }

    #[tokio::test]
    async fn answer_runs_count_tool_for_how_many_and_skips_llm() {
        let store = CannedStore {
            ann: vec![hit(1, "person at door")],
            fts: vec![hit(1, "person at door")],
            count: 42,
        };
        let svc = service(store);
        // "how many" → count tool fires; the mock chat reports unavailable so the
        // grounded answer is omitted (citations still returned).
        let out = svc
            .answer("how many people at the door", &Filter::default(), 5)
            .await
            .unwrap();
        assert_eq!(out.count, Some(42));
        assert!(out.answer.is_none(), "no LLM → no grounded answer");
        assert_eq!(out.citations.len(), 1);

        // A non-count query does not invoke the count tool.
        let out2 = svc
            .answer("person at the door", &Filter::default(), 5)
            .await
            .unwrap();
        assert!(out2.count.is_none());
    }

    #[tokio::test]
    async fn disabled_service_rejects_search() {
        let svc = SearchService::disabled(SearchConfig::default());
        assert!(!svc.enabled());
        assert!(matches!(
            svc.search("x", &Filter::default(), 5).await,
            Err(SearchError::Disabled)
        ));
    }
}
