//! Mirrors `db/legacy/zm_update-1.39.39.sql`: `Controls.Type` is dropped
//! (nothing ever read it), which leaves the shipped Pelco-D and Pelco-P
//! controls duplicated. For each of those two names the earliest row is kept,
//! monitors pointing at a later copy are moved to it, and the copies deleted.
//! Pelco-D and Pelco-P are not merged with each other: their Protocol differs.

use std::collections::BTreeMap;

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::{ConnectionTrait, DatabaseBackend};

use super::helpers::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        drop_column_if_present(m, "Controls", "Type").await?;
        if backend(m) == DatabaseBackend::Postgres {
            run_raw(m, "DROP TYPE IF EXISTS \"controls_type\"").await?;
        }

        let rows = m
            .get_connection()
            .query_all_raw(
                backend(m).build(
                    Query::select()
                        .expr_as(as_i64(m, "Id"), c("Id"))
                        .columns([c("Name"), c("Protocol")])
                        .from(t("Controls"))
                        .and_where(Expr::col(c("Name")).is_in(["Pelco-D", "Pelco-P"]))
                        .order_by(c("Id"), Order::Asc),
                ),
            )
            .await?;
        // (Name, Protocol) -> ids, ascending; Protocol compared null-safely.
        let mut groups: BTreeMap<(String, Option<String>), Vec<i64>> = BTreeMap::new();
        for row in rows {
            let id: i64 = row.try_get("", "Id")?;
            let name: String = row.try_get("", "Name")?;
            let protocol: Option<String> = row.try_get("", "Protocol")?;
            groups.entry((name, protocol)).or_default().push(id);
        }
        for ids in groups.values() {
            let Some((&keep, dups)) = ids.split_first() else {
                continue;
            };
            for &dup in dups {
                exec_stmt(
                    m,
                    Query::update()
                        .table(t("Monitors"))
                        .value(c("ControlId"), keep)
                        .and_where(Expr::col(c("ControlId")).eq(dup)),
                )
                .await?;
                exec_stmt(
                    m,
                    Query::delete()
                        .from_table(t("Controls"))
                        .and_where(Expr::col(c("Id")).eq(dup)),
                )
                .await?;
            }
        }
        Ok(())
    }

    async fn down(&self, _: &SchemaManager) -> Result<(), DbErr> {
        Err(DbErr::Migration(
            "upstream ZoneMinder schema updates are not reversible".into(),
        ))
    }
}
