//! Module for Ethereum transaction-related functionality.
//! Provides helpers for constructing, signing, and sending Ethereum transactions.

use alloy::primitives::{Address, address};
use displaydoc::Display;
use thiserror::Error;

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
