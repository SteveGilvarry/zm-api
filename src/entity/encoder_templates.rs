//! `SeaORM` Entity for ZoneMinder 1.39's `EncoderTemplates`: named encoder
//! parameter sets monitors' encoder settings refer to. Curated by hand to
//! match `zm_create.sql.in`.

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "EncoderTemplates")]
pub struct Model {
    #[sea_orm(column_name = "Id", primary_key)]
    pub id: u32,
    #[sea_orm(column_name = "Encoder")]
    pub encoder: String,
    #[sea_orm(column_name = "Name")]
    pub name: String,
    #[sea_orm(column_name = "Description", column_type = "Text", nullable)]
    pub description: Option<String>,
    #[sea_orm(column_name = "Params", column_type = "Text")]
    pub params: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
