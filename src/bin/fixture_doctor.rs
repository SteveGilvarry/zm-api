//! Fixture-leak doctor.
//!
//! Integration tests name every fixture row `Test_run_<unix_nanos><label>`
//! (see `tests/common/test_db.rs`). A clean run deletes them all via the
//! `RowGuard` RAII cleanup, but a crash, a `kill -9`, or a run against the
//! wrong database can still leave rows behind. This tool scans the leak-prone
//! tables for those rows and flags any whose embedded timestamp is older than
//! a threshold (default: 1 hour) — old enough that no in-flight test could
//! still own them.
//!
//! It is meant to run in CI (and locally) so a leak resurfaces visibly instead
//! of silently accumulating in a shared database.
//!
//! ```bash
//! # Report only (exit 1 if stale rows exist):
//! cargo run --bin fixture-doctor
//! # Custom age threshold (seconds):
//! cargo run --bin fixture-doctor -- --max-age-secs 7200
//! # Report AND delete the stale rows:
//! cargo run --bin fixture-doctor -- --delete
//! ```
//!
//! Connection target: `TEST_DATABASE_URL`/`DATABASE_URL` if set (matching the
//! integration-test harness in `tests/common/test_db.rs`), otherwise the app
//! config + `APP_DB__*` env overrides — so CI scans exactly the database the
//! tests wrote to.

use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

use sea_orm::{ConnectionTrait, Database, DbBackend, Statement};
use zm_api::constant::CONFIG;

/// Tables whose rows tests create with a `Test_`-prefixed name, paired with the
/// textual column that carries that name. Children are listed before parents so
/// a `--delete` pass removes them in foreign-key-safe order.
const SCANNED: &[(&str, &str)] = &[
    ("Events", "Name"),
    ("Zones", "Name"),
    ("ControlPresets", "Label"),
    ("Groups_Monitors", "MonitorId"), // join rows: no name, skipped by the LIKE
    ("TriggersX10", "MonitorId"),     // keyed by monitor: no name, skipped
    ("Monitors", "Name"),
    ("Groups", "Name"),
    ("Devices", "Name"),
    ("Filters", "Name"),
    ("Users", "Username"),
    ("MontageLayouts", "Name"),
    ("Reports", "Name"),
    ("States", "Name"),
    ("MonitorPresets", "Name"),
    ("ZonePresets", "Name"),
    ("Controls", "Name"),
    ("Manufacturers", "Name"),
    ("Servers", "Name"),
    ("Storage", "Name"),
    ("Tags", "Name"),
];

const DEFAULT_MAX_AGE_SECS: u64 = 3600;
const NANOS_PER_SEC: u128 = 1_000_000_000;

/// A fixture row that outlived the age threshold.
struct StaleRow {
    table: &'static str,
    column: &'static str,
    name: String,
    age_secs: u64,
}

/// Parse the unix-nanosecond stamp embedded in a `Test_run_<nanos><label>`
/// name. Returns `None` for names that don't follow the default pattern (e.g.
/// a custom `TEST_RUN_ID`), which we report separately rather than age.
fn parse_run_nanos(name: &str) -> Option<u128> {
    let rest = name.strip_prefix("Test_run_")?;
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return None;
    }
    digits.parse::<u128>().ok()
}

#[tokio::main]
async fn main() -> ExitCode {
    let mut max_age_secs = DEFAULT_MAX_AGE_SECS;
    let mut delete = false;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--delete" => delete = true,
            "--max-age-secs" => {
                max_age_secs = args
                    .next()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(DEFAULT_MAX_AGE_SECS);
            }
            "-h" | "--help" => {
                println!(
                    "fixture-doctor — flag/clean leaked Test_run_* fixture rows\n\n\
                     Options:\n  \
                     --max-age-secs <N>  age threshold in seconds (default {DEFAULT_MAX_AGE_SECS})\n  \
                     --delete            delete the stale rows instead of only reporting"
                );
                return ExitCode::SUCCESS;
            }
            other => {
                eprintln!("fixture-doctor: unknown argument {other:?}");
                return ExitCode::from(2);
            }
        }
    }

    // Prefer the same env the integration-test harness uses (see
    // tests/common/test_db.rs) so the doctor scans exactly the database the
    // tests wrote to; otherwise fall back to the app config + APP_DB__* env.
    let url = std::env::var("TEST_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .unwrap_or_else(|_| CONFIG.db.get_url());
    // Redact the password when echoing the target.
    let safe_url = redact_url(&url);
    println!("fixture-doctor: scanning {safe_url} (threshold: {max_age_secs}s)");

    let db = match Database::connect(&url).await {
        Ok(db) => db,
        Err(e) => {
            eprintln!("fixture-doctor: failed to connect: {e}");
            return ExitCode::from(2);
        }
    };

    let now_nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    let threshold_nanos = (max_age_secs as u128) * NANOS_PER_SEC;

    let mut stale: Vec<StaleRow> = Vec::new();
    let mut unparseable: Vec<(&str, String)> = Vec::new();
    let mut recent = 0u64;

    for &(table, column) in SCANNED {
        // Underscore is a LIKE wildcard, so escape it to match a literal `Test_`.
        let sql =
            format!("SELECT `{column}` AS name FROM `{table}` WHERE `{column}` LIKE 'Test\\_%'");
        let stmt = Statement::from_string(DbBackend::MySql, sql);
        let rows = match db.query_all(stmt).await {
            Ok(rows) => rows,
            Err(e) => {
                // A missing table/column is not fatal — schemas drift between
                // ZoneMinder versions. Note it and move on.
                eprintln!("fixture-doctor: skipping {table}.{column}: {e}");
                continue;
            }
        };
        for row in rows {
            let name = match row.try_get::<String>("", "name") {
                Ok(n) => n,
                Err(_) => continue,
            };
            match parse_run_nanos(&name) {
                Some(nanos) => {
                    let age_nanos = now_nanos.saturating_sub(nanos);
                    if age_nanos >= threshold_nanos {
                        stale.push(StaleRow {
                            table,
                            column,
                            name,
                            age_secs: (age_nanos / NANOS_PER_SEC) as u64,
                        });
                    } else {
                        recent += 1;
                    }
                }
                None => unparseable.push((table, name)),
            }
        }
    }

    if recent > 0 {
        println!("fixture-doctor: {recent} recent fixture row(s) within the threshold — likely an in-flight run, ignored.");
    }
    for (table, name) in &unparseable {
        println!("fixture-doctor: WARNING fixture-named row of unknown age in {table}: {name:?}");
    }

    if stale.is_empty() {
        println!("fixture-doctor: OK — no leaked fixture rows older than {max_age_secs}s.");
        return ExitCode::SUCCESS;
    }

    eprintln!(
        "fixture-doctor: found {} leaked fixture row(s) older than {max_age_secs}s:",
        stale.len()
    );
    for row in &stale {
        eprintln!(
            "  - {}.{} = {:?} (age ~{}s)",
            row.table, row.column, row.name, row.age_secs
        );
    }

    if !delete {
        eprintln!(
            "fixture-doctor: rerun with --delete to remove them, \
             or investigate why a test run leaked rows."
        );
        return ExitCode::FAILURE;
    }

    let mut deleted = 0u64;
    let mut failed = false;
    for row in &stale {
        let sql = format!("DELETE FROM `{}` WHERE `{}` = ?", row.table, row.column);
        let stmt = Statement::from_sql_and_values(
            DbBackend::MySql,
            sql,
            [sea_orm::Value::from(row.name.clone())],
        );
        match db.execute(stmt).await {
            Ok(res) => deleted += res.rows_affected(),
            Err(e) => {
                eprintln!(
                    "fixture-doctor: failed to delete {}.{} {:?}: {e}",
                    row.table, row.column, row.name
                );
                failed = true;
            }
        }
    }
    println!("fixture-doctor: deleted {deleted} leaked row(s).");
    if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

/// Replace the password in a `mysql://user:pass@host/db` URL with `***`.
fn redact_url(url: &str) -> String {
    let Some((scheme, rest)) = url.split_once("://") else {
        return url.to_string();
    };
    let Some((creds, tail)) = rest.split_once('@') else {
        return url.to_string();
    };
    let user = creds.split_once(':').map(|(u, _)| u).unwrap_or(creds);
    format!("{scheme}://{user}:***@{tail}")
}
