//! External-inference providers (embeddings / rerank / router LLM).
//!
//! All model inference is **local HTTP**, selected by URL — no in-process ONNX.
//! The HTTP impl speaks the common shapes: TEI-style `/embed` + `/rerank`, and
//! an OpenAI-compatible `/chat/completions` for the router. The traits keep the
//! rest of the search code provider-agnostic and let tests inject canned
//! responses (no live endpoint needed in CI).

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::store::Embedding;
use super::{SearchError, SearchResult};
use crate::configure::search::SearchConfig;

/// Turns text into dense vectors.
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    async fn embed(&self, texts: &[String]) -> SearchResult<Vec<Embedding>>;
}

/// Cross-encoder reranker: a relevance score per doc, in the input order.
#[async_trait]
pub trait RerankProvider: Send + Sync {
    async fn rerank(&self, query: &str, docs: &[String]) -> SearchResult<Vec<f32>>;
}

/// OpenAI-compatible chat completion for the query router / grounded answer.
#[async_trait]
pub trait ChatProvider: Send + Sync {
    async fn complete(&self, system: &str, user: &str) -> SearchResult<String>;
    /// Cheap reachability flag so callers can degrade gracefully when the LLM
    /// endpoint isn't configured/up.
    fn available(&self) -> bool {
        true
    }
}

/// The local-HTTP inference client (implements all three provider traits).
pub struct HttpInference {
    client: reqwest::Client,
    embed_url: String,
    rerank_url: String,
    chat_url: String,
    router_model: String,
    /// Matryoshka-truncate embeddings to this width to match `VECTOR(dim)`.
    dim: usize,
    chat_configured: bool,
}

impl HttpInference {
    pub fn new(client: reqwest::Client, cfg: &SearchConfig) -> Self {
        Self {
            client,
            embed_url: cfg.embed_url.clone(),
            rerank_url: cfg.rerank_url.clone(),
            // OpenAI-compatible base → /chat/completions.
            chat_url: format!("{}/chat/completions", cfg.router_url.trim_end_matches('/')),
            router_model: if cfg.router_model.is_empty() {
                "local".to_string()
            } else {
                cfg.router_model.clone()
            },
            dim: cfg.embed_dim as usize,
            chat_configured: !cfg.router_url.is_empty(),
        }
    }

    fn provider_err(stage: &str) -> impl Fn(reqwest::Error) -> SearchError + '_ {
        move |e| SearchError::Provider(format!("{stage}: {e}"))
    }
}

#[derive(Serialize)]
struct EmbedReq<'a> {
    inputs: &'a [String],
    truncate: bool,
}

#[derive(Serialize)]
struct RerankReq<'a> {
    query: &'a str,
    texts: &'a [String],
}

#[derive(Deserialize)]
struct RerankItem {
    index: usize,
    score: f32,
}

#[derive(Serialize)]
struct ChatReq<'a> {
    model: &'a str,
    messages: Vec<ChatMsg<'a>>,
    temperature: f32,
}

#[derive(Serialize)]
struct ChatMsg<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct ChatResp {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatRespMsg,
}

#[derive(Deserialize)]
struct ChatRespMsg {
    content: String,
}

#[async_trait]
impl EmbeddingProvider for HttpInference {
    async fn embed(&self, texts: &[String]) -> SearchResult<Vec<Embedding>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }
        let raw: Vec<Vec<f32>> = self
            .client
            .post(&self.embed_url)
            .json(&EmbedReq {
                inputs: texts,
                truncate: true,
            })
            .send()
            .await
            .map_err(Self::provider_err("embed request"))?
            .error_for_status()
            .map_err(Self::provider_err("embed status"))?
            .json()
            .await
            .map_err(Self::provider_err("embed decode"))?;

        // Matryoshka-truncate to the index width; reject too-short vectors (a
        // model/dim misconfig) rather than persisting a wrong-width VECTOR.
        raw.into_iter()
            .map(|mut v| {
                if v.len() < self.dim {
                    return Err(SearchError::Provider(format!(
                        "embedding width {} < configured embed_dim {}",
                        v.len(),
                        self.dim
                    )));
                }
                v.truncate(self.dim);
                Ok(Embedding(v))
            })
            .collect()
    }
}

#[async_trait]
impl RerankProvider for HttpInference {
    async fn rerank(&self, query: &str, docs: &[String]) -> SearchResult<Vec<f32>> {
        if docs.is_empty() {
            return Ok(Vec::new());
        }
        let items: Vec<RerankItem> = self
            .client
            .post(&self.rerank_url)
            .json(&RerankReq { query, texts: docs })
            .send()
            .await
            .map_err(Self::provider_err("rerank request"))?
            .error_for_status()
            .map_err(Self::provider_err("rerank status"))?
            .json()
            .await
            .map_err(Self::provider_err("rerank decode"))?;

        // Re-order scores back into doc order.
        let mut scores = vec![0.0f32; docs.len()];
        for it in items {
            if let Some(s) = scores.get_mut(it.index) {
                *s = it.score;
            }
        }
        Ok(scores)
    }
}

#[async_trait]
impl ChatProvider for HttpInference {
    async fn complete(&self, system: &str, user: &str) -> SearchResult<String> {
        let resp: ChatResp = self
            .client
            .post(&self.chat_url)
            .json(&ChatReq {
                model: &self.router_model,
                messages: vec![
                    ChatMsg {
                        role: "system",
                        content: system,
                    },
                    ChatMsg {
                        role: "user",
                        content: user,
                    },
                ],
                temperature: 0.1,
            })
            .send()
            .await
            .map_err(Self::provider_err("chat request"))?
            .error_for_status()
            .map_err(Self::provider_err("chat status"))?
            .json()
            .await
            .map_err(Self::provider_err("chat decode"))?;

        resp.choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .ok_or_else(|| SearchError::Provider("chat: empty choices".into()))
    }

    fn available(&self) -> bool {
        self.chat_configured
    }
}

#[cfg(test)]
pub(crate) mod mock {
    //! Canned providers for tests — no network. Embeddings are derived from the
    //! text deterministically so similar texts get similar vectors.
    use super::*;

    pub struct MockInference {
        pub dim: usize,
    }

    /// Deterministic toy embedding: a few hashed/lexical features so that texts
    /// sharing words land near each other (enough to test ranking).
    fn toy_embed(text: &str, dim: usize) -> Embedding {
        let mut v = vec![0.0f32; dim];
        for word in text.to_lowercase().split_whitespace() {
            let h = word
                .bytes()
                .fold(0u64, |a, b| a.wrapping_mul(131).wrapping_add(b as u64));
            let idx = (h as usize) % dim;
            v[idx] += 1.0;
        }
        Embedding(v)
    }

    #[async_trait]
    impl EmbeddingProvider for MockInference {
        async fn embed(&self, texts: &[String]) -> SearchResult<Vec<Embedding>> {
            Ok(texts.iter().map(|t| toy_embed(t, self.dim)).collect())
        }
    }

    #[async_trait]
    impl RerankProvider for MockInference {
        async fn rerank(&self, query: &str, docs: &[String]) -> SearchResult<Vec<f32>> {
            // Score by shared-word overlap with the query.
            let qwords: std::collections::HashSet<&str> = query.split_whitespace().collect();
            Ok(docs
                .iter()
                .map(|d| d.split_whitespace().filter(|w| qwords.contains(w)).count() as f32)
                .collect())
        }
    }

    #[async_trait]
    impl ChatProvider for MockInference {
        async fn complete(&self, _system: &str, _user: &str) -> SearchResult<String> {
            Ok("{}".to_string())
        }
        fn available(&self) -> bool {
            false
        }
    }

    #[tokio::test]
    async fn mock_embed_is_deterministic_and_overlapping() {
        let m = MockInference { dim: 16 };
        let a = m.embed(&["a red car".into()]).await.unwrap();
        let b = m.embed(&["a red car".into()]).await.unwrap();
        assert_eq!(a, b, "deterministic");
        let scores = m
            .rerank(
                "red car",
                &["a red car drives".into(), "a cat sleeps".into()],
            )
            .await
            .unwrap();
        assert!(scores[0] > scores[1], "more word overlap ranks higher");
    }
}
