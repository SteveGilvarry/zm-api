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

pub mod mariadb;
pub mod store;

use std::sync::Arc;

use sea_orm::DatabaseConnection;
use tracing::{info, warn};

use crate::configure::search::SearchConfig;
use mariadb::MariaDbVectorStore;
use store::{Backend, NullVectorStore, VectorStore};

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

/// Holds the resolved vector backend + config and (later) the inference
/// providers. Constructed once at startup; cheap to share via `Arc`.
pub struct SearchService {
    config: SearchConfig,
    backend: Backend,
    store: Arc<dyn VectorStore>,
}

impl SearchService {
    /// Resolve the backend (probing the DB when `auto`/`mariadb`), build the
    /// matching store, and create its schema. Off-by-default: when disabled, no
    /// probe runs and the store is the no-op [`NullVectorStore`]. If schema
    /// creation fails, search degrades to disabled rather than failing startup.
    pub async fn new(db: Arc<DatabaseConnection>, config: SearchConfig) -> Self {
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

        // Create the index schema for a real backend; degrade to disabled on
        // failure (e.g. insufficient grants) so the rest of the API still boots.
        if !matches!(backend, Backend::None) {
            if let Err(e) = store.ensure_schema().await {
                warn!(
                    "event search: ensure_schema failed on {} backend ({e}); disabling search",
                    backend.as_str()
                );
                return Self {
                    config,
                    backend: Backend::None,
                    store: Arc::new(NullVectorStore),
                };
            }
        }

        info!(
            "event search: enabled={:?} → backend={}",
            config.enabled,
            backend.as_str()
        );

        Self {
            config,
            backend,
            store,
        }
    }

    /// Build a disabled service synchronously (no DB probe) — for tests and the
    /// off path where no async resolution is needed.
    pub fn disabled(config: SearchConfig) -> Self {
        Self {
            config,
            backend: Backend::None,
            store: Arc::new(NullVectorStore),
        }
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
}
