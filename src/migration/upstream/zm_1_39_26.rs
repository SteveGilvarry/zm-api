//! Mirrors `db/legacy/zm_update-1.39.26.sql`: the per-bucket cascade triggers
//! go, `Event_Summaries` is resynced from the events themselves, and the
//! trigger set is replaced with the current `db/triggers.sql`
//! (`triggers_1_39_26::mysql_triggers`, generated).
//!
//! Triggers are MySQL-only here, as in the baseline; Postgres runs the resync
//! and skips them, which is logged.

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::{ConnectionTrait, DatabaseBackend, Statement};

use super::helpers::*;
use super::triggers_1_39_26::mysql_triggers;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// Every trigger name any 1.39 install may carry; all are dropped before the
/// current set is created, so the result is the same whichever set the
/// database started with.
const RETIRED_TRIGGERS: [&str; 12] = [
    "Events_Hour_delete_trigger",
    "Events_Hour_update_trigger",
    "Events_Day_delete_trigger",
    "Events_Day_update_trigger",
    "Events_Week_delete_trigger",
    "Events_Week_update_trigger",
    "Events_Month_delete_trigger",
    "Events_Month_update_trigger",
    "event_update_trigger",
    "event_delete_trigger",
    "Zone_Insert_Trigger",
    "Zone_Delete_Trigger",
];

/// (bucket table, count column, bytes column) → Event_Summaries.
const BUCKETS: [(&str, &str, &str); 6] = [
    ("Events_Hour", "HourEvents", "HourEventDiskSpace"),
    ("Events_Day", "DayEvents", "DayEventDiskSpace"),
    ("Events_Week", "WeekEvents", "WeekEventDiskSpace"),
    ("Events_Month", "MonthEvents", "MonthEventDiskSpace"),
    (
        "Events_Archived",
        "ArchivedEvents",
        "ArchivedEventDiskSpace",
    ),
    ("Events", "TotalEvents", "TotalEventDiskSpace"),
];

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let conn = m.get_connection();
        let mysql = backend(m) == DatabaseBackend::MySql;

        if mysql {
            for name in RETIRED_TRIGGERS {
                conn.execute(Statement::from_string(
                    DatabaseBackend::MySql,
                    format!("DROP TRIGGER IF EXISTS `{name}`"),
                ))
                .await?;
            }
        }

        // Resync: every summarised monitor, each bucket's count and bytes.
        let monitors: Vec<i64> = conn
            .query_all(
                backend(m).build(
                    Query::select()
                        .expr_as(as_i64(m, "MonitorId"), Alias::new("id"))
                        .from(t("Event_Summaries")),
                ),
            )
            .await?
            .into_iter()
            .map(|r| r.try_get::<i64>("", "id"))
            .collect::<Result<_, _>>()?;
        for (table, count_col, bytes_col) in BUCKETS {
            let rows = conn
                .query_all(
                    backend(m).build(
                        Query::select()
                            .expr_as(as_i64(m, "MonitorId"), Alias::new("id"))
                            .expr_as(
                                Expr::col(c("MonitorId")).count().cast_as(cast_target(m)),
                                Alias::new("n"),
                            )
                            .expr_as(
                                Func::coalesce([
                                    Expr::col(c("DiskSpace")).sum(),
                                    Expr::val(0).into(),
                                ])
                                .cast_as(cast_target(m)),
                                Alias::new("bytes"),
                            )
                            .from(t(table))
                            .group_by_col(c("MonitorId")),
                    ),
                )
                .await?;
            let mut by_monitor = std::collections::HashMap::new();
            for r in rows {
                by_monitor.insert(
                    r.try_get::<i64>("", "id")?,
                    (r.try_get::<i64>("", "n")?, r.try_get::<i64>("", "bytes")?),
                );
            }
            for monitor_id in &monitors {
                let (n, bytes) = by_monitor.get(monitor_id).copied().unwrap_or((0, 0));
                exec_stmt(
                    m,
                    Query::update()
                        .table(t("Event_Summaries"))
                        .value(c(count_col), n)
                        .value(c(bytes_col), bytes)
                        .and_where(Expr::col(c("MonitorId")).eq(*monitor_id)),
                )
                .await?;
            }
        }

        if mysql {
            for (_, create) in mysql_triggers() {
                conn.execute(Statement::from_string(
                    DatabaseBackend::MySql,
                    create.to_string(),
                ))
                .await?;
            }
        } else {
            tracing::info!(
                "zm_1_39_26: summary triggers are MySQL-only; not created on this backend"
            );
        }
        Ok(())
    }

    async fn down(&self, _: &SchemaManager) -> Result<(), DbErr> {
        Err(DbErr::Migration(
            "upstream ZoneMinder schema updates are not reversible".into(),
        ))
    }
}

fn cast_target(m: &SchemaManager<'_>) -> Alias {
    match backend(m) {
        DatabaseBackend::MySql => Alias::new("SIGNED"),
        _ => Alias::new("BIGINT"),
    }
}
