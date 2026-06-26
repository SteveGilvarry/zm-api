//! MariaDB 11.8+ native vector backend.
//!
//! Rides zm-api's existing sea-orm/sqlx MySQL connection — no new dependency.
//! The index lives in the same engine as ZoneMinder's data (one engine,
//! transactional). Uses native `VECTOR(dim)` columns + an HNSW `VECTOR INDEX`
//! (`VEC_DISTANCE_COSINE`) for ANN and a `FULLTEXT` index for lexical search.
//! Selected at startup by the functional capability probe; on a non-vector
//! server (MySQL 8.0 / MariaDB ≤ 11.7) the probe falls back, so this impl only
//! runs where `VECTOR` exists.

use std::sync::Arc;

use async_trait::async_trait;
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, Statement, Value};

use super::store::{Embedding, Filter, Hit, UpsertItem, VectorStore};
use super::{SearchError, SearchResult};

/// The zm-api-owned vector index table (additive — never touches ZM's tables).
const TABLE: &str = "zmnext_event_vectors";

pub struct MariaDbVectorStore {
    db: Arc<DatabaseConnection>,
    /// Fixed embedding width; the `VECTOR(dim)` column is created with this.
    dim: u32,
}

impl MariaDbVectorStore {
    pub fn new(db: Arc<DatabaseConnection>, dim: u32) -> Self {
        Self { db, dim }
    }

    fn st(&self, sql: impl Into<String>, values: Vec<Value>) -> Statement {
        Statement::from_sql_and_values(DbBackend::MySql, sql.into(), values)
    }

    /// Render an embedding as the `VEC_FromText('[…]')` argument string.
    fn vec_text(v: &Embedding) -> String {
        let mut s = String::with_capacity(v.0.len() * 8 + 2);
        s.push('[');
        for (i, x) in v.0.iter().enumerate() {
            if i > 0 {
                s.push(',');
            }
            // Plain decimal; MariaDB's VEC_FromText parses JSON-style floats.
            s.push_str(&format!("{x}"));
        }
        s.push(']');
        s
    }

    /// Build the metadata pre-filter `AND …` clause + ordered params for a
    /// row alias `t` (or the table itself when `alias` is empty).
    fn filter_clause(alias: &str, f: &Filter) -> (String, Vec<Value>) {
        let col = |c: &str| {
            if alias.is_empty() {
                c.to_string()
            } else {
                format!("{alias}.{c}")
            }
        };
        let mut clause = String::new();
        let mut params: Vec<Value> = Vec::new();

        if !f.monitor_ids.is_empty() {
            let marks = vec!["?"; f.monitor_ids.len()].join(",");
            clause.push_str(&format!(" AND {} IN ({marks})", col("monitor_id")));
            params.extend(f.monitor_ids.iter().map(|m| Value::from(*m)));
        }
        if let Some(from) = f.from {
            clause.push_str(&format!(" AND {} >= ?", col("ts")));
            params.push(Value::from(from));
        }
        if let Some(to) = f.to {
            clause.push_str(&format!(" AND {} <= ?", col("ts")));
            params.push(Value::from(to));
        }
        for class in &f.classes {
            clause.push_str(&format!(" AND {} LIKE ?", col("classes")));
            params.push(Value::from(format!("%{class}%")));
        }
        (clause, params)
    }
}

#[async_trait]
impl VectorStore for MariaDbVectorStore {
    async fn ensure_schema(&self) -> SearchResult<()> {
        // VECTOR(dim) NOT NULL is required for the index; FULLTEXT on `text` for
        // lexical search; a UNIQUE (event_id, kind) lets re-ingest upsert (NULLs
        // are distinct, so unreconciled rows never collide).
        let sql = format!(
            "CREATE TABLE IF NOT EXISTS {TABLE} (\
               id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,\
               event_id BIGINT UNSIGNED NULL,\
               monitor_id INT UNSIGNED NOT NULL,\
               ts BIGINT NOT NULL,\
               kind VARCHAR(8) NOT NULL,\
               classes TEXT NOT NULL,\
               vec VECTOR({dim}) NOT NULL,\
               `text` TEXT NOT NULL,\
               created_at DATETIME NOT NULL,\
               UNIQUE KEY uq_event_kind (event_id, kind),\
               VECTOR INDEX (vec) M=8 DISTANCE=cosine,\
               FULLTEXT INDEX ft_text (`text`),\
               INDEX idx_monitor_ts (monitor_id, ts),\
               INDEX idx_event (event_id)\
             ) ENGINE=InnoDB",
            dim = self.dim
        );
        self.db
            .execute(self.st(sql, vec![]))
            .await
            .map_err(|e| SearchError::Store(e.to_string()))?;
        Ok(())
    }

    async fn upsert(&self, items: &[UpsertItem]) -> SearchResult<()> {
        for item in items {
            // Replace any existing row for this (event_id, kind) — keeps one row
            // per event/kind without relying on VALUES() semantics over VECTOR.
            if item.event_id != 0 {
                self.db
                    .execute(self.st(
                        format!("DELETE FROM {TABLE} WHERE event_id = ? AND kind = ?"),
                        vec![Value::from(item.event_id), Value::from(item.kind.as_str())],
                    ))
                    .await
                    .map_err(|e| SearchError::Store(e.to_string()))?;
            }
            let event_id = (item.event_id != 0).then_some(item.event_id);
            self.db
                .execute(self.st(
                    format!(
                        "INSERT INTO {TABLE} \
                         (event_id, monitor_id, ts, kind, classes, vec, `text`, created_at) \
                         VALUES (?, ?, ?, ?, ?, VEC_FromText(?), ?, UTC_TIMESTAMP())"
                    ),
                    vec![
                        Value::from(event_id),
                        Value::from(item.monitor_id),
                        Value::from(item.ts),
                        Value::from(item.kind.as_str()),
                        Value::from(item.classes.join(" ")),
                        Value::from(Self::vec_text(&item.vec)),
                        Value::from(item.text.clone()),
                    ],
                ))
                .await
                .map_err(|e| SearchError::Store(e.to_string()))?;
        }
        Ok(())
    }

    async fn search(&self, q: &Embedding, f: &Filter, k: usize) -> SearchResult<Vec<Hit>> {
        // The HNSW index is used only for `ORDER BY VEC_DISTANCE_COSINE … LIMIT`,
        // and it post-filters, so over-fetch the nearest set then apply the
        // metadata filter + final LIMIT in the outer query (oversample when a
        // filter is present to keep recall).
        let has_filter = !f.monitor_ids.is_empty()
            || f.from.is_some()
            || f.to.is_some()
            || !f.classes.is_empty();
        let inner_k = if has_filter { (k * 8).max(k) } else { k };
        let (clause, mut fparams) = Self::filter_clause("t", f);

        let sql = format!(
            "SELECT event_id, ts, snippet, dist FROM (
               SELECT event_id, monitor_id, ts, classes, `text` AS snippet,
                      VEC_DISTANCE_COSINE(vec, VEC_FromText(?)) AS dist
               FROM {TABLE} ORDER BY dist LIMIT {inner_k}
             ) t WHERE t.event_id IS NOT NULL{clause} ORDER BY t.dist LIMIT {k}"
        );
        let mut params = vec![Value::from(Self::vec_text(q))];
        params.append(&mut fparams);

        let rows = self
            .db
            .query_all(self.st(sql, params))
            .await
            .map_err(|e| SearchError::Store(e.to_string()))?;

        Ok(rows
            .into_iter()
            .filter_map(|r| {
                let event_id: u64 = r.try_get("", "event_id").ok()?;
                let ts: i64 = r.try_get("", "ts").ok()?;
                let dist: f64 = r.try_get("", "dist").ok()?;
                let snippet: String = r.try_get("", "snippet").unwrap_or_default();
                Some(Hit {
                    event_id,
                    // Cosine distance → similarity score (1 = identical).
                    score: (1.0 - dist) as f32,
                    ts,
                    snippet,
                })
            })
            .collect())
    }

    async fn fts(&self, query: &str, f: &Filter, k: usize) -> SearchResult<Vec<Hit>> {
        let (clause, mut fparams) = Self::filter_clause("", f);
        let sql = format!(
            "SELECT event_id, ts, `text` AS snippet,
                    MATCH(`text`) AGAINST(? IN NATURAL LANGUAGE MODE) AS score
             FROM {TABLE}
             WHERE event_id IS NOT NULL
               AND MATCH(`text`) AGAINST(? IN NATURAL LANGUAGE MODE){clause}
             ORDER BY score DESC LIMIT {k}"
        );
        let mut params = vec![Value::from(query), Value::from(query)];
        params.append(&mut fparams);

        let rows = self
            .db
            .query_all(self.st(sql, params))
            .await
            .map_err(|e| SearchError::Store(e.to_string()))?;

        Ok(rows
            .into_iter()
            .filter_map(|r| {
                let event_id: u64 = r.try_get("", "event_id").ok()?;
                let ts: i64 = r.try_get("", "ts").ok()?;
                let score: f64 = r.try_get("", "score").unwrap_or(0.0);
                let snippet: String = r.try_get("", "snippet").unwrap_or_default();
                Some(Hit {
                    event_id,
                    score: score as f32,
                    ts,
                    snippet,
                })
            })
            .collect())
    }

    async fn similar(&self, event_id: u64, f: &Filter, k: usize) -> SearchResult<Vec<Hit>> {
        // ANN against the stored vector of `event_id`, excluding itself. The
        // index needs ORDER BY VEC_DISTANCE … LIMIT, so over-fetch then filter.
        let has_filter = !f.monitor_ids.is_empty()
            || f.from.is_some()
            || f.to.is_some()
            || !f.classes.is_empty();
        let inner_k = if has_filter { (k * 8).max(k) } else { k } + 1; // +1 for self
        let (clause, mut fparams) = Self::filter_clause("t", f);

        let sql = format!(
            "SELECT event_id, ts, snippet, dist FROM (
               SELECT event_id, monitor_id, ts, classes, `text` AS snippet,
                      VEC_DISTANCE_COSINE(vec, (SELECT vec FROM {TABLE} WHERE event_id = ? LIMIT 1)) AS dist
               FROM {TABLE} ORDER BY dist LIMIT {inner_k}
             ) t WHERE t.event_id IS NOT NULL AND t.event_id <> ?{clause}
             ORDER BY t.dist LIMIT {k}"
        );
        let mut params = vec![Value::from(event_id), Value::from(event_id)];
        params.append(&mut fparams);

        let rows = self
            .db
            .query_all(self.st(sql, params))
            .await
            .map_err(|e| SearchError::Store(e.to_string()))?;
        Ok(rows
            .into_iter()
            .filter_map(|r| {
                let event_id: u64 = r.try_get("", "event_id").ok()?;
                let ts: i64 = r.try_get("", "ts").ok()?;
                let dist: f64 = r.try_get("", "dist").ok()?;
                let snippet: String = r.try_get("", "snippet").unwrap_or_default();
                Some(Hit {
                    event_id,
                    score: (1.0 - dist) as f32,
                    ts,
                    snippet,
                })
            })
            .collect())
    }

    async fn count(&self, f: &Filter) -> SearchResult<u64> {
        let (clause, params) = Self::filter_clause("", f);
        let sql = format!(
            "SELECT COUNT(DISTINCT event_id) AS c FROM {TABLE} WHERE event_id IS NOT NULL{clause}"
        );
        let row = self
            .db
            .query_one(self.st(sql, params))
            .await
            .map_err(|e| SearchError::Store(e.to_string()))?
            .ok_or_else(|| SearchError::Store("count returned no row".into()))?;
        let c: i64 = row.try_get("", "c").unwrap_or(0);
        Ok(c.max(0) as u64)
    }

    async fn rebuild(&self) -> SearchResult<()> {
        // Re-embedding from Events/descriptions needs the embedding provider,
        // wired in the embed-at-ingest phase. The schema is created here; rebuild
        // is a no-op until then.
        self.ensure_schema().await
    }

    fn backend(&self) -> &'static str {
        "mariadb"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::search::store::EmbedKind;

    #[test]
    fn vec_text_formats_json_array() {
        let s = MariaDbVectorStore::vec_text(&Embedding(vec![1.0, -0.5, 2.25]));
        assert_eq!(s, "[1,-0.5,2.25]");
    }

    #[test]
    fn filter_clause_builds_params_in_order() {
        let f = Filter {
            monitor_ids: vec![3, 7],
            from: Some(100),
            to: Some(200),
            classes: vec!["person".into()],
        };
        let (clause, params) = MariaDbVectorStore::filter_clause("t", &f);
        assert!(clause.contains("t.monitor_id IN (?,?)"));
        assert!(clause.contains("t.ts >= ?"));
        assert!(clause.contains("t.ts <= ?"));
        assert!(clause.contains("t.classes LIKE ?"));
        // 2 monitors + from + to + 1 class = 5 params.
        assert_eq!(params.len(), 5);
    }

    #[test]
    fn empty_filter_is_empty_clause() {
        let (clause, params) = MariaDbVectorStore::filter_clause("t", &Filter::default());
        assert!(clause.is_empty());
        assert!(params.is_empty());
    }

    #[test]
    fn upsert_item_kind_maps_to_column_string() {
        assert_eq!(EmbedKind::Text.as_str(), "text");
        assert_eq!(EmbedKind::Image.as_str(), "image");
    }
}
