//! Create the zm-api-owned `media_jobs` table: the media job queue of
//! FILTERS_PLAN.md (Subsystem 3). Its first kind generates a video for an
//! event (#187). `event_id` and `monitor_id` are logical FKs with no hard
//! constraint, like `monitor_pipeline`; `monitor_id` lets a row-level ACL
//! filter jobs without a join.

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DatabaseBackend;

#[derive(DeriveMigrationName)]
pub struct Migration;

fn media_jobs_table(backend: DatabaseBackend) -> TableCreateStatement {
    // Postgres has no unsigned auto-increment.
    let mut id = ColumnDef::new(MediaJobs::Id);
    match backend {
        DatabaseBackend::MySql => id.unsigned(),
        _ => id.integer(),
    };
    Table::create()
        .table(MediaJobs::Table)
        .if_not_exists()
        .col(id.not_null().auto_increment().primary_key())
        .col(ColumnDef::new(MediaJobs::Kind).string_len(32).not_null())
        .col(ColumnDef::new(MediaJobs::EventId).big_unsigned().not_null())
        .col(ColumnDef::new(MediaJobs::MonitorId).unsigned().not_null())
        // The kind's parameters, as JSON.
        .col(ColumnDef::new(MediaJobs::Params).text().not_null())
        // queued | running | done | failed | cancelled
        .col(ColumnDef::new(MediaJobs::Status).string_len(16).not_null())
        .col(ColumnDef::new(MediaJobs::Output).string_len(255).null())
        .col(ColumnDef::new(MediaJobs::Error).text().null())
        .col(ColumnDef::new(MediaJobs::CreatedBy).unsigned().null())
        .col(ColumnDef::new(MediaJobs::CreatedAt).date_time().not_null())
        .col(ColumnDef::new(MediaJobs::StartedAt).date_time().null())
        .col(ColumnDef::new(MediaJobs::FinishedAt).date_time().null())
        .to_owned()
}

fn status_index() -> IndexCreateStatement {
    Index::create()
        .if_not_exists()
        .name("idx_media_jobs_status")
        .table(MediaJobs::Table)
        .col(MediaJobs::Status)
        .to_owned()
}

fn event_index() -> IndexCreateStatement {
    Index::create()
        .if_not_exists()
        .name("idx_media_jobs_event")
        .table(MediaJobs::Table)
        .col(MediaJobs::EventId)
        .to_owned()
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(media_jobs_table(manager.get_database_backend()))
            .await?;
        manager.create_index(status_index()).await?;
        manager.create_index(event_index()).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(MediaJobs::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum MediaJobs {
    #[sea_orm(iden = "media_jobs")]
    Table,
    #[sea_orm(iden = "id")]
    Id,
    #[sea_orm(iden = "kind")]
    Kind,
    #[sea_orm(iden = "event_id")]
    EventId,
    #[sea_orm(iden = "monitor_id")]
    MonitorId,
    #[sea_orm(iden = "params")]
    Params,
    #[sea_orm(iden = "status")]
    Status,
    #[sea_orm(iden = "output")]
    Output,
    #[sea_orm(iden = "error")]
    Error,
    #[sea_orm(iden = "created_by")]
    CreatedBy,
    #[sea_orm(iden = "created_at")]
    CreatedAt,
    #[sea_orm(iden = "started_at")]
    StartedAt,
    #[sea_orm(iden = "finished_at")]
    FinishedAt,
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::sea_query::{MysqlQueryBuilder, PostgresQueryBuilder};

    #[test]
    fn table_ddl_has_expected_columns() {
        for sql in [
            media_jobs_table(DatabaseBackend::MySql).to_string(MysqlQueryBuilder),
            media_jobs_table(DatabaseBackend::Postgres).to_string(PostgresQueryBuilder),
        ] {
            let sql = sql.to_lowercase();
            for col in [
                "media_jobs",
                "event_id",
                "monitor_id",
                "params",
                "status",
                "finished_at",
            ] {
                assert!(sql.contains(col), "{col}: {sql}");
            }
        }
    }
}
