//! Mirrors `db/legacy/zm_update-1.39.18.sql`: composite index on `Server_Stats`.

use sea_orm_migration::prelude::*;

use super::helpers::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        add_index_if_missing(
            m,
            "Server_Stats",
            "Server_Stats_ServerId_idx",
            &["ServerId", "TimeStamp"],
            false,
        )
        .await
    }

    async fn down(&self, _: &SchemaManager) -> Result<(), DbErr> {
        Err(DbErr::Migration(
            "upstream ZoneMinder schema updates are not reversible".into(),
        ))
    }
}
