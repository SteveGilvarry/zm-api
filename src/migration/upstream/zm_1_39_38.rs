//! Mirrors `db/legacy/zm_update-1.39.38.sql`: the NVSocket and Libvlc
//! capture methods are removed. Libvlc rows become Ffmpeg; NVSocket monitors
//! become Ffmpeg with capturing off, since there is no url to convert. Then
//! both values leave the `Type` enum of Monitors, MonitorPresets and
//! Controls.

use sea_orm_migration::prelude::*;

use super::helpers::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

const TYPES: [&str; 7] = [
    "Local", "Remote", "File", "Ffmpeg", "cURL", "WebSite", "VNC",
];

const TABLES: [(&str, &str); 3] = [
    ("Monitors", "monitors_type"),
    ("MonitorPresets", "monitor_presets_type"),
    ("Controls", "controls_type"),
];

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        for (table, type_name) in TABLES {
            if !column_exists(m, table, "Type").await? {
                continue;
            }
            for gone in ["Libvlc", "NVSocket"] {
                let mut update = Query::update();
                update
                    .table(t(table))
                    .value(c("Type"), enum_val(type_name, "Ffmpeg"))
                    .and_where(Expr::col(c("Type")).eq(enum_val(type_name, gone)));
                if table == "Monitors" && gone == "NVSocket" {
                    update.value(c("Capturing"), enum_val("monitors_capturing", "None"));
                }
                exec_stmt(m, &update).await?;
            }
        }
        for (table, type_name) in TABLES {
            if !column_exists(m, table, "Type").await? {
                continue;
            }
            narrow_enum(
                m,
                table,
                "Type",
                type_name,
                &TYPES,
                "Local",
                enum_col("Type", type_name, &TYPES)
                    .not_null()
                    .default("Local")
                    .to_owned(),
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
