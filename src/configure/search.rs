//! Configuration for natural-language / semantic event search.
//!
//! Search is **off by default**. When enabled it persists per-event embeddings
//! (text, optionally image) into a runtime-selected vector backend and serves
//! hybrid retrieval + a grounded NL answer. All model inference is **external
//! local HTTP** (embeddings / rerank / router LLM selected by URL, not in-process
//! ONNX) — see `src/service/search/`.

use serde::Deserialize;
use std::path::PathBuf;

/// Tri-state master switch. `Auto` enables search only when a usable vector
/// backend is detected; `Off` is a hard no-op (the default).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SearchEnabled {
    /// Enable iff a backend is available.
    Auto,
    /// Force on (still degrades to `none` if no backend can be opened).
    On,
    /// Hard off — no schema, no ingest, endpoints report disabled.
    #[default]
    Off,
}

/// Which vector backend to use. `Auto` resolves at startup via a *functional*
/// capability probe (not a version string).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BackendPref {
    /// Probe: mariadb-native → sqlite (→ postgres later).
    #[default]
    Auto,
    /// Force sqlite-vec (the universal floor; works on MySQL 8.0).
    Sqlite,
    /// Require MariaDB 11.8+ native `VECTOR`; disable search if absent.
    Mariadb,
    /// Require pgvector; disable search if absent.
    Postgres,
    /// Explicitly disabled.
    None,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct SearchConfig {
    pub enabled: SearchEnabled,
    pub backend: BackendPref,
    /// Embedding width; Matryoshka-truncate to 256–384.
    pub embed_dim: u32,
    /// Vector-index sqlite file (the sqlite-vec backend's store).
    pub sqlite_path: PathBuf,
    /// Cross-encoder rerank of the fused top-M (highest-ROI RAG lever).
    pub rerank: bool,
    /// Local text(+image) embeddings endpoint (TEI / llama.cpp / Ollama).
    pub embed_url: String,
    /// Local cross-encoder reranker endpoint.
    pub rerank_url: String,
    /// Local OpenAI-compatible LLM for the query router.
    pub router_url: String,
    /// Model name sent to the router endpoint (many local servers ignore it).
    pub router_model: String,
    /// Also embed the on-disk crop JPEGs (off = text-only).
    pub image_embed: bool,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            enabled: SearchEnabled::Off,
            backend: BackendPref::Auto,
            embed_dim: 384,
            sqlite_path: PathBuf::from("/var/lib/zm_api/search.db"),
            rerank: true,
            embed_url: "http://127.0.0.1:8090/embed".to_string(),
            rerank_url: "http://127.0.0.1:8090/rerank".to_string(),
            router_url: "http://127.0.0.1:8091/v1".to_string(),
            router_model: "local".to_string(),
            image_embed: false,
        }
    }
}

impl SearchConfig {
    /// True unless hard-`off` — i.e. ingest/serve paths should consider running.
    /// (`auto`/`on` still degrade to disabled if no backend opens.)
    pub fn is_enabled(&self) -> bool {
        self.enabled != SearchEnabled::Off
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_off_and_auto() {
        let c = SearchConfig::default();
        assert_eq!(c.enabled, SearchEnabled::Off);
        assert!(!c.is_enabled());
        assert_eq!(c.backend, BackendPref::Auto);
        assert_eq!(c.embed_dim, 384);
        assert!(c.rerank);
    }

    #[test]
    fn enabled_parses_tristate() {
        let on: SearchEnabled = serde_json::from_str("\"on\"").unwrap();
        let auto: SearchEnabled = serde_json::from_str("\"auto\"").unwrap();
        let off: SearchEnabled = serde_json::from_str("\"off\"").unwrap();
        assert_eq!(
            (on, auto, off),
            (SearchEnabled::On, SearchEnabled::Auto, SearchEnabled::Off)
        );
    }

    #[test]
    fn backend_parses() {
        let b: BackendPref = serde_json::from_str("\"sqlite\"").unwrap();
        assert_eq!(b, BackendPref::Sqlite);
    }
}
