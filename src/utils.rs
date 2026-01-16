//! Module with utility functions.
//! Includes helpers for encoding, decoding, and other common tasks.

use std::str::FromStr;

use alloy::{
    network::TransactionBuilder,
    primitives::{Address, U256},
    providers::{DynProvider, PendingTransactionError, Provider, ProviderBuilder},
    rpc::types::TransactionReceipt,
    signers::{Signer, local::PrivateKeySigner},
};
use bigdecimal::{BigDecimal, ToPrimitive};

async fn transfer_funds(
    provider: &DynProvider,
    from: Address,
    to: Address,
    amount: U256,
    retries: usize,
) -> alloy::rpc::types::TransactionReceipt {
    let tx = provider
        .transaction_request()
        .with_chain_id(provider.get_chain_id().await.unwrap())
        .from(from)
        .max_priority_fee_per_gas(1_000_000_000) // 1 gwei
        .max_fee_per_gas(5_000_000_000) // 5 gwei
        .gas_limit(2_800_000)
        .to(to)
        .value(amount);

    let mut res: Result<TransactionReceipt, PendingTransactionError> =
        Err(PendingTransactionError::FailedToRegister);
    for attempt in 0..=retries {
        let pending_tx = provider
            .send_transaction(tx.clone())
            .await
            .expect("failed to send transaction");
        match pending_tx.get_receipt().await {
            Ok(receipt) => {
                res = Ok(receipt);
                break;
            }
            Err(err) if attempt == retries => {
                res = Err(err);
                break;
            }
            Err(_) => std::thread::sleep(std::time::Duration::from_secs(1)),
        }
    }
    res.expect("failed to get receipt")
}

/// Only useful for local development and testing. Funds an account with some eth.
/// The provider must not have a wallet linked to it, otherwise the provider will
/// attempt to look for the signing credential of account index 0.
pub async fn fund_account(
    node_provider: &DynProvider,
    address: Address,
    amount: U256,
) -> alloy::rpc::types::TransactionReceipt {
    let from = node_provider
        .get_accounts()
        .await
        .expect("failed to get accounts")
        .first()
        .expect("no accounts available on node")
        .to_owned();
    transfer_funds(node_provider, from, address, amount, 5).await
}

/// Generate the minimum required fee history for transactions to succeed. Use this when connecting to a dev node that has
/// no history to pull gas information from. This can be useful if you are running into `receipts not found` errors.
pub async fn generate_fee_history(node_provider: &DynProvider, http_endpoint: reqwest::Url) {
    const MIN_FEE_HISTORY: std::ops::Range<i32> = 0..5;
    let chain_id = node_provider
        .get_chain_id()
        .await
        .expect("failed to get chain id from node provider");
    let signer_pair = || async move {
        let random_signer = || PrivateKeySigner::random().with_chain_id(Some(chain_id));
        let alice = random_signer();
        let bob = random_signer();
        fund_account(
            node_provider,
            alice.address(),
            U256::from_limbs([0, 100, 0, 0]),
        )
        .await;
        (alice, bob)
    };
    for _ in MIN_FEE_HISTORY {
        let (alice, bob) = signer_pair().await;
        let wallet_provider = ProviderBuilder::new()
            .with_chain_id(chain_id)
            .wallet(alice.clone())
            .connect_http(http_endpoint.clone())
            .erased();

        transfer_funds(
            &wallet_provider,
            alice.address(),
            bob.address(),
            U256::from_limbs([50, 0, 0, 0]),
            5,
        )
        .await;
    }
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
