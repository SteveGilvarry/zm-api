//! `SeaORM` Entity for ZoneMinder 1.39's `Menu_Items`: the navbar's entries,
//! their order, labels and whether they show. Curated by hand to match
//! `zm_create.sql.in`. `IconType` is an enum column read as text.

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "Menu_Items")]
pub struct Model {
    #[sea_orm(column_name = "Id", primary_key)]
    pub id: u32,
    #[sea_orm(column_name = "MenuKey", unique)]
    pub menu_key: String,
    #[sea_orm(column_name = "Enabled")]
    pub enabled: i8,
    #[sea_orm(column_name = "Label")]
    pub label: Option<String>,
    #[sea_orm(column_name = "SortOrder")]
    pub sort_order: i16,
    #[sea_orm(column_name = "Icon")]
    pub icon: Option<String>,
    #[sea_orm(column_name = "IconType")]
    pub icon_type: String,
    #[sea_orm(column_name = "Link")]
    pub link: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
