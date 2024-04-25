pub mod entity;
pub mod migrator;
pub mod repo;

pub use sea_orm::*;

// use sea_orm::{Database, DatabaseConnection, DbErr};
use sea_orm_migration::MigratorTrait;

use crate::migrator::Migrator;

pub async fn connect(url: &str) -> Result<DatabaseConnection, DbErr> {
    let db = Database::connect(url).await?;

    Migrator::up(&db, None).await?;

    // let schema_manager = SchemaManager::new(&db);
    // assert!(schema_manager.has_table("edges_to_wallets").await?);
    // assert!(schema_manager.has_table("tokens").await?);
    // assert!(schema_manager.has_table("wallets").await?);
    // assert!(schema_manager.has_table("wallets_to_tokens").await?);

    Ok(db)
}
