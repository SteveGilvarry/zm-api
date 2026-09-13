//! Mirrors `db/legacy/zm_update-1.39.27.sql`: one composite `Frames` index
//! replaces three single-column ones.

use sea_orm_migration::prelude::*;

use super::helpers::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        add_index_if_missing(
            m,
            "Frames",
            "EventId_FrameId_idx",
            &["EventId", "FrameId"],
            false,
        )
        .await?;
        for old in ["EventId_idx", "Type", "TimeStamp"] {
            drop_index_if_present(m, "Frames", old).await?;
        }
        Ok(())
    }

    async fn down(&self, _: &SchemaManager) -> Result<(), DbErr> {
        Err(DbErr::Migration(
            "upstream ZoneMinder schema updates are not reversible".into(),
        ))
    }
}
