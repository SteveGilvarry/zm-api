//! Mirrors `db/legacy/zm_update-1.39.12.sql`: Dahua/Amcrest RPC controls
//! and two Amcrest models.

use sea_orm_migration::prelude::*;

use super::controls::add_control;
use super::helpers::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let caps = [
            "CanReset",
            "CanReboot",
            "CanZoom",
            "CanZoomCon",
            "HasPresets",
            "NumPresets",
            "HasHomePreset",
            "CanSetPresets",
            "CanMove",
            "CanMoveDiag",
            "CanMoveCon",
            "CanPan",
            "CanTilt",
        ];
        let rows: [(&str, [i32; 13]); 3] = [
            (
                "Dahua/Amcrest RPC",
                [1, 1, 1, 1, 1, 25, 1, 1, 1, 1, 1, 1, 1],
            ),
            (
                "Amcrest ASH21-B RPC",
                [1, 1, 0, 0, 0, 0, 0, 0, 1, 0, 1, 1, 1],
            ),
            ("Amcrest ADC2W RPC", [1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        ];
        for (name, vals) in rows {
            let cols: Vec<(&str, i32)> = caps.iter().copied().zip(vals).collect();
            add_control(m, name, "Ffmpeg", "Dahua_RPC", &cols).await?;
        }
        for model in ["ASH21-B", "ADC2W"] {
            insert_if_absent(
                m,
                probe_eq("Models", "Name", model),
                Query::insert()
                    .into_table(t("Models"))
                    .columns([c("Name"), c("ManufacturerId")])
                    .values_panic([model.into(), 2.into()])
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
