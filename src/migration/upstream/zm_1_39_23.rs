//! Mirrors `db/legacy/zm_update-1.39.23.sql`: `Coords` becomes TEXT on
//! `Zones`, and on `Maps` where that table still exists.

use sea_orm_migration::prelude::*;

use super::helpers::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        modify_column(
            m,
            "Zones",
            ColumnDef::new(c("Coords")).text().not_null().to_owned(),
        )
        .await?;
        if table_exists(m, "Maps").await? {
            modify_column(
                m,
                "Maps",
                ColumnDef::new(c("Coords")).text().not_null().to_owned(),
            )
            .await?;
        }
        Ok(())
    }

    async fn down(&self, _: &SchemaManager) -> Result<(), DbErr> {
        Err(DbErr::Migration(
            "upstream ZoneMinder schema updates are not reversible".into(),
        ))
    }
}
