//! Create the zm-api-owned `zmnext_secret` table.
//!
//! Secrets from a monitor's stored zm-next processing graph (MQTT password,
//! webhook `auth_header`, LLM `api_key`) are kept here, encrypted, instead of in
//! `monitor_pipeline.graph_json`, which holds `{"$secret": "<name>"}` references
//! in their place. See `src/service/zmnext/secrets.rs`.
//!
//! `(monitor_id, name)` is the primary key; `monitor_id` is a logical FK to
//! `Monitors.Id` with no hard constraint, like `monitor_pipeline`.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

fn zmnext_secret_table() -> TableCreateStatement {
    Table::create()
        .table(ZmnextSecret::Table)
        .if_not_exists()
        .col(
            ColumnDef::new(ZmnextSecret::MonitorId)
                .unsigned()
                .not_null(),
        )
        // Short and ASCII (node id or path plus key), so 191 fits a utf8mb4 key.
        .col(
            ColumnDef::new(ZmnextSecret::Name)
                .string_len(191)
                .not_null(),
        )
        // base64(nonce || ChaCha20-Poly1305 ciphertext).
        .col(ColumnDef::new(ZmnextSecret::Ciphertext).text().not_null())
        .col(
            ColumnDef::new(ZmnextSecret::UpdatedAt)
                .date_time()
                .not_null(),
        )
        .primary_key(
            Index::create()
                .col(ZmnextSecret::MonitorId)
                .col(ZmnextSecret::Name),
        )
        .to_owned()
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(zmnext_secret_table()).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ZmnextSecret::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ZmnextSecret {
    #[sea_orm(iden = "zmnext_secret")]
    Table,
    #[sea_orm(iden = "monitor_id")]
    MonitorId,
    #[sea_orm(iden = "name")]
    Name,
    #[sea_orm(iden = "ciphertext")]
    Ciphertext,
    #[sea_orm(iden = "updated_at")]
    UpdatedAt,
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::sea_query::{MysqlQueryBuilder, PostgresQueryBuilder};

    #[test]
    fn table_ddl_has_expected_columns() {
        for sql in [
            zmnext_secret_table().to_string(MysqlQueryBuilder),
            zmnext_secret_table().to_string(PostgresQueryBuilder),
        ] {
            let sql = sql.to_lowercase();
            assert!(sql.contains("zmnext_secret"), "{sql}");
            assert!(
                sql.contains("primary key") && sql.contains("monitor_id"),
                "{sql}"
            );
            assert!(sql.contains("ciphertext") && sql.contains("text"), "{sql}");
            assert!(sql.contains("varchar(191)"), "{sql}");
        }
    }
}
