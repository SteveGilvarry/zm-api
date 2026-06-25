//! Create the zm_api-owned `event_synopsis` table.
//!
//! One row per `review_assets` (0x0306) event: it stores the manifest JSON, the
//! resolved asset directory, and the render lifecycle (`status`/`rendered_path`/
//! `expires_at`). `event_id` is a *logical* FK to `Events.Id` (nullable until the
//! recording handshake reconciles it); no hard cross-table constraint is created
//! because zm-api does not own ZoneMinder's `Events` table.
//!
//! Columns are snake_case to match the hand-written entity in
//! `src/entity/event_synopsis.rs`. `status` is a short portable string rather
//! than a native `ENUM`, so the same migration runs on MySQL and Postgres.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// The `event_synopsis` table create statement. Extracted so the DDL can be
/// rendered and asserted offline (the migration itself needs a live DB).
fn event_synopsis_table() -> TableCreateStatement {
    Table::create()
        .table(EventSynopsis::Table)
        .if_not_exists()
        .col(
            ColumnDef::new(EventSynopsis::Id)
                .big_unsigned()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(ColumnDef::new(EventSynopsis::EventId).big_unsigned().null())
        .col(
            ColumnDef::new(EventSynopsis::MonitorId)
                .unsigned()
                .not_null(),
        )
        .col(
            ColumnDef::new(EventSynopsis::ClipToken)
                .string_len(128)
                .not_null(),
        )
        .col(
            ColumnDef::new(EventSynopsis::ManifestJson)
                .text()
                .not_null(),
        )
        .col(ColumnDef::new(EventSynopsis::AssetDir).text().not_null())
        .col(
            ColumnDef::new(EventSynopsis::Status)
                .string_len(16)
                .not_null()
                .default("pending"),
        )
        .col(ColumnDef::new(EventSynopsis::RenderedPath).text().null())
        .col(
            ColumnDef::new(EventSynopsis::TubeCount)
                .unsigned()
                .not_null()
                .default(0),
        )
        .col(
            ColumnDef::new(EventSynopsis::SourceW)
                .unsigned()
                .not_null()
                .default(0),
        )
        .col(
            ColumnDef::new(EventSynopsis::SourceH)
                .unsigned()
                .not_null()
                .default(0),
        )
        .col(
            ColumnDef::new(EventSynopsis::CreatedAt)
                .date_time()
                .not_null(),
        )
        .col(ColumnDef::new(EventSynopsis::ExpiresAt).date_time().null())
        .to_owned()
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(event_synopsis_table()).await?;

        // One synopsis row per recording segment; `clip_token` is the stable key
        // even before `event_id` is reconciled.
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("uniq_event_synopsis_monitor_clip")
                    .table(EventSynopsis::Table)
                    .col(EventSynopsis::MonitorId)
                    .col(EventSynopsis::ClipToken)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_event_synopsis_event")
                    .table(EventSynopsis::Table)
                    .col(EventSynopsis::EventId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(EventSynopsis::Table).to_owned())
            .await
    }
}

/// Idens spell the table/column names exactly as the entity expects them.
#[derive(DeriveIden)]
enum EventSynopsis {
    #[sea_orm(iden = "event_synopsis")]
    Table,
    #[sea_orm(iden = "id")]
    Id,
    #[sea_orm(iden = "event_id")]
    EventId,
    #[sea_orm(iden = "monitor_id")]
    MonitorId,
    #[sea_orm(iden = "clip_token")]
    ClipToken,
    #[sea_orm(iden = "manifest_json")]
    ManifestJson,
    #[sea_orm(iden = "asset_dir")]
    AssetDir,
    #[sea_orm(iden = "status")]
    Status,
    #[sea_orm(iden = "rendered_path")]
    RenderedPath,
    #[sea_orm(iden = "tube_count")]
    TubeCount,
    #[sea_orm(iden = "source_w")]
    SourceW,
    #[sea_orm(iden = "source_h")]
    SourceH,
    #[sea_orm(iden = "created_at")]
    CreatedAt,
    #[sea_orm(iden = "expires_at")]
    ExpiresAt,
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::sea_query::MysqlQueryBuilder;

    /// Render the create-table DDL offline and assert its shape, so a mistake in
    /// the column types/defaults is caught in the normal `cargo test` pass — the
    /// migration itself only runs against a live MySQL in the DB-gated suite.
    #[test]
    fn table_ddl_has_expected_columns() {
        let sql = event_synopsis_table()
            .to_string(MysqlQueryBuilder)
            .to_lowercase();

        assert!(sql.contains("`event_synopsis`"), "table name: {sql}");
        // Auto-increment bigint primary key.
        assert!(sql.contains("`id`") && sql.contains("bigint"), "id: {sql}");
        assert!(sql.contains("auto_increment"), "auto_increment: {sql}");
        // Status is a short varchar defaulting to 'pending' (no native ENUM).
        assert!(
            sql.contains("`status`") && sql.contains("varchar(16)"),
            "status varchar: {sql}"
        );
        assert!(sql.contains("default 'pending'"), "status default: {sql}");
        // Text + datetime columns.
        assert!(
            sql.contains("`manifest_json`") && sql.contains("text"),
            "manifest text: {sql}"
        );
        assert!(
            sql.contains("`clip_token`") && sql.contains("varchar(128)"),
            "clip varchar: {sql}"
        );
        assert!(
            sql.contains("`created_at`") && sql.contains("datetime"),
            "created_at: {sql}"
        );
        // Nullable event_id / expires_at; non-null monitor_id.
        assert!(sql.contains("`event_id`"), "event_id: {sql}");
        assert!(sql.contains("`expires_at`"), "expires_at: {sql}");
    }
}
