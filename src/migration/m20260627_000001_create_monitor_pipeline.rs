//! Create the zm_api-owned `monitor_pipeline` table.
//!
//! One row per monitor that runs on zm-next: it stores the **processing plugin
//! graph** (the `{id,kind,cfg,children}` tree from `decode` downward) that
//! ZoneMinder's schema has no concept of. The capture node, credentials, the
//! `store` (recording) node, and zones are NOT stored here — they are re-derived
//! from `Monitors`/`Zones` and composed in at spawn, so this table is purely
//! additive and never a competing source of truth (and holds no secrets).
//!
//! `monitor_id` is the primary key (one graph per monitor) and a *logical* FK to
//! `Monitors.Id`; no hard cross-table constraint is created because zm-api does
//! not own ZoneMinder's `Monitors` table. Columns are snake_case to match the
//! hand-written entity in `src/entity/monitor_pipeline.rs`.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// The `monitor_pipeline` table create statement. Extracted so the DDL can be
/// rendered and asserted offline (the migration itself needs a live DB).
fn monitor_pipeline_table() -> TableCreateStatement {
    Table::create()
        .table(MonitorPipeline::Table)
        .if_not_exists()
        .col(
            ColumnDef::new(MonitorPipeline::MonitorId)
                .unsigned()
                .not_null()
                .primary_key(),
        )
        // The processing plugin graph as a JSON document (validated by zm-api
        // before write; deeply validated by zm-next at spawn).
        .col(ColumnDef::new(MonitorPipeline::GraphJson).text().not_null())
        // Document/schema version, so the graph shape can evolve additively.
        .col(
            ColumnDef::new(MonitorPipeline::Version)
                .unsigned()
                .not_null()
                .default(1),
        )
        .col(
            ColumnDef::new(MonitorPipeline::CreatedAt)
                .date_time()
                .not_null(),
        )
        .col(
            ColumnDef::new(MonitorPipeline::UpdatedAt)
                .date_time()
                .not_null(),
        )
        .to_owned()
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(monitor_pipeline_table()).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(MonitorPipeline::Table).to_owned())
            .await
    }
}

/// Idens spell the table/column names exactly as the entity expects them.
#[derive(DeriveIden)]
enum MonitorPipeline {
    #[sea_orm(iden = "monitor_pipeline")]
    Table,
    #[sea_orm(iden = "monitor_id")]
    MonitorId,
    #[sea_orm(iden = "graph_json")]
    GraphJson,
    #[sea_orm(iden = "version")]
    Version,
    #[sea_orm(iden = "created_at")]
    CreatedAt,
    #[sea_orm(iden = "updated_at")]
    UpdatedAt,
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
        let sql = monitor_pipeline_table()
            .to_string(MysqlQueryBuilder)
            .to_lowercase();

        assert!(sql.contains("`monitor_pipeline`"), "table name: {sql}");
        // monitor_id is the (non-auto-increment) primary key.
        assert!(
            sql.contains("`monitor_id`") && sql.contains("primary key"),
            "monitor_id pk: {sql}"
        );
        assert!(!sql.contains("auto_increment"), "no auto_increment: {sql}");
        // Graph stored as TEXT.
        assert!(
            sql.contains("`graph_json`") && sql.contains("text"),
            "graph_json text: {sql}"
        );
        // Version defaults to 1.
        assert!(
            sql.contains("`version`") && sql.contains("default 1"),
            "version default: {sql}"
        );
        // Timestamps.
        assert!(
            sql.contains("`created_at`") && sql.contains("datetime"),
            "created_at: {sql}"
        );
        assert!(sql.contains("`updated_at`"), "updated_at: {sql}");
    }
}
