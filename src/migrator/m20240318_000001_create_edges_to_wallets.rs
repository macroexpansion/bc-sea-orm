use sea_orm_migration::prelude::*;

use super::m20240318_000003_create_wallets::Wallets;

#[derive(Iden)]
pub enum EdgesToWallets {
    Table,
    Id,
    EdgeId,
    SrcWalletId,
    DstWalletId,
    NftWalletId,
}

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20240318_000001_create_edges_to_wallets.rs"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .if_not_exists()
                    .table(EdgesToWallets::Table)
                    .col(
                        ColumnDef::new(EdgesToWallets::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(EdgesToWallets::EdgeId).integer().not_null())
                    .col(
                        ColumnDef::new(EdgesToWallets::SrcWalletId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(EdgesToWallets::DstWalletId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(EdgesToWallets::NftWalletId)
                            .integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(EdgesToWallets::Table, EdgesToWallets::SrcWalletId)
                            .to(Wallets::Table, Wallets::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(EdgesToWallets::Table, EdgesToWallets::DstWalletId)
                            .to(Wallets::Table, Wallets::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(EdgesToWallets::Table, EdgesToWallets::NftWalletId)
                            .to(Wallets::Table, Wallets::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(EdgesToWallets::Table).to_owned())
            .await
    }
}
