//! Mirrors `db/legacy/zm_update-1.39.29.sql`: audio capabilities on
//! `Controls` and the ONVIF IP Speaker control.

use sea_orm_migration::prelude::*;

use super::controls::add_control;
use super::helpers::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        add_column_if_missing(
            m,
            "Controls",
            "CanAudioPlay",
            ColumnDef::new(c("CanAudioPlay"))
                .tiny_unsigned()
                .not_null()
                .default("0")
                .to_owned(),
        )
        .await?;
        add_column_if_missing(
            m,
            "Controls",
            "MinAudioFile",
            ColumnDef::new(c("MinAudioFile")).unsigned().to_owned(),
        )
        .await?;
        add_column_if_missing(
            m,
            "Controls",
            "MaxAudioFile",
            ColumnDef::new(c("MaxAudioFile")).unsigned().to_owned(),
        )
        .await?;
        add_column_if_missing(
            m,
            "Controls",
            "CanAudioVolume",
            ColumnDef::new(c("CanAudioVolume"))
                .tiny_unsigned()
                .not_null()
                .default("0")
                .to_owned(),
        )
        .await?;
        add_control(
            m,
            "ONVIF IP Speaker",
            "Ffmpeg",
            "IPSpeaker",
            &[
                ("CanAudioPlay", 1),
                ("MinAudioFile", 10),
                ("MaxAudioFile", 14),
                ("CanAudioVolume", 1),
            ],
        )
        .await?;
        Ok(())
    }

    async fn down(&self, _: &SchemaManager) -> Result<(), DbErr> {
        Err(DbErr::Migration(
            "upstream ZoneMinder schema updates are not reversible".into(),
        ))
    }
}
