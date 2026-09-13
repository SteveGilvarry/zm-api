//! Mirrors `db/legacy/zm_update-1.39.28.sql`: redundant single-column indexes
//! dropped across six tables. `Stats.MonitorId`/`ZoneId` are kept where a
//! foreign key still needs them, as upstream checks.

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::{ConnectionTrait, DatabaseBackend, Statement};

use super::helpers::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// MySQL requires an index on a foreign-key column; Postgres does not.
async fn fk_needs_index(m: &SchemaManager<'_>, table: &str, column: &str) -> Result<bool, DbErr> {
    if backend(m) != DatabaseBackend::MySql {
        return Ok(false);
    }
    Ok(m.get_connection()
        .query_one(Statement::from_sql_and_values(
            DatabaseBackend::MySql,
            "SELECT 1 FROM information_schema.KEY_COLUMN_USAGE WHERE TABLE_SCHEMA = DATABASE() \
             AND TABLE_NAME = ? AND COLUMN_NAME = ? AND REFERENCED_TABLE_NAME IS NOT NULL",
            [table.into(), column.into()],
        ))
        .await?
        .is_some())
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        drop_index_if_present(m, "Logs", "TimeKey").await?;
        for col in ["MonitorId", "ZoneId"] {
            if !fk_needs_index(m, "Stats", col).await? {
                drop_index_if_present(m, "Stats", col).await?;
            }
        }
        drop_index_if_present(m, "EncoderTemplates", "Encoder").await?;
        drop_index_if_present(
            m,
            "Role_Groups_Permissions",
            "Role_Groups_Permissions_RoleId_idx",
        )
        .await?;
        drop_index_if_present(
            m,
            "Role_Monitors_Permissions",
            "Role_Monitors_Permissions_RoleId_idx",
        )
        .await?;
        drop_index_if_present(m, "Monitor_Status", "Monitor_Status_UpdatedOn_idx").await
    }

    async fn down(&self, _: &SchemaManager) -> Result<(), DbErr> {
        Err(DbErr::Migration(
            "upstream ZoneMinder schema updates are not reversible".into(),
        ))
    }
}
