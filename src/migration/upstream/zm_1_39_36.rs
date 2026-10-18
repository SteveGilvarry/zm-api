//! Mirrors `db/legacy/zm_update-1.39.36.sql`: `(EventId, FrameId)` becomes
//! the primary key of `Frames` and the surrogate `Id` is dropped.
//!
//! `AI_Detections.FrameId` stops being a foreign key to `Frames.Id` and
//! becomes the per-event frame number, converted from the old Id first; its
//! two single-column indexes merge into `(EventId, FrameId)`.
//! `Frames.TimeStamp` loses `ON UPDATE CURRENT_TIMESTAMP` (MySQL only; the
//! Postgres baseline never had it).
//!
//! Duplicate `(EventId, FrameId)` rows are removed before the new key is
//! added, keeping the earliest. Upstream batches that delete a hundred events
//! at a time to keep row locks bounded on huge tables; existing MySQL
//! installs reach this version through the raw chain, which runs upstream's
//! procedure. The databases this migration runs on are fresh or Postgres, so
//! one statement is enough.

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DatabaseBackend;

use super::helpers::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let mysql = backend(m) == DatabaseBackend::MySql;
        let has_id = column_exists(m, "Frames", "Id").await?;

        if table_exists(m, "AI_Detections").await? {
            drop_foreign_keys_to(m, "AI_Detections", "Frames").await?;
            if has_id {
                run_raw(
                    m,
                    if mysql {
                        "UPDATE `AI_Detections` D JOIN `Frames` F ON F.`Id` = D.`FrameId` \
                         SET D.`FrameId` = F.`FrameId`"
                    } else {
                        "UPDATE \"AI_Detections\" d SET \"FrameId\" = f.\"FrameId\" \
                         FROM \"Frames\" f WHERE f.\"Id\" = d.\"FrameId\""
                    },
                )
                .await?;
            }
            if !index_exists(m, "AI_Detections", "AI_Detections_EventId_FrameId_idx").await? {
                modify_column(
                    m,
                    "AI_Detections",
                    ColumnDef::new(c("FrameId")).unsigned().to_owned(),
                )
                .await?;
                add_index_if_missing(
                    m,
                    "AI_Detections",
                    "AI_Detections_EventId_FrameId_idx",
                    &["EventId", "FrameId"],
                    false,
                )
                .await?;
            }
            drop_index_if_present(m, "AI_Detections", "AI_Detections_FrameId_idx").await?;
            drop_index_if_present(m, "AI_Detections", "AI_Detections_EventId_idx").await?;
        }

        if !has_id {
            return Ok(());
        }

        run_raw(
            m,
            if mysql {
                "DELETE F1 FROM `Frames` F1 JOIN `Frames` F2 \
                 ON F1.`EventId` = F2.`EventId` AND F1.`FrameId` = F2.`FrameId` \
                 AND F1.`Id` > F2.`Id`"
            } else {
                "DELETE FROM \"Frames\" f1 USING \"Frames\" f2 \
                 WHERE f1.\"EventId\" = f2.\"EventId\" AND f1.\"FrameId\" = f2.\"FrameId\" \
                 AND f1.\"Id\" > f2.\"Id\""
            },
        )
        .await?;

        if mysql {
            // One statement, as upstream does: an AUTO_INCREMENT column must
            // stay a key, so the old key and Id go together.
            let mut sql = String::from(
                "ALTER TABLE `Frames` DROP PRIMARY KEY, DROP COLUMN `Id`, \
                 ADD PRIMARY KEY (`EventId`, `FrameId`)",
            );
            if index_exists(m, "Frames", "EventId_FrameId_idx").await? {
                sql.push_str(", DROP INDEX `EventId_FrameId_idx`");
            }
            sql.push_str(", MODIFY `TimeStamp` timestamp NOT NULL default CURRENT_TIMESTAMP");
            run_raw(m, &sql).await
        } else {
            // Dropping Id drops its primary-key constraint with it.
            drop_column_if_present(m, "Frames", "Id").await?;
            run_raw(
                m,
                "ALTER TABLE \"Frames\" ADD PRIMARY KEY (\"EventId\", \"FrameId\")",
            )
            .await?;
            drop_index_if_present(m, "Frames", "EventId_FrameId_idx").await
        }
    }

    async fn down(&self, _: &SchemaManager) -> Result<(), DbErr> {
        Err(DbErr::Migration(
            "upstream ZoneMinder schema updates are not reversible".into(),
        ))
    }
}
