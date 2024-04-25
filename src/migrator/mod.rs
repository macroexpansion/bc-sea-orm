mod m20240318_000001_create_edges_to_wallets;
mod m20240318_000002_create_tokens;
mod m20240318_000003_create_wallets;
mod m20240318_000004_create_wallets_to_tokens;

use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240318_000002_create_tokens::Migration),
            Box::new(m20240318_000003_create_wallets::Migration),
            Box::new(m20240318_000001_create_edges_to_wallets::Migration),
            Box::new(m20240318_000004_create_wallets_to_tokens::Migration),
        ]
    }
}
