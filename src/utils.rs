//! Module with utility functions.
//! Includes helpers for encoding, decoding, and other common tasks.

use std::str::FromStr;

use alloy::{
    network::TransactionBuilder,
    primitives::{Address, U256},
    providers::{DynProvider, Provider},
};
use bigdecimal::{BigDecimal, ToPrimitive};

/// Only useful for local development and testing. Funds an account with some eth.
/// The provider must not have a wallet linked to it, otherwise the provider will
/// attempt to look for the signing credential of account index 0.
pub async fn fund_account(
    provider: &DynProvider,
    address: Address,
    amount: U256,
) -> alloy::rpc::types::TransactionReceipt {
    let from = provider
        .get_accounts()
        .await
        .unwrap()
        .get(0)
        .expect("no accounts available on node")
        .to_owned();
    let tx = provider
        .transaction_request()
        .with_chain_id(provider.get_chain_id().await.unwrap())
        .from(from)
        .max_priority_fee_per_gas(1_000_000_000) // 1 gwei
        .max_fee_per_gas(5_000_000_000) // 5 gwei
        .gas_limit(2_800_000)
        .to(address)
        .value(amount);

    provider
        .send_transaction(tx)
        .await
        .unwrap()
        .get_receipt()
        .await
        .unwrap()
}

/// Converts an ETH amount to wei as a `U256`.
/// Accepts a `BigDecimal` ETH value and returns the equivalent amount in wei as a `U256`.
/// This is useful for preparing values for smart contract calls or transactions.
/// Returns an error if the value is too large to fit in a `u128`.
pub fn eth_to_wei(eth: BigDecimal) -> anyhow::Result<U256> {
    let wei = (eth * BigDecimal::from(1_000_000_000_000_000_000u128))
        .to_u128()
        .ok_or_else(|| anyhow::anyhow!("Value too large"))?;
    Ok(U256::from(wei))
}

/// Converts a wei amount (`U256`) to ETH as a `BigDecimal`.
/// Useful for displaying human-readable ETH values from raw wei amounts, such as for UI or logs.
/// Panics if the `U256` value cannot be parsed as a string (should not happen for valid values).
pub fn wei_to_eth(wei: U256) -> BigDecimal {
    BigDecimal::from_str(&wei.to_string()).unwrap()
        / BigDecimal::from(1_000_000_000_000_000_000u128)
}
