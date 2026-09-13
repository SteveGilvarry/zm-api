//! Mirrors `db/legacy/zm_update-1.39.21.sql`: the `Events_Lock` advisory-lock
//! table (deliberately no foreign key to Events — see upstream's comment).

use sea_orm_migration::prelude::*;

use super::helpers::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        if table_exists(m, "Events_Lock").await? {
            return Ok(());
        }
        create_table(
            m,
            Table::create()
                .table(t("Events_Lock"))
                .col(
                    ColumnDef::new(c("EventId"))
                        .big_unsigned()
                        .not_null()
                        .primary_key(),
                )
                .col(ColumnDef::new(c("LockedBy")).string_len(64).not_null())
                .col(ColumnDef::new(c("LockedAt")).date_time().not_null())
                .col(ColumnDef::new(c("ExpiresAt")).date_time().not_null())
                .to_owned(),
            "Events_Lock",
            &[("Events_Lock_ExpiresAt_idx", &["ExpiresAt"], false)],
        )
        .await
    }

    async fn down(&self, _: &SchemaManager) -> Result<(), DbErr> {
        Err(DbErr::Migration(
            "upstream ZoneMinder schema updates are not reversible".into(),
        ))
    }
}
