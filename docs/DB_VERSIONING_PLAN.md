# Database versioning: baseline 1.39.1, mirrored upgrades, Postgres

**Status: Active** — started 2026-09-13. Phases 1–3 are implemented and proven on MySQL (`schema-parity` against 1.39.33 and `upgrade-parity` from 1.38.3 both pass); phase 4 (Postgres) passes locally — `migrator up` on Postgres 16 yields the same 56 tables/729 columns as MySQL; phase 5 (entities) is proven — the full test suite and the DB-backed suites pass against the 1.39.33 entities. Replaces the frozen-cutover model
described in `src/migration/mod.rs` and issue #48.

## The requirement

Bring a new database server online — MySQL/MariaDB *or* Postgres — and have
zm-api produce, at any time, the equivalent of ZoneMinder's `zm_create.sql`
at the start of the 1.39 series and then every schema upgrade upstream has
shipped since. The same
thing `zmupdate.pl` does for an existing MySQL install, but from zm-api's own
migration system and backend-neutral. The purpose is to demonstrate that
zm-api can own ZoneMinder's schema lifecycle; it is optional for now, and
nothing changes for an existing MySQL install.

## What exists today, and why it does not meet that

- `m00000000_000001_zm_baseline` creates the schema **at 1.39.17**, generated
  once from a vendored `zm_create.sql.in`. Every upstream schema change means
  regenerating it (issue #48 — upstream is at 1.39.33 as of this writing,
  sixteen updates on).
- Existing MySQL installs are upgraded by `migrator bridge`, which runs the
  vendored `zm_update-*.sql` files verbatim (`legacy_bridge/chain.rs`) up to
  the cutover version, then stamps the baseline. Those files are MySQL client
  scripts (`PREPARE`, `INFORMATION_SCHEMA` probes, `DELIMITER`) and cannot run
  on Postgres.
- The baseline generator already maps MySQL types for Postgres; the summary
  triggers are MySQL-only ("the Postgres port lands with the Postgres schema
  work"), and `stamp.rs` uses `DATABASE()`, `UNIX_TIMESTAMP()` and `?`
  placeholders, so nothing past `migrator up` has ever run on Postgres.

## Design

```
                fresh MySQL / Postgres                existing MySQL install
                ──────────────────────                ──────────────────────
 migrator up:   baseline (= zm_create @ 1.39.1)       migrator bridge:
                m_1_39_2  … m_1_39_33  (SeaORM,         raw zm_update chain to the
                  portable, one per upstream update)    latest vendored version,
                m2026…_event_synopsis                   then stamp baseline +
                m2026…_monitor_pipeline                  every m_1_39_x ≤ that version
```

1. **Baseline at 1.39.1.** `db/baseline-1.39.1/` holds `zm_create.sql.in` and
   the fragments it sources, taken from upstream commit `e6ace6fc` — the
   commit that added `zm_update-1.39.1.sql`, so the create script there *is*
   the 1.39.1 schema. (1.39.1 is the first version with an update file;
   there is no 1.39.0 tag and nothing identifies a 1.39.0 schema.)
   `scripts/gen_baseline_migration.py --snapshot db/baseline-1.39.1` reads
   that directory instead of the repo-root `zm_create.sql.in`. The baseline is
   regenerated **once** and then never moves: it is the start of the 1.39
   series, not a moving cutover.

2. **One SeaORM migration per upstream update**, `m00000001_39_<n>_<slug>`,
   mirroring `db/zm_update-1.39.<n>.sql` in sea-query DDL and portable SQL:
   add/drop index, add/modify column, create table, seed rows. Where an update
   is MySQL-only (the 1.39.26 trigger rewrite) the migration branches on
   backend; Postgres gets PL/pgSQL equivalents where they exist and a logged
   skip where they do not. Each migration is written to be re-runnable
   (existence checks) so a stamping mistake cannot break an install. Each one
   is reviewed against the upstream file it mirrors — the file is quoted in
   the migration's doc comment.

3. **Stamping records the version reached.** After `bridge` walks the raw
   chain to version V, it records the baseline *and* every `m_1_39_x` with
   x ≤ V as applied, so `up` then runs only what is newer. The
   "schema already current" stamp for externally created databases does the
   same for the version its marker column proves. The version is stored where
   ZoneMinder stores it: `Config.ZM_DYN_DB_VERSION`.

4. **Mirroring upstream is the routine.** When upstream ships
   `zm_update-1.39.34.sql`: copy it into `db/legacy/` (raw chain, MySQL
   installs), write `m00000001_39_34_…` (fresh installs, both backends),
   regenerate entities, run the parity jobs. No baseline regeneration.

5. **Postgres is `migrator up -u postgres://…`.** Nothing else. A CI job runs
   it against the Postgres service and checks that every table and column
   present on a fresh MySQL exists on Postgres (names, not types — the type
   mapping is the generator's documented decisions). The API itself stays
   MySQL-only until the entities and query layer are exercised on Postgres;
   that is later work, and out of scope here.

6. **"Assuming control"** has a concrete meaning: the day the ZoneMinder fork
   stops shipping `zm_update` files, the mirror stops and new schema changes
   are written as zm-api migrations only, on both backends.

## What the parity jobs prove afterwards

- `schema-parity`: `migrator up` on an empty MySQL == the latest vendored
  `zm_create.sql.in`. This is now the proof that the 32 migrations reproduce
  upstream exactly.
- `upgrade-parity` (from 1.34.26 / 1.36.35 / 1.38.3): `bridge` + stamp + `up`
  == fresh `up`. Unchanged in intent.
- `postgres-schema` (new): `migrator up` on Postgres has every table and
  column the MySQL fresh schema has.

## Phases

| # | Work | Verifies | State |
|---|---|---|---|
| 1 | Vendor 1.39.1 snapshot; generator takes a path; regenerate baseline at 1.39.1 | `migrator up` on MySQL == 1.39.1 create (parity run against the snapshot) | proven (MySQL) |
| 2 | Mirror `zm_update-1.39.18…33` into `db/legacy/`; regenerate chain; write `m_1_39_2…33` | `schema-parity` against latest create | proven (MySQL) |
| 3 | Version-aware stamping in `bridge` and `stamp.rs`; migrator records `ZM_DYN_DB_VERSION` | `upgrade-parity` matrix; unit tests on the stamp set for a given version | proven (MySQL) |
| 4 | Backend-neutral `stamp.rs`; Postgres CI job | `postgres-schema` | proven locally (`postgres-schema.sh`: 729 columns match) |
| 5 | Regenerate entities at 1.39.33; fix code for changed columns (`FrameSkip` gone, Janus fields non-null, new `Controls`/`Monitors` columns) | `cargo test`, DB suites | proven |

Phase 5 is what closes #48. Phases 1–4 are the demonstration.

## What Postgres needed

Getting `migrator up` through on Postgres surfaced four systematic
differences, each fixed once, in the generator or the shared helpers, rather
than per migration:

- **Unsigned serial ids.** sea-query refuses `unsigned` + `auto_increment`
  on Postgres; ids are `serial`/`bigserial` there and keep their unsigned
  type on MySQL.
- **Index names are per-schema.** ZoneMinder reuses `Name`, `Encoder`, etc.
  across tables; on Postgres the index is prefixed with its table unless the
  name already starts with it. MySQL keeps upstream's exact name.
- **Typed seed values.** MySQL coerces a quoted `'1'` into any column;
  Postgres binds it as text. Seed values are typed from their column:
  numbers, booleans, and enum members cast with `as_enum`.
- **No inline indexes in CREATE TABLE.** MySQL's `KEY ...` syntax has no
  Postgres form; new-table migrations create indexes afterwards there.

## Out of scope

- Running the API against Postgres (query layer, `NOW()`/timezone semantics,
  the `?timezone=SYSTEM` connection option — all MySQL-specific today).
- Postgres equivalents of the Event_Summaries triggers beyond what already
  exists in the generator's notes; they are logged as skipped on Postgres.
