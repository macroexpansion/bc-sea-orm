use sea_orm_migration::prelude::*;

#[derive(Iden)]
pub enum Tokens {
    Table,
    Id,
    Token,
}

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20240318_000002_create_tokens.rs"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .if_not_exists()
                    .table(Tokens::Table)
                    .col(
                        ColumnDef::new(Tokens::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Tokens::Token).text().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Tokens::Table).to_owned())
            .await
    }
}
