//! Module for Ethereum transaction-related functionality.
//! Provides helpers for constructing, signing, and sending Ethereum transactions.

use alloy::{
    network::TransactionBuilder,
    primitives::{Address, TxKind, address},
    providers::Provider,
    rpc::types::{TransactionReceipt as RawTransactionReceipt, TransactionRequest},
};
use displaydoc::Display;
use thiserror::Error;

use crate::{
    Client,
    tx::{
        Transaction,
        ops::{create::Create, delete::Delete, extend::Extend, update::Update},
        receipt::{
            TransactionReceipt, create::CreateReceipt, delete::DeleteReceipt,
            extend::ExtendReceipt, update::UpdateReceipt,
        },
    },
};

alloy::sol! {
    contract ArkivAbi {
        event EntityCreated(
            uint256 indexed entityKey,
            uint256 expirationBlock
        );

        event EntityUpdated(
            uint256 indexed entityKey,
            uint256 expirationBlock
        );

        event EntityDeleted(
            uint256 indexed entityKey
        );

        event EntityExtended(
            uint256 indexed entityKey,
            uint256 oldExpirationBlock,
            uint256 newExpirationBlock
        );
    }
}

/// Represents errors that can occur in the Arkiv ETH client.
/// Used for wrapping transaction, receipt, and log decoding errors.
#[derive(Debug, Display, Error)]
pub enum Error {
    /// Failed to send transaction: {0}
    TransactionSendError(String),
    /// Failed to get transaction receipt: {0}
    TransactionReceiptError(String),
    /// Unexpected log data: {0}
    UnexpectedLogDataError(String),
}

/// The Ethereum address of the Arkiv storage contract.
/// All entity-related transactions are sent to this address.
pub const STORAGE_ADDRESS: Address = address!("0x0000000000000000000000000000000060138453");

impl Client {
    pub async fn send_transaction(&self, tx: Transaction) -> Result<TransactionReceipt, Error> {
        self.send_raw_transaction(tx).await?.try_into()
    }

    /// Creates one or more new entities in Arkiv and returns their results.
    /// Sends a transaction to the storage contract and parses the resulting logs.
    pub async fn create_entities<Creates: Into<Vec<Create>>>(
        &self,
        creates: Creates,
    ) -> Result<Vec<CreateReceipt>, Error> {
        let result = self
            .send_transaction(Transaction::builder().creates(creates.into()).build())
            .await;

        result.and_then(|res| match res {
            TransactionReceipt {
                created,
                updated,
                deleted,
                extended,
            } if updated.is_empty() && deleted.is_empty() && extended.is_empty() => Ok(created),
            _ => Err(Error::UnexpectedLogDataError(
                "Unexpected content in tx logs, this should never happen!".to_string(),
            )),
        })
    }

    /// Updates one or more entities in Arkiv and returns their results.
    /// Sends a transaction to the storage contract and parses the resulting logs.
    pub async fn update_entities<Updates: Into<Vec<Update>>>(
        &self,
        updates: Updates,
    ) -> Result<Vec<UpdateReceipt>, Error> {
        let result = self
            .send_transaction(Transaction::builder().updates(updates.into()).build())
            .await;

        result.and_then(|res| match res {
            TransactionReceipt {
                created,
                updated,
                deleted,
                extended,
            } if created.is_empty() && deleted.is_empty() && extended.is_empty() => Ok(updated),
            _ => Err(Error::UnexpectedLogDataError(
                "Unexpected content in tx logs, this should never happen!".to_string(),
            )),
        })
    }

    /// Deletes one or more entities in Arkiv and returns their results.
    /// Sends a transaction to the storage contract and parses the resulting logs.
    pub async fn delete_entities<Deletes: Into<Vec<Delete>>>(
        &self,
        deletes: Deletes,
    ) -> Result<Vec<DeleteReceipt>, Error> {
        let result = self
            .send_transaction(Transaction::builder().deletes(deletes.into()).build())
            .await;

        result.and_then(|res| match res {
            TransactionReceipt {
                created,
                updated,
                deleted,
                extended,
            } if created.is_empty() && updated.is_empty() && extended.is_empty() => Ok(deleted),
            _ => Err(Error::UnexpectedLogDataError(
                "Unexpected content in tx logs, this should never happen!".to_string(),
            )),
        })
    }

    /// Extends the BTL (block time to live) of one or more entities and returns their results.
    /// Sends a transaction to the storage contract and parses the resulting logs for old and new expiration blocks.
    pub async fn extend_entities<Extensions: Into<Vec<Extend>>>(
        &self,
        extensions: Extensions,
    ) -> Result<Vec<ExtendReceipt>, Error> {
        let result = self
            .send_transaction(Transaction::builder().extensions(extensions.into()).build())
            .await;

        result.and_then(|res| match res {
            TransactionReceipt {
                created,
                updated,
                deleted,
                extended,
            } if created.is_empty() && updated.is_empty() && deleted.is_empty() => Ok(extended),
            _ => Err(Error::UnexpectedLogDataError(
                "Unexpected content in tx logs, this should never happen!".to_string(),
            )),
        })
    }

    /// NOTE: Nonce management is tricky!
    /// - This implementation always tries to fetch the latest on-chain nonce before sending a transaction,
    ///   and only falls back to the locally cached base_nonce if the sync fails.
    /// - Only the number of in-flight transactions is tracked locally.
    /// - For robust production use, consider also handling stuck transactions (e.g., gas bumping/EIP-1559).
    async fn next_nonce(&self) -> u64 {
        // This is sadly needed because `self.provider.get_transaction_count(self.wallet.address()).pending()`
        // doesn't give the right number...
        //
        //      Error: server returned an error response: error code -32000: replacement transaction underpriced
        let mut nm = self.nonce_manager.lock().await;
        let wallet_address = self.wallet.address();
        match self
            .ro_client
            .provider()
            .get_transaction_count(wallet_address)
            .await
        {
            Ok(on_chain_nonce) => {
                nm.base_nonce = on_chain_nonce;
            }
            Err(e) => {
                tracing::warn!("Failed to fetch on-chain nonce: {e}");
            }
        }
        nm.next_nonce().await
    }

    /// Sends a raw transaction to the Arkiv storage contract.
    /// Encodes the transaction payload and sends it to the contract address.
    pub async fn send_raw_transaction(
        &self,
        payload: Transaction,
    ) -> Result<RawTransactionReceipt, Error> {
        tracing::debug!("payload: {payload:?}");
        let encoded = payload.encoded();
        tracing::debug!("buffer: {encoded:?}");

        let nonce = self.next_nonce().await;

        let mut tx = TransactionRequest {
            to: Some(TxKind::Call(STORAGE_ADDRESS)),
            input: encoded.into(),
            chain_id: Some(
                self.ro_client
                    .provider()
                    .get_chain_id()
                    .await
                    .map_err(|e| Error::TransactionSendError(e.to_string()))?,
            ),
            nonce: Some(nonce),
            ..Default::default()
        };
        tracing::debug!("transaction: {tx:?}");

        let gas_limit = if let Some(gas_limit) = payload.gas_limit {
            gas_limit
        } else {
            self.ro_client
                .provider()
                .estimate_gas(tx.clone())
                .await
                .map_err(|e| Error::TransactionSendError(format!("Failed to estimate gas: {e}")))?
        };
        tx = tx.with_gas_limit(gas_limit);

        if let Some(max_fee_per_gas) = payload.max_fee_per_gas {
            tx = tx.with_max_fee_per_gas(max_fee_per_gas);
        }

        if let Some(max_priority_fee_per_gas) = payload.max_priority_fee_per_gas {
            tx = tx.with_max_priority_fee_per_gas(max_priority_fee_per_gas);
        }

        let pending_tx = self
            .ro_client
            .provider()
            .send_transaction(tx.clone())
            .await
            .map_err(|e| Error::TransactionSendError(e.to_string()))?;
        tracing::debug!("pending transaction: {pending_tx:?}");
        let receipt = pending_tx
            .get_receipt()
            .await
            .map_err(|e| Error::TransactionReceiptError(e.to_string()))?;
        tracing::debug!("receipt: {receipt:?}");
        {
            let mut nm = self.nonce_manager.lock().await;
            nm.complete().await;
        }

        if !receipt.status() {
            self.ro_client.provider().call(tx).await.map_err(|e| {
                Error::TransactionReceiptError(format!("Error during tx execution: {e}"))
            })?;
        }

        Ok(receipt)
    }
}
