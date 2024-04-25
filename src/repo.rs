use std::sync::Arc;

use bigchaindb::{
    connection::Connection,
    ed25519_keypair,
    transaction::{Operation, Transaction, UnspentOutput},
};

use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, DatabaseTransaction,
    DbErr, EntityTrait, FromQueryResult, IntoActiveModel, JoinType, QueryFilter, QuerySelect,
    RelationTrait, TransactionTrait, TryIntoModel,
};
use serde::{Deserialize, Serialize};
use serde_json;

use crate::entity::{prelude::*, *};

#[derive(Deserialize, Debug)]
pub struct ProvisionWallet {
    pub edge_id: i32,
    pub asset: serde_json::Value,
}

#[derive(Deserialize, Debug)]
pub struct TransferToken {
    pub edge_id: i32,
}

#[derive(FromQueryResult, Debug, Serialize)]
pub struct Wallet {
    #[serde(skip_serializing)]
    pub wallet_id: i32,

    pub public_key: String,
    pub private_key: String,
    pub token: String,
    pub volume: i32,
}

#[derive(Serialize, Debug)]
pub struct EdgeWallet {
    pub edge_id: i32,
    pub src_wallet: Wallet,
    pub dst_wallet: Wallet,
    pub token: String,
    pub nft: String,
}

pub struct Repo {
    pub db: DatabaseConnection,
    pub bigchain_url: String,
    pub init_amount: i32,
}

impl Repo {
    pub async fn provision_wallet(
        self: Arc<Self>,
        data: ProvisionWallet,
    ) -> anyhow::Result<EdgeWallet> {
        let _self = self.clone();
        _self
            .db
            .transaction::<_, (), DbErr>(|tx| {
                Box::pin(async move {
                    let src_wallet = self.create_wallet(tx).await?;
                    let dst_wallet = self.create_wallet(tx).await?;
                    let nft_wallet = self.create_wallet(tx).await?;

                    let metadata = serde_json::json!({ "co": "devr" });

                    // create FT
                    let token = self
                        .create_token(
                            &src_wallet,
                            100,
                            Some(data.asset),
                            Some(metadata.clone()),
                            tx,
                        )
                        .await
                        .map_err(|_| DbErr::Custom("create FT error".to_string()))?;

                    // create NFT
                    let nft_asset = serde_json::json!({
                        "token": "NFT",
                    });
                    let nft = self
                        .create_token(&nft_wallet, 1, Some(nft_asset), Some(metadata), tx)
                        .await
                        .map_err(|_| DbErr::Custom("create NFT error".to_string()))?;

                    // create wallet_to_token
                    let _ = self
                        .create_wallet_to_token(src_wallet.id, token.id, self.init_amount, tx)
                        .await
                        .map_err(|_| {
                            DbErr::Custom("create src wallet_to_token error".to_string())
                        })?;
                    let _ = self
                        .create_wallet_to_token(dst_wallet.id, token.id, 0, tx)
                        .await
                        .map_err(|_| {
                            DbErr::Custom("create dst wallet_to_token error".to_string())
                        })?;
                    let _ = self
                        .create_wallet_to_token(nft_wallet.id, nft.id, 1, tx)
                        .await
                        .map_err(|_| {
                            DbErr::Custom("create nft wallet_to_token error".to_string())
                        })?;

                    let _ = self
                        .create_edge_to_wallet(
                            data.edge_id,
                            src_wallet.id,
                            dst_wallet.id,
                            nft_wallet.id,
                            tx,
                        )
                        .await
                        .map_err(|_| DbErr::Custom("create edge_to_wallet error".to_string()))?;

                    Ok(())
                })
            })
            .await?;

        _self.get_edge_wallet(data.edge_id).await
    }

    pub async fn transfer_token(
        self: Arc<Self>,
        data: TransferToken,
    ) -> anyhow::Result<EdgeWallet> {
        let edge_wallet = self.get_edge_wallet(data.edge_id).await?;

        let _ = self
            .bigchain_transfer_token(
                &edge_wallet.src_wallet,
                &edge_wallet.dst_wallet,
                &edge_wallet.token,
                1,
            )
            .await?;

        let _self = self.clone();
        _self
            .db
            .transaction::<_, (), DbErr>(|tx| {
                Box::pin(async move {
                    let mut src_wallet = WalletsToTokens::find()
                        .filter(
                            wallets_to_tokens::Column::WalletId
                                .eq(edge_wallet.src_wallet.wallet_id),
                        )
                        .one(tx)
                        .await?
                        .ok_or_else(|| DbErr::Custom("find src_wallet error".to_string()))?
                        .into_active_model();

                    let src_wallet_vol = src_wallet.volume.clone().unwrap();
                    if src_wallet_vol > 0 {
                        src_wallet.volume = Set(src_wallet_vol - 1);
                    }
                    let _ = src_wallet.update(tx).await?;

                    let mut dst_wallet = WalletsToTokens::find()
                        .filter(
                            wallets_to_tokens::Column::WalletId
                                .eq(edge_wallet.dst_wallet.wallet_id),
                        )
                        .one(tx)
                        .await?
                        .ok_or_else(|| DbErr::Custom("find src_wallet error".to_string()))?
                        .into_active_model();

                    let dst_wallet_vol = dst_wallet.volume.clone().unwrap();
                    dst_wallet.volume = Set(dst_wallet_vol + 1);
                    let _ = dst_wallet.update(tx).await?;

                    Ok(())
                })
            })
            .await?;

        _self.get_edge_wallet(data.edge_id).await
    }

    async fn bigchain_transfer_token(
        &self,
        sender: &Wallet,
        receiver: &Wallet,
        token: &str,
        transfer_amount: i32,
    ) -> anyhow::Result<()> {
        let mut conn = Connection::new(vec![&self.bigchain_url]);

        let list_outputs = conn.list_outputs(&sender.public_key, Some(false)).await?;

        // find unspent_output of sender's pubkey and token
        let mut unspent_outputs = Vec::new();
        for output in list_outputs.iter() {
            let tx = conn.get_transaction(&output.transaction_id).await?;
            unspent_outputs.push(UnspentOutput {
                tx,
                output_index: output.output_index,
            })
        }
        let unspent_output = unspent_outputs.iter().find(|e| match &e.tx.operation {
            Some(Operation::CREATE) => {
                if let Some(id) = &e.tx.id {
                    return id == token;
                }
                false
            }
            Some(Operation::TRANSFER) => {
                if let Some(asset) = &e.tx.asset {
                    if let Some(id) = asset.get_link_id() {
                        return id == token;
                    }
                }
                false
            }
            None => false,
        });
        let unspent_output = unspent_output.unwrap();

        let total_amount = unspent_output.tx.outputs[unspent_output.output_index]
            .amount
            .parse::<i32>()?;

        // create transaction output
        let mut outputs = Vec::new();
        for (amount, pubkey) in [
            (total_amount - transfer_amount, &sender.public_key),
            (transfer_amount, &receiver.public_key),
        ] {
            if amount > 0 {
                outputs.push(Transaction::make_output(
                    Transaction::make_ed25519_condition(pubkey, true).unwrap(),
                    amount.to_string(),
                ));
            }
        }

        // make transfer transaction
        let transfer_tx = Transaction::make_transfer_transaction(
            vec![unspent_output.clone()],
            outputs,
            Some(serde_json::json!({
                "transfer_to": &receiver.public_key,
                "transfer_amount": transfer_amount,
            })),
        );

        // signed tranasction with sender's private_key
        let signed_tx = Transaction::sign_transaction(&transfer_tx, vec![&sender.private_key]);

        // commit tranasction to BigchainDB
        let _tx = conn.post_transaction_commit(signed_tx).await?;

        Ok(())
    }

    async fn get_wallets_to_tokens(&self, wallet_id: i32) -> anyhow::Result<Wallet> {
        let wallet = WalletsToTokens::find()
            .column_as(wallets::Column::Id, "wallet_id")
            .column_as(tokens::Column::Token, "token")
            .column_as(wallets::Column::PublicKey, "public_key")
            .column_as(wallets::Column::PrivateKey, "private_key")
            .filter(wallets_to_tokens::Column::WalletId.eq(wallet_id))
            .join(
                JoinType::InnerJoin,
                wallets_to_tokens::Relation::Tokens.def(),
            )
            .join(
                JoinType::InnerJoin,
                wallets_to_tokens::Relation::Wallets.def(),
            )
            .into_model::<Wallet>()
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("dst_wallet_id not found"))?;
        Ok(wallet)
    }

    pub async fn get_edge_wallet(&self, edge_id: i32) -> anyhow::Result<EdgeWallet> {
        let edge_to_wallet = self.get_edges_to_wallets(edge_id).await?;

        let src_wallet = self
            .get_wallets_to_tokens(edge_to_wallet.src_wallet_id)
            .await?;
        let dst_wallet = self
            .get_wallets_to_tokens(edge_to_wallet.dst_wallet_id)
            .await?;
        let nft_wallet = self
            .get_wallets_to_tokens(edge_to_wallet.nft_wallet_id)
            .await?;

        let token = src_wallet.token.clone();
        let nft = nft_wallet.token.clone();
        let resp = EdgeWallet {
            edge_id,
            src_wallet,
            dst_wallet,
            token,
            nft,
        };

        Ok(resp)
    }

    async fn get_edges_to_wallets(&self, edge_id: i32) -> anyhow::Result<edges_to_wallets::Model> {
        let record = EdgesToWallets::find()
            .filter(edges_to_wallets::Column::EdgeId.eq(edge_id))
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("edge_id not found"))?;
        Ok(record)
    }

    // async fn get_wallet_by_id(&self, wallet_id: i32) -> anyhow::Result<wallets::Model> {
    //     let record = Wallets::find_by_id(wallet_id)
    //         .one(&self.db)
    //         .await?
    //         .ok_or_else(|| anyhow::anyhow!("wallet_id not found"))?;
    //     Ok(record)
    // }

    async fn create_wallet(&self, tx: &DatabaseTransaction) -> Result<wallets::Model, DbErr> {
        let keypair = ed25519_keypair();

        let wallet = wallets::ActiveModel {
            public_key: Set(keypair.pk),
            private_key: Set(keypair.sk),
            ..Default::default()
        }
        .save(tx)
        .await?
        .try_into_model()?;

        Ok(wallet)
    }

    async fn create_token(
        &self,
        signer: &wallets::Model,
        init_amount: i32,
        asset: Option<serde_json::Value>,
        metadata: Option<serde_json::Value>,
        db_tx: &DatabaseTransaction,
    ) -> Result<tokens::Model, DbErr> {
        let condition = Transaction::make_ed25519_condition(&signer.public_key, true).unwrap();
        let output = Transaction::make_output(condition, init_amount.to_string());
        let tx = Transaction::make_create_transaction(
            asset,
            metadata,
            vec![output],
            vec![signer.private_key.to_string()],
        );
        let signed_tx = Transaction::sign_transaction(&tx, vec![&signer.private_key]);

        let mut conn = Connection::new(vec![&self.bigchain_url]);
        let bigchain_tx = conn
            .post_transaction_commit(signed_tx)
            .await
            .map_err(|_| DbErr::Custom("BigchainDB post transaction error".to_string()))?;

        let token = tokens::ActiveModel {
            token: Set(bigchain_tx.id.clone().unwrap()),
            ..Default::default()
        }
        .save(db_tx)
        .await?
        .try_into_model()?;

        Ok(token)
    }

    async fn create_wallet_to_token(
        &self,
        wallet_id: i32,
        token_id: i32,
        amount: i32,
        tx: &DatabaseTransaction,
    ) -> Result<wallets_to_tokens::Model, DbErr> {
        let model = wallets_to_tokens::ActiveModel {
            token_id: Set(token_id),
            wallet_id: Set(wallet_id),
            volume: Set(amount),
        };
        let record = model.insert(tx).await?;
        Ok(record)
    }

    async fn create_edge_to_wallet(
        &self,
        edge_id: i32,
        src_wallet_id: i32,
        dst_wallet_id: i32,
        nft_wallet_id: i32,
        tx: &DatabaseTransaction,
    ) -> Result<edges_to_wallets::Model, DbErr> {
        let model = edges_to_wallets::ActiveModel {
            edge_id: Set(edge_id),
            src_wallet_id: Set(src_wallet_id),
            dst_wallet_id: Set(dst_wallet_id),
            nft_wallet_id: Set(nft_wallet_id),
            ..Default::default()
        }
        .save(tx)
        .await?
        .try_into_model()?;
        Ok(model)
    }
}
