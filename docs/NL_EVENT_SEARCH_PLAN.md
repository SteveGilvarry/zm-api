# Natural-Language Event Search + Vector Store — zm-api Implementation Plan

**Audience:** an engineer/AI working in the **zm-api** (Rust/Axum) repo. This is an execution-ready
plan to build semantic + natural-language search over zm-next events — the single largest shipped-
feature gap vs Frigate. Design rationale lives in the **zm-next** repo at
`docs/Research_Motion_and_LLM_Review.md` (NL-search design) and the *Vector store* section there
(feature-flagged, capability-detected backend). File paths below are against the current zm-api tree;
verify line numbers before editing.

## zm-api current state (VERIFIED — corrects earlier assumptions)
An earlier draft of this plan inherited assumptions from a code *audit*, not the repo. Ground truth
(Cargo.toml + src), so don't be misled:
- **No ML infra exists.** No `ort`/onnxruntime, no `tokenizers`, no embedding/reranker/LLM code. The
  whole embed + rerank + router stack is **greenfield**. (The `describe_vlm`/VLM provider lives in
  **zm-next**, not here — it cannot be "reused" from zm-api.)
- **DB is MySQL 8.0** → **no native `VECTOR`** (that's MySQL 9.0+/MariaDB 11.8+). So **sqlite-vec is the
  primary backend today**, not a fallback; MariaDB-native/pgvector are *future* backends behind the trait.
- **sea-orm is `sqlx-mysql` + `sqlx-postgres` only (no sqlite).** sqlite-vec is therefore a **new
  dependency on a separate connection** (`rusqlite` or `sqlx` sqlite + the sqlite-vec C extension),
  NOT through sea-orm. zm-api keeps its sea-orm MySQL connection for ZM data + a separate sqlite handle
  for the vector index.
- **What already exists and helps:** `reqwest` 0.13 (HTTP → external inference endpoints are cheap),
  `image` 0.25 (decode crop JPEGs in-process, no ffmpeg needed for embeds), `ffmpeg-next` 8
  (`spawn_blocking` decode pattern in `src/streaming/snapshot.rs`), DashMap caching, sea-orm-migration,
  the `EventIngestor` (`src/service/zmnext/ingest.rs`), `MonitorScope` ACL + `media_auth_middleware`.

## Inference architecture — DECIDED: external local HTTP endpoints (Option 1)
The router LLM must be an endpoint regardless (no in-process function-calling LLM in Rust), so unify
**embeddings + rerank + router** on one local-HTTP provider pattern — matching zm-next's existing
local mlx-vlm sidecar. **Do NOT add `ort`/in-process ONNX** (heavy native build + shipped model assets,
and it wouldn't remove the router endpoint anyway). Concretely:
- Run local model servers on the box: a **text-embeddings + rerank** server (TEI / llama.cpp / Ollama —
  expose embed + rerank endpoints) and a **local OpenAI-compatible LLM** for the router (llama.cpp /
  vllm-mlx / Ollama; may be the same server already serving the VLM).
- In zm-api define `EmbeddingProvider` / `RerankProvider` / `ChatProvider` traits over **`reqwest`**
  (local-first, optional cloud fallback, models selected by config **URL** — not model files).
- Image embeddings: decode the on-disk crop JPEG with the **`image`** crate and POST to the embed
  endpoint. (Optional later optimization: accept a zm-next-emitted image embedding as a precomputed
  input — never a dependency, so search works for non-zm-next/legacy cameras too.)
This supersedes any "ort already used / reuse describe_vlm provider" wording below.

## What zm-next already provides (don't rebuild)
- **Event metadata** over the worker stream-socket: `detection` (0x0301), `description` (0x0302,
  VLM scene text from `describe_vlm`/`llm_event_review` with `{description, threat_level, confidence}`),
  `recording_saved` (0x0303), and `review_assets` (0x0306). Ingested today by
  `src/service/zmnext/ingest.rs` (`EventIngestor`) → `Events`/`Frames` rows.
- **Track crops / thumbnails** on disk (event clips + motion-synopsis tube cutouts under
  `{event_dir}/synopsis/`) — the image source for multimodal embeddings, already-decoded (no re-decode).
- zm-next is **DB-less**; it can optionally emit an image embedding as an event field (emit-ingredients
  pattern), but zm-api may also compute embeddings itself. Either way **zm-api owns all persistence.**

> **Anti-recompute rule:** prefer embedding the **already-produced** description text + existing crop
> thumbnails. Do not re-decode clips or re-run detection to build the index.

## Architecture
```
zm-next events ──▶ EventIngestor (ingest.rs)
                      │  on description/detection/recording_saved:
                      ▼
              embed-at-ingest (text [+ image]) ──▶ VectorStore.upsert()
                                                      │  (MariaDB-native | sqlite-vec | pg, probed)
   GET /api/v3/search?q=... ──▶ QueryRouter ─┬─ count_events()  → direct SQL COUNT/GROUP BY
                                             └─ search_events() → hybrid: SQL pre-filter
                                                  → vector ANN + FTS → RRF → cross-encoder rerank
                                             → grounded answer (strict-citation, verify event_id)
```

---

## Phase 0 — VectorStore abstraction + capability detection + feature flag

**Goal:** one interface, runtime-selected backend, off by default, graceful on old DBs.

`src/service/search/store.rs` (new):
```rust
pub struct Embedding(pub Vec<f32>);
pub struct UpsertItem { pub event_id: u64, pub monitor_id: u32, pub ts: i64,
    pub kind: EmbedKind /*Text|Image*/, pub vec: Embedding, pub text: String /*for FTS*/ }
pub struct Filter { pub monitor_ids: Vec<u32>, pub from: Option<i64>, pub to: Option<i64>,
    pub classes: Vec<String> }
pub struct Hit { pub event_id: u64, pub score: f32, pub ts: i64, pub snippet: String }

#[async_trait]
pub trait VectorStore: Send + Sync {
    async fn ensure_schema(&self) -> Result<()>;
    async fn upsert(&self, items: &[UpsertItem]) -> Result<()>;
    async fn search(&self, q: &Embedding, f: &Filter, k: usize) -> Result<Vec<Hit>>; // pre-filtered
    async fn fts(&self, query: &str, f: &Filter, k: usize) -> Result<Vec<Hit>>;       // lexical
    async fn rebuild(&self) -> Result<()>;  // re-embed from Events/descriptions
    fn backend(&self) -> &'static str;
}
```
Impls: `MariaDbVectorStore`, `SqliteVecStore`, `PgVectorStore` (later), `NullVectorStore` (disabled).
The engine-specific vector SQL is **hidden inside each impl** (not ORM-portable) — see *Backends*.

**Feature flag** — add `src/configure/search.rs` + a `[search]` block (mirror `src/configure/zmnext.rs`,
wire into `AppState` in `src/server/state.rs`):
```toml
[search]
enabled = "auto"          # auto | on | off
backend = "auto"          # auto -> sqlite today (MySQL 8.0 has no native VECTOR); mariadb|postgres later
embed_dim = 384           # truncate (Matryoshka) to 256–384
sqlite_path = "/var/lib/zm_api/search.db"
rerank = true
# Inference = external local HTTP endpoints (Option 1). Models are chosen by URL, not files:
embed_url  = "http://127.0.0.1:8090/embed"     # text (+image) embeddings (TEI/llama.cpp/Ollama)
rerank_url = "http://127.0.0.1:8090/rerank"    # cross-encoder reranker (bge-reranker-v2-m3)
router_url = "http://127.0.0.1:8091/v1"        # OpenAI-compatible LLM for the query router
image_embed = false       # POST on-disk crop JPEGs to embed_url too (off = text-only)
```

**Capability detection** (`store::detect_backend(db) -> Backend`): probe the *already-connected* DB
**functionally**, not by version string (forks/distro patches):
- MariaDB: `SELECT VERSION()` ≥ 11.8 **and** a `CREATE TEMPORARY TABLE _vchk(v VECTOR(3))` probe
  succeeds → `mariadb`. (Community 11.8 LTS ships native `VECTOR` + HNSW `VEC_DISTANCE_*`, GPLv2.)
- Postgres: `CREATE EXTENSION IF NOT EXISTS vector` succeeds → `postgres`.
- else → `sqlite` (universal floor: bundled `sqlite-vec`, no server-version dependency).
`backend="auto"` picks mariadb → sqlite (→ postgres later); `backend="mariadb"` (strict) disables
search when native is absent. Cache + log the chosen backend loudly.

---

## Phase 1 — schema + embed-at-ingest

**Schema** (zm-api-owned, **additive — never touch ZoneMinder's shared tables**). SeaORM migration
adds `zmnext_event_vectors`: `id`, `event_id` (FK, nullable until reconciled), `monitor_id`, `ts`
(epoch int, indexed), `kind` (text|image), `classes` (text), `dim`, `vec` (engine-native vector
type / BLOB for sqlite-vec), `text` (for FTS), `created_at`. Plus an FTS index (MariaDB FULLTEXT /
Postgres tsvector+GIN / sqlite FTS5).

**Embedding models** (local, compute-once-per-event) — called over HTTP (`reqwest`) to the local
`embed_url`, NOT in-process ONNX (Option 1):
- **Text** (always): `bge-small-en-v1.5` or `Qwen3-Embedding-0.6B` (one family with the router),
  served by the local embeddings endpoint. Truncate to `embed_dim` (Matryoshka).
- **Image** (optional, higher recall): SigLIP2 / JinaCLIP-v2 of the event/track-crop thumbnail
  (already on disk under the event dir / `synopsis/` — no re-decode). Store as a second `kind=image`
  row; fuse at query time.
- Build the embed text from event metadata (see zm-next design): `"{cam} {ts} | {labels} | {description}"`.

**Hook:** extend `EventIngestor` (`src/service/zmnext/ingest.rs`) with
`async fn handle_description(...)` / on detection/recording_saved → build `UpsertItem`(s) →
`VectorStore.upsert()`. If `enabled != on/auto`, no-op.

---

## Phase 2 — hybrid retrieval

`VectorStore.search()` must **pre-filter** (monitor/time/class) *before* the ANN — the decisive
property for selective surveillance queries. Then:
1. vector ANN (top-N) + FTS/BM25 (top-N) over the same filtered set;
2. fuse with **Reciprocal Rank Fusion** `score = Σ 1/(k + rank)`, `k=60`;
3. **cross-encoder rerank** the fused top-M with `bge-reranker-v2-m3` / `Qwen3-Reranker-0.6B`
   (highest-ROI RAG lever, ~+17% Recall@5) → final top-k.

---

## Phase 3 — query router (not pure RAG)

`src/service/search/router.rs`: a NEW `ChatProvider` (over `reqwest` → `router_url`, OpenAI-compatible;
the `describe_vlm` provider is in zm-next and is NOT available here) — a local function-calling LLM
classifies intent + extracts slots (`time_range` → epoch
bounds resolved **in code**, `monitor_id`, `classes`), then routes:
- **count** → `count_events(time_range, monitor_id, classes, group_by)` = direct SQL `COUNT…GROUP BY`
  (never let the LLM tally — it miscounts; top-k truncation undercounts).
- **search/what/when** → `search_events(...)` = the Phase-2 hybrid retrieval.

**Grounded answer:** strict-citation closed-book prompt — answer only from the returned events; every
claim cites an `event_id`; counts come only from the count tool. **Verify each cited `event_id` exists**
in the result set before returning (drop/flag unverifiable). Returned `event_id`s deep-link to the
clip/snapshot in the apps.

---

## Phase 4 — HTTP API + apps

Routes in `src/routes/` (wrap with `media_auth_middleware`; enforce `MonitorScope::allows` per result
exactly like `src/handlers/events_playback.rs:198-210` — filter hits to monitors the caller may see):
- `GET /api/v3/search?q=&from=&to=&monitor_id=&k=` → `{ answer, citations:[{event_id,ts,snippet,url}],
  count? }`.
- `GET /api/v3/events/{id}/similar?k=` → "find more like this" (vector NN of the event's embedding).
Cache hot queries (DashMap, mirror `SnapshotService` `src/streaming/snapshot.rs:52-62`); ETag.
Apps: NL query box + "find more like this" + citation deep-links in zm-dashboard / zm-mobile.

---

## Backends (the engine-specific SQL each impl hides)
- **sqlite-vec — PRIMARY today** (the box is MySQL 8.0, no native vector). A separate sqlite file
  (`sqlite_path`) opened via `rusqlite`/`sqlx` (NOT sea-orm, which has no sqlite feature here) with the
  sqlite-vec C extension loaded; `vec0` virtual table + FTS5; pre-filter via in-DB joins. Build this one
  first — it unblocks everything and works on MySQL 8.0.
- **MariaDB 11.8+ (future):** `vec` is `VECTOR(dim)`; `VECTOR INDEX` (HNSW);
  `ORDER BY VEC_DISTANCE_COSINE(vec, ?)` with the metadata pre-filter in the same `WHERE`; FTS via
  `MATCH … AGAINST`. Only when the deployment's DB is upgraded to MariaDB 11.8+; then it collapses to
  one engine (no separate sqlite file). Selected by the capability probe.
- **pgvector ≥ 0.8 (future):** `vector` type, HNSW, `<=>` cosine, **iterative index scans** for filtered
  search; tsvector/GIN or ParadeDB BM25 for FTS.
**Rebuild-on-upgrade = near-zero lock-in:** the index is derived from descriptions/events, so switching
backends later (e.g. after a MariaDB 11.8 upgrade) is `rebuild()`, not a data migration.

## Insertion points (real zm-api files)
- `src/service/zmnext/ingest.rs` — `EventIngestor`: add embed-at-ingest.
- `src/configure/search.rs` (new) + `src/server/state.rs` — config + `AppState`.
- `src/service/search/{store.rs,router.rs,mod.rs}` (new) — VectorStore + impls + router.
- migration crate — `zmnext_event_vectors` table + FTS index.
- `src/routes/` + `src/handlers/` — endpoints; reuse `MonitorScope` + `media_auth_middleware`.
- Embedding/rerank/router: NEW `EmbeddingProvider`/`RerankProvider`/`ChatProvider` over `reqwest` to
  the local `embed_url`/`rerank_url`/`router_url` (no `ort`). Decode crop JPEGs with the `image` crate.

## Config / dependencies (NEW deps this feature adds)
- **Already present (reuse):** `reqwest` (provider HTTP), `image` (crop JPEG decode), `sea-orm` +
  `sea-orm-migration` (mysql/pg), `tokio` (`spawn_blocking`), DashMap.
- **Add:** a sqlite driver for the vector store — `rusqlite` (with `bundled` + `load_extension`) or
  `sqlx` with the `sqlite` feature — plus the **sqlite-vec** extension (load at runtime), since sea-orm
  here is mysql+pg only. This is the one genuinely new native dependency.
- **Do NOT add `ort`/ONNX/tokenizers** — inference is external HTTP (Option 1). No vector-DB server
  (Qdrant/Milvus) — local-first. The embeddings/rerank/router model servers are operator-run (same
  posture as zm-next's existing local mlx-vlm sidecar).

## Testing / acceptance
- Backend probe unit tests (mariadb-native vs fallback selection) with a functional `VECTOR(3)` probe.
- ingest→upsert→search round-trip on sqlite-vec (no server needed in CI).
- Router: "how many cars today" → SQL count path (not LLM tally); "when did someone bring a package"
  → hybrid + grounded answer with verified `event_id` citations.
- ACL: a caller without a monitor's scope never sees its events in results.

## Open decisions for the zm-api side
- Which local model server to run behind `embed_url`/`rerank_url`/`router_url` (TEI vs llama.cpp vs
  Ollama vs vllm-mlx) and which models (e.g. bge-small vs Qwen3-Embedding-0.6B); whether `image_embed`
  is on at launch.
- Whether zm-next should emit the image embedding (emit-ingredients) vs zm-api computing it — the trait
  is agnostic; decide per compute budget.
- `expires_at`/retention for the vector index vs source-event retention (it's rebuildable).
- Router LLM: local-only vs the existing cloud-provider fallback.
