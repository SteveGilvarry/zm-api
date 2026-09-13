//! Mirrors `db/legacy/zm_update-1.39.8.sql`: `Reports.CreatedBy`.

use sea_orm_migration::prelude::*;

use super::helpers::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        add_column_if_missing(
            m,
            "Reports",
            "CreatedBy",
            ColumnDef::new(c("CreatedBy")).unsigned().to_owned(),
        )
        .await
    }

    async fn down(&self, _: &SchemaManager) -> Result<(), DbErr> {
        Err(DbErr::Migration(
            "upstream ZoneMinder schema updates are not reversible".into(),
        ))
    }
}
