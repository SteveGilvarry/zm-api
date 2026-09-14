//! zm-api-owned `zmnext_secret` table: encrypted secrets taken out of stored
//! zm-next processing graphs. Hand-written, not generated from ZoneMinder's
//! schema (see `src/migration/m20260914_000001_create_zmnext_secret.rs`).

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "zmnext_secret")]
pub struct Model {
    /// Logical FK to `Monitors.Id`.
    #[sea_orm(primary_key, auto_increment = false)]
    pub monitor_id: u32,
    /// The name a `{"$secret": name}` reference in the graph uses.
    #[sea_orm(primary_key, auto_increment = false)]
    pub name: String,
    /// base64(nonce || ChaCha20-Poly1305 ciphertext); never returned by the API.
    #[sea_orm(column_type = "Text")]
    pub ciphertext: String,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
