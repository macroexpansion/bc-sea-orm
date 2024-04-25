use sea_orm_migration::prelude::*;

use super::m20240318_000002_create_tokens::Tokens;
use super::m20240318_000003_create_wallets::Wallets;

#[derive(Iden)]
pub enum WalletsToTokens {
    Table,
    WalletId,
    TokenId,
    Volume,
}
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20240318_000004_create_wallets_to_tokens.rs"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(WalletsToTokens::Table)
                    .col(
                        ColumnDef::new(WalletsToTokens::WalletId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(WalletsToTokens::TokenId)
                            .integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(WalletsToTokens::Volume).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(WalletsToTokens::Table, WalletsToTokens::WalletId)
                            .to(Wallets::Table, Wallets::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(WalletsToTokens::Table, WalletsToTokens::TokenId)
                            .to(Tokens::Table, Tokens::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .primary_key(
                        Index::create()
                            .col(WalletsToTokens::WalletId)
                            .col(WalletsToTokens::TokenId),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(WalletsToTokens::Table).to_owned())
            .await
    }
}
