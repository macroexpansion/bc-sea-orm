use sea_orm_migration::prelude::*;

#[derive(Iden)]
pub enum Wallets {
    Table,
    Id,
    PublicKey,
    PrivateKey,
}

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20240318_000003_create_wallets.rs"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .if_not_exists()
                    .table(Wallets::Table)
                    .col(
                        ColumnDef::new(Wallets::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Wallets::PublicKey).text().not_null())
                    .col(ColumnDef::new(Wallets::PrivateKey).text().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Wallets::Table).to_owned())
            .await
    }
}
