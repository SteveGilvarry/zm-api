//! Mirrors `db/legacy/zm_update-1.39.19.sql`: `User_Preferences.Name` becomes
//! NOT NULL and `(UserId, Name)` unique, after deduplicating (the newest row
//! per pair is kept) and dropping the old single-column index.

use sea_orm_migration::prelude::*;

use super::helpers::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        exec_stmt(
            m,
            Query::delete()
                .from_table(t("User_Preferences"))
                .and_where(Expr::col(c("Name")).is_null()),
        )
        .await?;
        modify_column(
            m,
            "User_Preferences",
            ColumnDef::new(c("Name"))
                .string_len(64)
                .not_null()
                .to_owned(),
        )
        .await?;

        // Keep MAX(Id) per (UserId, Name). The derived table is what lets
        // MySQL delete from the table it is also selecting from.
        let keep = Query::select()
            .expr_as(Expr::col(c("Id")).max(), Alias::new("KeepId"))
            .from(t("User_Preferences"))
            .group_by_columns([c("UserId"), c("Name")])
            .to_owned();
        let keep_ids = Query::select()
            .column(c("KeepId"))
            .from_subquery(keep, Alias::new("k"))
            .to_owned();
        exec_stmt(
            m,
            Query::delete()
                .from_table(t("User_Preferences"))
                .and_where(Expr::col(c("Id")).not_in_subquery(keep_ids)),
        )
        .await?;

        add_index_if_missing(
            m,
            "User_Preferences",
            "User_Preferences_UserId_Name_idx",
            &["UserId", "Name"],
            true,
        )
        .await?;
        drop_index_if_present(m, "User_Preferences", "User_Preferences_UserID_idx").await
    }

    async fn down(&self, _: &SchemaManager) -> Result<(), DbErr> {
        Err(DbErr::Migration(
            "upstream ZoneMinder schema updates are not reversible".into(),
        ))
    }
}
