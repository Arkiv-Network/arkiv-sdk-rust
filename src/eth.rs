use crate::entity::Hash;
use crate::entity::{
    Create, DeleteResult, EntityResult, Extend, ExtendResult, GolemBaseTransaction, Update,
};
use crate::GolemBaseClient;

use alloy::primitives::{address, Address, TxKind};
use alloy::providers::Provider;
use alloy::providers::ProviderBuilder;
use alloy::rpc::types::{Log, TransactionReceipt, TransactionRequest};
use alloy_rlp::Encodable;
use displaydoc::Display;
use thiserror::Error;

/// Represents errors that can occur in the GolemBase ETH client.
#[derive(Debug, Display, Error)]
pub enum Error {
    /// Failed to send transaction: {0}
    TransactionSendError(String),
    /// Failed to get transaction receipt: {0}
    TransactionReceiptError(String),
    /// Failed to decode expiration block: {0}
    ExpirationBlockDecodeError(String),
    /// Unexpected log data format
    UnexpectedLogDataError,
}

/// The Ethereum address of the GolemBase storage contract.
pub const STORAGE_ADDRESS: Address = address!("0x0000000000000000000000000000000060138453");

impl GolemBaseClient {
    /// Creates one or more new entities in GolemBase and returns their results.
    pub async fn create_entities(&self, creates: Vec<Create>) -> Result<Vec<EntityResult>, Error> {
        let receipt = self
            .create_raw_transaction(GolemBaseTransaction {
                creates,
                updates: vec![],
                deletes: vec![],
                extensions: vec![],
            })
            .await?;
        self.process_receipt(receipt, |log| {
            if log.topics().len() < 2 {
                return None;
            }
            let expiration_block = Self::parse_expiration_block(log.data().data.as_ref());
            Some(EntityResult {
                entity_key: log.topics()[1],
                expiration_block,
            })
        })
        .await
    }

    /// Updates one or more entities in GolemBase and returns their results.
    pub async fn update_entities(&self, updates: Vec<Update>) -> Result<Vec<EntityResult>, Error> {
        let receipt = self
            .create_raw_transaction(GolemBaseTransaction {
                creates: vec![],
                updates,
                deletes: vec![],
                extensions: vec![],
            })
            .await?;
        self.process_receipt(receipt, |log| {
            if log.topics().len() < 2 {
                return None;
            }
            let expiration_block = Self::parse_expiration_block(log.data().data.as_ref());
            Some(EntityResult {
                entity_key: log.topics()[1],
                expiration_block,
            })
        })
        .await
    }

    /// Deletes one or more entities in GolemBase and returns their results.
    pub async fn delete_entities(&self, deletes: Vec<Hash>) -> Result<Vec<DeleteResult>, Error> {
        let receipt = self
            .create_raw_transaction(GolemBaseTransaction {
                creates: vec![],
                updates: vec![],
                deletes,
                extensions: vec![],
            })
            .await?;
        self.process_receipt(receipt, |log| {
            if log.topics().len() < 2 {
                return None;
            }
            Some(DeleteResult {
                entity_key: log.topics()[1],
            })
        })
        .await
    }

    /// Extends the BTL of one or more entities in GolemBase and returns their results.
    pub async fn extend_entities(
        &self,
        extensions: Vec<Extend>,
    ) -> Result<Vec<ExtendResult>, Error> {
        let receipt = self
            .create_raw_transaction(GolemBaseTransaction {
                creates: vec![],
                updates: vec![],
                deletes: vec![],
                extensions,
            })
            .await?;
        self.process_receipt(receipt, |log| {
            let data = log.data().data.as_ref();
            if log.topics().len() < 2 {
                return None;
            }
            let old_expiration_block = Self::parse_expiration_block(&data[..8]);
            let new_expiration_block = Self::parse_expiration_block(&data[8..]);
            Some(ExtendResult {
                entity_key: log.topics()[1],
                old_expiration_block,
                new_expiration_block,
            })
        })
        .await
    }

    /// Creates and sends a raw transaction to the GolemBase storage contract.
    pub async fn create_raw_transaction(
        &self,
        payload: GolemBaseTransaction,
    ) -> Result<TransactionReceipt, Error> {
        log::debug!("payload: {:?}", payload);
        let mut buffer = Vec::new();
        payload.encode(&mut buffer);
        log::debug!("buffer: {:?}", buffer);
        let tx = TransactionRequest {
            to: Some(TxKind::Call(STORAGE_ADDRESS)),
            input: buffer.into(),
            chain_id: Some(
                self.provider
                    .get_chain_id()
                    .await
                    .map_err(|e| Error::TransactionSendError(e.to_string()))?,
            ),
            ..Default::default()
        };
        log::debug!("transaction: {:?}", tx);
        let provider = ProviderBuilder::new()
            .wallet(self.wallet.clone())
            .connect_http(self.rpc_url.clone());
        log::debug!("provider: {:?}", provider);
        let pending_tx = provider
            .send_transaction(tx)
            .await
            .map_err(|e| Error::TransactionSendError(e.to_string()))?;
        log::debug!("pending transaction: {:?}", pending_tx);
        let receipt = pending_tx
            .get_receipt()
            .await
            .map_err(|e| Error::TransactionReceiptError(e.to_string()))?;
        log::debug!("receipt: {:?}", receipt);
        Ok(receipt)
    }

    /// Processes a transaction receipt and maps logs into the desired result type.
    async fn process_receipt<T, F>(
        &self,
        receipt: TransactionReceipt,
        log_mapper: F,
    ) -> Result<Vec<T>, Error>
    where
        F: Fn(&Log) -> Option<T>,
    {
        let results: Vec<T> = receipt
            .logs()
            .iter()
            .filter(|log| log.address() == STORAGE_ADDRESS)
            .filter_map(log_mapper)
            .collect();
        Ok(results)
    }

    /// Parses a single `u64` value from log data, padding the beginning with zeros if needed.
    fn parse_expiration_block(data: &[u8]) -> u64 {
        let mut padded_data = [0u8; 8];
        let start = 8_usize.saturating_sub(data.len());
        padded_data[start..].copy_from_slice(&data[..data.len().min(8)]);
        u64::from_be_bytes(padded_data)
    }
}
