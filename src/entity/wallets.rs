//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.15

use sea_orm::entity::prelude::*;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize)]
#[sea_orm(table_name = "wallets")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(column_type = "Text")]
    pub public_key: String,
    #[sea_orm(column_type = "Text")]
    pub private_key: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::wallets_to_tokens::Entity")]
    WalletsToTokens,
}

impl Related<super::wallets_to_tokens::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::WalletsToTokens.def()
    }
}

impl Related<super::tokens::Entity> for Entity {
    fn to() -> RelationDef {
        super::wallets_to_tokens::Relation::Tokens.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::wallets_to_tokens::Relation::Wallets.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
