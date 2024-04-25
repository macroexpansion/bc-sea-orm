use bc_orm::{
    connect,
    entity::{prelude::*, *},
};
use sea_orm::{ActiveModelTrait, DatabaseConnection, DbErr, EntityTrait, InsertResult, Set};

const DATABASE_URL: &str = "postgres://username:password123@localhost:5432/bc";

#[tokio::main]
async fn main() {
    let db = connect(DATABASE_URL).await.unwrap();

    // let token = tokens::ActiveModel {
    //     token: Set("token".to_owned()),
    //     ..Default::default()
    // };

    // insert_1(token.clone(), &db).await.unwrap();
    // insert_2(token, &db).await.unwrap();
}

// /// Insert and return inserted record
// async fn insert_1(token: tokens::ActiveModel, db: &DatabaseConnection) -> Result<(), DbErr> {
//     let model: tokens::Model = token.insert(db).await?;
//     println!("{model:?}");
//
//     Ok(())
// }
//
// /// Insert and return number of affected records
// async fn insert_2(token: tokens::ActiveModel, db: &DatabaseConnection) -> Result<(), DbErr> {
//     let res: InsertResult<_> = Tokens::insert(token).exec(db).await?;
//     println!("{res:?}");
//
//     Ok(())
// }
