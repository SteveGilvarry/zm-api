//! Mirrors `db/legacy/zm_update-1.39.9.sql`: the `EncoderTemplates` table
//! and its fourteen presets.

use sea_orm_migration::prelude::*;

use super::helpers::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

const PRESETS: [(&str, &str, &str, &str); 14] = [
    ("libx264", "Balanced", "1080p recording with reasonable CPU cost. Good default for most cameras.", "preset=fast\ncrf=23\ng=30\nprofile=high\npix_fmt=yuv420p"),
    ("libx264", "Archival (high quality)", "Slow encode for archival storage; substantially smaller files at higher CPU cost.", "preset=slow\ncrf=20\ng=30\nprofile=high\npix_fmt=yuv420p"),
    ("libx264", "Low CPU", "Highest encoding speed for slow CPUs; quality and file size trade off.", "preset=ultrafast\ncrf=26\ng=30\nprofile=baseline\npix_fmt=yuv420p"),
    ("libx265", "Balanced", "1080p HEVC recording with reasonable CPU cost. Significantly smaller files than x264 at similar quality.", "preset=fast\ncrf=25\ng=30\nprofile=main\npix_fmt=yuv420p"),
    ("libx265", "Archival (high quality)", "Slow HEVC encode for archival storage.", "preset=slow\ncrf=22\ng=30\nprofile=main\npix_fmt=yuv420p"),
    ("libx265", "Low CPU", "Highest HEVC encoding speed for slow CPUs.", "preset=ultrafast\ncrf=28\ng=30\nprofile=main\npix_fmt=yuv420p"),
    ("h264_nvenc", "Balanced", "1080p H.264 on NVIDIA GPU; sane vbr+cq defaults, no B-frames for low latency.", "preset=p4\nrc=vbr\ncq=23\ng=30\nbf=0\nprofile=high\npix_fmt=nv12"),
    ("h264_nvenc", "Low Power", "Faster preset for thermally-constrained NVIDIA hardware.", "preset=p1\nrc=vbr\ncq=26\ng=30\nbf=0\nprofile=high\npix_fmt=nv12"),
    ("hevc_nvenc", "Balanced", "1080p HEVC on NVIDIA GPU; sane vbr+cq defaults, no B-frames.", "preset=p4\nrc=vbr\ncq=28\ng=30\nbf=0\nprofile=main\npix_fmt=nv12"),
    ("hevc_nvenc", "Low Power", "Faster preset for thermally-constrained NVIDIA hardware.", "preset=p1\nrc=vbr\ncq=30\ng=30\nbf=0\nprofile=main\npix_fmt=nv12"),
    ("h264_vaapi", "Balanced", "1080p H.264 via VA-API (Intel/AMD/Mesa); no B-frames.", "rc_mode=CQP\nqp=24\ng=30\nbf=0\nprofile=high\npix_fmt=nv12"),
    ("h264_vaapi", "Low Power", "Lower-quality VA-API encode using the low_power codepath.", "rc_mode=CQP\nqp=27\ng=30\nbf=0\nprofile=high\npix_fmt=nv12\nlow_power=1"),
    ("hevc_vaapi", "Balanced", "1080p HEVC via VA-API; no B-frames.", "rc_mode=CQP\nqp=27\ng=30\nbf=0\nprofile=main\npix_fmt=nv12"),
    ("hevc_vaapi", "Low Power", "Lower-quality HEVC VA-API encode using the low_power codepath.", "rc_mode=CQP\nqp=30\ng=30\nbf=0\nprofile=main\npix_fmt=nv12\nlow_power=1"),
];

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        if !table_exists(m, "EncoderTemplates").await? {
            create_table(
                m,
                Table::create()
                    .table(t("EncoderTemplates"))
                    .col(autoinc_pk(m, "Id", false))
                    .col(ColumnDef::new(c("Encoder")).string_len(32).not_null())
                    .col(ColumnDef::new(c("Name")).string_len(64).not_null())
                    .col(ColumnDef::new(c("Description")).text())
                    .col(ColumnDef::new(c("Params")).text().not_null())
                    .to_owned(),
                "EncoderTemplates",
                &[
                    ("Encoder", &["Encoder"], false),
                    ("Encoder_Name", &["Encoder", "Name"], true),
                ],
            )
            .await?;
        }
        for (encoder, name, description, params) in PRESETS {
            insert_if_absent(
                m,
                Query::select()
                    .expr(Expr::value(1))
                    .from(t("EncoderTemplates"))
                    .and_where(Expr::col(c("Encoder")).eq(encoder))
                    .and_where(Expr::col(c("Name")).eq(name))
                    .to_owned(),
                Query::insert()
                    .into_table(t("EncoderTemplates"))
                    .columns([c("Encoder"), c("Name"), c("Description"), c("Params")])
                    .values_panic([
                        encoder.into(),
                        name.into(),
                        description.into(),
                        params.into(),
                    ])
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
