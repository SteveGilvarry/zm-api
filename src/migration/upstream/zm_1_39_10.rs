//! Mirrors `db/legacy/zm_update-1.39.10.sql`: `DefaultScale` widens to a
//! string (`fit_to` → `fit_to_width`), `Logs` gets its composite index,
//! `ZonePresets` thresholds are re-seeded as percentages, `Sessions.access`
//! is indexed.

use sea_orm_migration::prelude::*;

use super::helpers::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        for table in ["Monitors", "MonitorPresets"] {
            modify_column(
                m,
                table,
                ColumnDef::new(c("DefaultScale"))
                    .string_len(16)
                    .not_null()
                    .default("0")
                    .to_owned(),
            )
            .await?;
            exec_stmt(
                m,
                Query::update()
                    .table(t(table))
                    .value(c("DefaultScale"), "fit_to_width")
                    .and_where(Expr::col(c("DefaultScale")).eq("fit_to")),
            )
            .await?;
        }

        add_index_if_missing(
            m,
            "Logs",
            "Logs_Component_Level_TimeKey_Id_idx",
            &["Component", "Level", "TimeKey", "Id"],
            false,
        )
        .await?;
        drop_index_if_present(m, "Logs", "Logs_Component_idx").await?;

        // (Id, MinAlarm, MaxAlarm, MinFilter, MaxFilter, MinBlob)
        let presets: [(i32, &[(&str, f64)]); 7] = [
            (
                1,
                &[
                    ("MinAlarmPixels", 0.5),
                    ("MaxAlarmPixels", 75.0),
                    ("MinFilterPixels", 0.35),
                    ("MaxFilterPixels", 75.0),
                    ("MinBlobPixels", 0.3),
                ],
            ),
            (2, &[("MinAlarmPixels", 3.0)]),
            (3, &[("MinAlarmPixels", 0.5)]),
            (4, &[("MinAlarmPixels", 0.1)]),
            (
                5,
                &[
                    ("MinAlarmPixels", 5.0),
                    ("MinFilterPixels", 3.5),
                    ("MinBlobPixels", 3.0),
                ],
            ),
            (
                6,
                &[
                    ("MinAlarmPixels", 1.0),
                    ("MinFilterPixels", 0.7),
                    ("MinBlobPixels", 0.6),
                ],
            ),
            (
                7,
                &[
                    ("MinAlarmPixels", 0.2),
                    ("MinFilterPixels", 0.14),
                    ("MinBlobPixels", 0.12),
                ],
            ),
        ];
        for (id, values) in presets {
            let mut upd = Query::update();
            upd.table(t("ZonePresets"))
                .and_where(Expr::col(c("Id")).eq(id));
            for (col, v) in values.iter() {
                upd.value(c(col), *v);
            }
            exec_stmt(m, &upd).await?;
        }

        add_index_if_missing(m, "Sessions", "Sessions_access_idx", &["access"], false).await
    }

    async fn down(&self, _: &SchemaManager) -> Result<(), DbErr> {
        Err(DbErr::Migration(
            "upstream ZoneMinder schema updates are not reversible".into(),
        ))
    }
}
