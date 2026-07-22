//! Legacy upgrade bridge (issue #12, migration system phase 2).
//!
//! Brings an existing ZoneMinder database (>= 1.34.0) into the SeaORM
//! migration system:
//!
//! 1. Detect a legacy DB: `Config` table present, baseline migration not
//!    recorded in `seaql_migrations`.
//! 2. Read `ZM_DYN_DB_VERSION` and apply every embedded `zm_update-*.sql`
//!    with a greater version, in version order, updating
//!    `ZM_DYN_DB_VERSION` after each file so an interrupted run resumes.
//! 3. Converge triggers: legacy chains leave era-dependent triggers; drop
//!    and recreate the current set from the baseline.
//! 4. Stamp `m00000000_000001_zm_baseline` as applied WITHOUT executing it,
//!    then run the normal migration set (zm-api-owned tables still apply).
//!
//! MySQL/MariaDB only by definition - legacy ZoneMinder databases have no
//! other backend. Versions below the 1.34.0 support floor are refused
//! (upgrade with ZoneMinder's zmupdate.pl first); the vendored chain is
//! pruned to 1.34.0+ accordingly.

mod chain;

use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseConnection, DbErr, Statement, Value};
use tracing::info;

use super::m00000000_000001_zm_baseline::triggers;
use super::Migrator;
use sea_orm_migration::MigratorTrait;

pub const BASELINE_VERSION: &str = "m00000000_000001_zm_baseline";
const FLOOR_VERSION: &str = "1.34.0";
const ADVISORY_LOCK: &str = "zm_api_migration";

/// Numeric dotted-version ordering ("1.36.9" < "1.36.12" < "1.37.0").
fn version_key(v: &str) -> Vec<u64> {
    v.split('.').filter_map(|p| p.parse().ok()).collect()
}

fn version_lt(a: &str, b: &str) -> bool {
    version_key(a) < version_key(b)
}

/// Split a legacy update file into executable statements.
///
/// Handles the mysql-client `DELIMITER` command (used by the 1.31.x files
/// that define stored procedures) and quoted strings/backticked identifiers.
/// Comments are preserved inside statements (the server ignores them); pure
/// comment/empty chunks are dropped.
/// Remove non-executable `/* ... */` block comments (quote-aware), keeping
/// MySQL executable hints (`/*! ... */`). The legacy files comment out whole
/// statement groups with block comments containing semicolons, which would
/// otherwise confuse statement splitting.
fn strip_block_comments(sql: &str) -> String {
    let mut out = String::with_capacity(sql.len());
    let bytes = sql.as_bytes();
    let mut i = 0;
    let mut quote: Option<u8> = None;
    while i < bytes.len() {
        let c = bytes[i];
        match quote {
            Some(q) => {
                out.push(c as char);
                if c == b'\\' && q == b'\'' && i + 1 < bytes.len() {
                    out.push(bytes[i + 1] as char);
                    i += 2;
                    continue;
                }
                if c == q {
                    quote = None;
                }
            }
            None => {
                if c == b'\'' || c == b'"' || c == b'`' {
                    quote = Some(c);
                    out.push(c as char);
                } else if c == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
                    if i + 2 < bytes.len() && bytes[i + 2] == b'!' {
                        out.push('/'); // executable hint, keep verbatim
                    } else {
                        // Skip to the closing */
                        if let Some(end) = sql[i + 2..].find("*/") {
                            i += 2 + end + 2;
                            continue;
                        }
                        break; // unterminated comment: drop the rest
                    }
                } else {
                    out.push(c as char);
                }
            }
        }
        i += 1;
    }
    out
}

fn split_sql_statements(sql: &str) -> Vec<String> {
    let sql = strip_block_comments(sql);
    let mut out = Vec::new();
    let mut delim = ";".to_string();
    let mut buf = String::new();

    for raw_line in sql.lines() {
        let trimmed = raw_line.trim();
        // DELIMITER is a client command, always alone on a line.
        if buf.trim().is_empty() {
            if let Some(rest) = trimmed
                .strip_prefix("DELIMITER")
                .or_else(|| trimmed.strip_prefix("delimiter"))
            {
                let d = rest.trim();
                if !d.is_empty() {
                    delim = d.to_string();
                }
                continue;
            }
            // Drop pure comment lines between statements.
            if trimmed.starts_with("--") || trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
        }
        buf.push_str(raw_line);
        buf.push('\n');

        if ends_with_delimiter_outside_quotes(&buf, &delim) {
            let stmt = buf.trim().trim_end_matches(delim.as_str()).trim();
            if !stmt.is_empty() {
                out.push(stmt.to_string());
            }
            buf.clear();
        }
    }
    let tail = buf.trim();
    if !tail.is_empty() {
        let stmt = tail.trim_end_matches(delim.as_str()).trim();
        if !stmt.is_empty() {
            out.push(stmt.to_string());
        }
    }
    out
}

/// True when the accumulated buffer ends with the delimiter at quote depth 0
/// (ignoring trailing whitespace and trailing `-- comments` on the last line).
fn ends_with_delimiter_outside_quotes(buf: &str, delim: &str) -> bool {
    // Strip quoted regions, then check the tail.
    let mut cleaned = String::with_capacity(buf.len());
    let mut chars = buf.chars().peekable();
    let mut quote: Option<char> = None;
    while let Some(c) = chars.next() {
        match quote {
            Some(q) => {
                if c == '\\' && q == '\'' {
                    chars.next();
                } else if c == q {
                    quote = None;
                }
            }
            None => {
                if c == '\'' || c == '"' || c == '`' {
                    quote = Some(c);
                } else {
                    cleaned.push(c);
                }
            }
        }
    }
    if quote.is_some() {
        return false; // still inside a string literal
    }
    cleaned.trim_end().ends_with(delim)
}

async fn scalar(
    conn: &DatabaseConnection,
    sql: &str,
    params: Vec<Value>,
) -> Result<Option<String>, DbErr> {
    let row = conn
        .query_one(Statement::from_sql_and_values(
            DatabaseBackend::MySql,
            sql,
            params,
        ))
        .await?;
    match row {
        Some(r) => r.try_get_by_index::<Option<String>>(0),
        None => Ok(None),
    }
}

async fn scalar_i64(
    conn: &DatabaseConnection,
    sql: &str,
    params: Vec<Value>,
) -> Result<Option<i64>, DbErr> {
    let row = conn
        .query_one(Statement::from_sql_and_values(
            DatabaseBackend::MySql,
            sql,
            params,
        ))
        .await?;
    match row {
        Some(r) => r.try_get_by_index::<Option<i64>>(0),
        None => Ok(None),
    }
}

async fn table_exists(conn: &DatabaseConnection, table: &str) -> Result<bool, DbErr> {
    Ok(scalar(
        conn,
        "SELECT TABLE_NAME FROM information_schema.TABLES \
         WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = ?",
        vec![table.into()],
    )
    .await?
    .is_some())
}

async fn baseline_applied(conn: &DatabaseConnection) -> Result<bool, DbErr> {
    if !table_exists(conn, "seaql_migrations").await? {
        return Ok(false);
    }
    Ok(scalar(
        conn,
        "SELECT version FROM seaql_migrations WHERE version = ?",
        vec![BASELINE_VERSION.into()],
    )
    .await?
    .is_some())
}

async fn db_version(conn: &DatabaseConnection) -> Result<Option<String>, DbErr> {
    if !table_exists(conn, "Config").await? {
        return Ok(None);
    }
    scalar(
        conn,
        "SELECT Value FROM Config WHERE Name = 'ZM_DYN_DB_VERSION'",
        vec![],
    )
    .await
}

async fn set_config(conn: &DatabaseConnection, name: &str, value: &str) -> Result<(), DbErr> {
    conn.execute(Statement::from_sql_and_values(
        DatabaseBackend::MySql,
        "UPDATE Config SET Value = ? WHERE Name = ?",
        vec![value.into(), name.into()],
    ))
    .await?;
    Ok(())
}

/// Run the full bridge. Errors are surfaced verbatim from the failing
/// statement so the operator can see exactly which legacy patch broke.
pub async fn run(conn: &DatabaseConnection) -> Result<(), DbErr> {
    if conn.get_database_backend() != DatabaseBackend::MySql {
        return Err(DbErr::Custom(
            "the legacy bridge only applies to MySQL/MariaDB databases; \
             fresh installs on other backends use `migrator up`"
                .into(),
        ));
    }

    // Serialize against other zm-api instances (multi-server).
    let got = scalar_i64(conn, "SELECT GET_LOCK(?, 300)", vec![ADVISORY_LOCK.into()]).await?;
    if got != Some(1) {
        return Err(DbErr::Custom(
            "another migration is in progress (GET_LOCK timeout)".into(),
        ));
    }
    let result = run_locked(conn).await;
    let _ = scalar_i64(conn, "SELECT RELEASE_LOCK(?)", vec![ADVISORY_LOCK.into()]).await;
    result
}

async fn run_locked(conn: &DatabaseConnection) -> Result<(), DbErr> {
    if baseline_applied(conn).await? {
        info!("baseline already recorded; running any pending migrations");
        return Migrator::up(conn, None).await;
    }

    let version = match db_version(conn).await? {
        Some(v) => v,
        None => {
            return Err(DbErr::Custom(
                "no ZoneMinder Config table found - this is not a legacy \
                 ZoneMinder database. Use `migrator up` for a fresh install."
                    .into(),
            ));
        }
    };

    if version_lt(&version, FLOOR_VERSION) {
        return Err(DbErr::Custom(format!(
            "database version {version} is below the bridge floor \
             {FLOOR_VERSION}; upgrade with ZoneMinder's zmupdate.pl first"
        )));
    }

    info!("legacy database at version {version}; walking update chain");
    for (v, sql) in chain::CHAIN {
        if !version_lt(&version, v) {
            continue;
        }
        info!("applying zm_update-{v}.sql");
        for stmt in split_sql_statements(sql) {
            conn.execute_unprepared(&stmt).await.map_err(|e| {
                DbErr::Custom(format!("zm_update-{v}.sql failed: {e}\nstatement: {stmt}"))
            })?;
        }
        set_config(conn, "ZM_DYN_DB_VERSION", v).await?;
    }

    info!("converging schema drift left by the legacy chain");
    converge_schema(conn).await?;

    info!("converging triggers onto the current set");
    for (name, create) in triggers::mysql_triggers() {
        conn.execute_unprepared(&format!("DROP TRIGGER IF EXISTS `{name}`"))
            .await?;
        conn.execute_unprepared(create).await?;
    }

    info!("stamping baseline migration as applied");
    Migrator::install(conn).await?;
    conn.execute(Statement::from_sql_and_values(
        DatabaseBackend::MySql,
        "INSERT INTO seaql_migrations (version, applied_at) VALUES (?, UNIX_TIMESTAMP())",
        vec![BASELINE_VERSION.into()],
    ))
    .await?;

    set_config(conn, "ZM_DYN_DB_VERSION", chain::CUTOVER_ZM_VERSION).await?;
    set_config(conn, "ZM_DYN_CURR_VERSION", chain::CUTOVER_ZM_VERSION).await?;

    info!("running remaining migrations");
    Migrator::up(conn, None).await
}

async fn index_exists(conn: &DatabaseConnection, table: &str, index: &str) -> Result<bool, DbErr> {
    Ok(scalar(
        conn,
        "SELECT INDEX_NAME FROM information_schema.STATISTICS \
         WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = ? AND INDEX_NAME = ? LIMIT 1",
        vec![table.into(), index.into()],
    )
    .await?
    .is_some())
}

async fn column_exists(
    conn: &DatabaseConnection,
    table: &str,
    column: &str,
) -> Result<bool, DbErr> {
    Ok(scalar(
        conn,
        "SELECT COLUMN_NAME FROM information_schema.COLUMNS \
         WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = ? AND COLUMN_NAME = ?",
        vec![table.into(), column.into()],
    )
    .await?
    .is_some())
}

/// Reconcile drift between chain-upgraded and fresh-created schemas.
///
/// The legacy zm_update chain and zm_create.sql.in diverged in places over
/// the years; zmupdate.pl never reconciled them, so real installs differ
/// from fresh installs. Every action is conditional (information_schema
/// checked first) so the pass is idempotent and era-independent. The
/// convergence target is always the fresh baseline schema, verified by
/// scripts/upgrade-parity.sh.
async fn converge_schema(conn: &DatabaseConnection) -> Result<(), DbErr> {
    // -- Indexes the chain never created ------------------------------------
    for (table, index, ddl) in [
        (
            "Logs",
            "TimeKey",
            "CREATE INDEX `TimeKey` ON `Logs` (`TimeKey`)",
        ),
        (
            "Monitor_Status",
            "Monitor_Status_UpdatedOn_idx",
            "CREATE INDEX `Monitor_Status_UpdatedOn_idx` ON `Monitor_Status` (`UpdatedOn`)",
        ),
        (
            "Role_Groups_Permissions",
            "Role_Groups_Permissions_RoleId_idx",
            "CREATE INDEX `Role_Groups_Permissions_RoleId_idx` ON `Role_Groups_Permissions` (`RoleId`)",
        ),
        (
            "Role_Monitors_Permissions",
            "Role_Monitors_Permissions_RoleId_idx",
            "CREATE INDEX `Role_Monitors_Permissions_RoleId_idx` ON `Role_Monitors_Permissions` (`RoleId`)",
        ),
        (
            "Stats",
            "EventId_ZoneId",
            "CREATE INDEX `EventId_ZoneId` ON `Stats` (`EventId`, `ZoneId`)",
        ),
        (
            "Groups",
            "ParentId",
            "CREATE INDEX `ParentId` ON `Groups` (`ParentId`)",
        ),
    ] {
        if table_exists(conn, table).await? && !index_exists(conn, table, index).await? {
            info!("adding missing index {table}.{index}");
            conn.execute_unprepared(ddl).await?;
        }
    }

    // Stats.EventId is superseded by the composite EventId_ZoneId.
    if index_exists(conn, "Stats", "EventId").await? {
        info!("dropping superseded index Stats.EventId");
        conn.execute_unprepared("DROP INDEX `EventId` ON `Stats`")
            .await?;
    }

    // -- Index names that differ between chain and create -------------------
    for (table, from, to) in [
        (
            "Snapshots_Events",
            "Snapshot_Events_SnapshotId_idx",
            "Snapshots_Events_SnapshotId_idx",
        ),
        ("Users", "Users_RoleId_idx", "RoleId"),
        // 1.34-era names before the StartTime -> StartDateTime column rename
        ("Events", "Events_StartTime_idx", "Events_StartDateTime_idx"),
        (
            "Events",
            "Events_EndTime_DiskSpace",
            "Events_EndDateTime_DiskSpace",
        ),
        (
            "Events_Hour",
            "Events_Hour_StartTime_idx",
            "Events_Hour_StartDateTime_idx",
        ),
        (
            "Events_Day",
            "Events_Day_StartTime_idx",
            "Events_Day_StartDateTime_idx",
        ),
        (
            "Events_Week",
            "Events_Week_StartTime_idx",
            "Events_Week_StartDateTime_idx",
        ),
        (
            "Events_Month",
            "Events_Month_StartTime_idx",
            "Events_Month_StartDateTime_idx",
        ),
    ] {
        if index_exists(conn, table, from).await? && !index_exists(conn, table, to).await? {
            info!("renaming index {table}.{from} -> {to}");
            conn.execute_unprepared(&format!(
                "ALTER TABLE `{table}` RENAME INDEX `{from}` TO `{to}`"
            ))
            .await?;
        }
    }

    // -- Columns added by the chain but later removed from zm_create --------
    // (unused by any current code; fresh installs never have them)
    for (table, column) in [
        ("Monitors", "Janus_RTSP_User"),
        ("Monitors", "Janus_Use_RTSP_Restream"),
    ] {
        if column_exists(conn, table, column).await? {
            info!("dropping orphaned column {table}.{column}");
            conn.execute_unprepared(&format!("ALTER TABLE `{table}` DROP COLUMN `{column}`"))
                .await?;
        }
    }

    // -- Column definitions where chain and create disagree -----------------
    // MODIFY to the fresh definition; metadata-only or trivially safe.
    for (table, column, ddl) in [
        (
            "Monitors",
            "Janus_Profile_Override",
            "ALTER TABLE `Monitors` MODIFY `Janus_Profile_Override` varchar(30) NOT NULL default ''",
        ),
        (
            "Monitors",
            "Janus_RTSP_Session_Timeout",
            "ALTER TABLE `Monitors` MODIFY `Janus_RTSP_Session_Timeout` int NOT NULL default 0",
        ),
        (
            "Monitors",
            "MQTT_Subscriptions",
            "ALTER TABLE `Monitors` MODIFY `MQTT_Subscriptions` varchar(255) default ''",
        ),
        (
            "Monitors",
            "OutputCodecName",
            "ALTER TABLE `Monitors` MODIFY `OutputCodecName` varchar(32) NOT NULL default 'auto'",
        ),
        (
            "Monitors",
            "OutputContainer",
            "ALTER TABLE `Monitors` MODIFY `OutputContainer` enum('auto','mp4','mkv','webm') default NULL",
        ),
        (
            "ZonePresets",
            "Units",
            "ALTER TABLE `ZonePresets` MODIFY `Units` enum('Pixels','Percent') NOT NULL default 'Percent'",
        ),
        // 1.34-era enums never gained 'VNC' via the chain
        (
            "Controls",
            "Type",
            "ALTER TABLE `Controls` MODIFY `Type` enum('Local','Remote','File','Ffmpeg','Libvlc','cURL','WebSite','NVSocket','VNC') NOT NULL default 'Local'",
        ),
        (
            "MonitorPresets",
            "Type",
            "ALTER TABLE `MonitorPresets` MODIFY `Type` enum('Local','Remote','File','Ffmpeg','Libvlc','cURL','WebSite','NVSocket','VNC') NOT NULL default 'Local'",
        ),
        // 1.34-era defaults the chain never updated to match zm_create
        (
            "Monitors",
            "ImageBufferCount",
            "ALTER TABLE `Monitors` MODIFY `ImageBufferCount` smallint unsigned NOT NULL default 3",
        ),
        (
            "Monitors",
            "StreamReplayBuffer",
            "ALTER TABLE `Monitors` MODIFY `StreamReplayBuffer` int unsigned NOT NULL default 0",
        ),
        (
            "Monitors",
            "WarmupCount",
            "ALTER TABLE `Monitors` MODIFY `WarmupCount` smallint unsigned NOT NULL default 0",
        ),
    ] {
        if column_exists(conn, table, column).await? {
            conn.execute_unprepared(ddl).await?;
        }
    }

    // OutputCodec became NOT NULL in zm_create; clear NULLs before MODIFY.
    if column_exists(conn, "Monitors", "OutputCodec").await? {
        conn.execute_unprepared(
            "UPDATE `Monitors` SET `OutputCodec` = 0 WHERE `OutputCodec` IS NULL",
        )
        .await?;
        conn.execute_unprepared(
            "ALTER TABLE `Monitors` MODIFY `OutputCodec` int unsigned NOT NULL default 0",
        )
        .await?;
    }

    // -- Tables created by the chain under the server-default collation -----
    // Fresh creates inherit utf8mb4_unicode_ci from the connection; chain
    // creates on newer MariaDB got utf8mb4_uca1400_ai_ci.
    conn.execute_unprepared(
        "ALTER TABLE `EncoderTemplates` CONVERT TO CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci",
    )
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_ordering_is_numeric_not_lexical() {
        assert!(version_lt("1.36.9", "1.36.12"));
        assert!(version_lt("1.36.12", "1.37.0"));
        assert!(version_lt("1.9.0", "1.34.0"));
        assert!(!version_lt("1.26.0", "1.26.0"));
        assert!(!version_lt("1.37.1", "1.37.0"));
    }

    #[test]
    fn chain_is_strictly_ascending_and_nonempty() {
        assert!(chain::CHAIN.len() > 100);
        for w in chain::CHAIN.windows(2) {
            assert!(
                version_lt(w[0].0, w[1].0),
                "chain not ascending: {} then {}",
                w[0].0,
                w[1].0
            );
        }
    }

    #[test]
    fn cutover_is_at_or_past_the_last_chain_entry() {
        let last = chain::CHAIN.last().unwrap().0;
        assert!(!version_lt(chain::CUTOVER_ZM_VERSION, last));
    }

    #[test]
    fn every_chain_file_splits_cleanly() {
        // Some files are comment-only ("no updates from X to Y") - that's
        // fine; what must never happen is DELIMITER leaking into a statement.
        let mut total = 0;
        for (v, sql) in chain::CHAIN {
            for s in split_sql_statements(sql) {
                assert!(
                    !s.to_uppercase().starts_with("DELIMITER"),
                    "unstripped DELIMITER in {v}: {s}"
                );
                total += 1;
            }
        }
        assert!(total > 500, "suspiciously few statements: {total}");
    }

    #[test]
    fn delimiter_era_files_split_correctly() {
        // 1.35.13/1.35.14 redefine triggers via DELIMITER blocks.
        let sql = chain::CHAIN
            .iter()
            .find(|(v, _)| *v == "1.35.13")
            .expect("1.35.13 present")
            .1;
        let stmts = split_sql_statements(sql);
        assert!(stmts
            .iter()
            .any(|s| s.to_uppercase().contains("CREATE TRIGGER")));
        assert!(stmts.iter().all(|s| !s.trim().is_empty()));
    }

    #[test]
    fn splitter_handles_delimiter_blocks() {
        let sql = "DROP PROCEDURE IF EXISTS p;\n\
                   DELIMITER //\n\
                   CREATE PROCEDURE p()\n\
                   BEGIN\n\
                     UPDATE a SET b = 1;\n\
                     UPDATE c SET d = 2;\n\
                   END //\n\
                   DELIMITER ;\n\
                   CALL p();\n\
                   DROP PROCEDURE p;\n";
        let stmts = split_sql_statements(sql);
        assert_eq!(stmts.len(), 4);
        assert!(stmts[1].contains("UPDATE c SET d = 2;"));
        assert!(stmts[1].ends_with("END"));
        assert_eq!(stmts[2], "CALL p()");
    }

    #[test]
    fn splitter_ignores_semicolons_in_strings() {
        let stmts = split_sql_statements("INSERT INTO t VALUES ('a;b');\nSELECT 1;\n");
        assert_eq!(stmts.len(), 2);
        assert_eq!(stmts[0], "INSERT INTO t VALUES ('a;b')");
    }

    #[test]
    fn splitter_keeps_statement_internal_comment_lines() {
        // A comment line inside an open statement must not be dropped.
        let stmts = split_sql_statements("SELECT 1\n-- keep me\n+ 2;\n");
        assert_eq!(stmts.len(), 1);
        assert!(stmts[0].contains("keep me"));
    }

    #[test]
    fn baseline_version_matches_migration_name() {
        use sea_orm_migration::MigrationName;
        assert_eq!(
            super::super::m00000000_000001_zm_baseline::Migration.name(),
            BASELINE_VERSION
        );
    }

    #[test]
    fn trigger_set_is_the_generated_dozen() {
        let names: Vec<_> = triggers::mysql_triggers()
            .into_iter()
            .map(|(n, _)| n)
            .collect();
        assert_eq!(names.len(), 12);
        assert!(names.contains(&"Events_Hour_delete_trigger"));
        assert!(names.contains(&"Zone_Delete_Trigger"));
    }
}
