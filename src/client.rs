//! Module for Arkiv client functionality.
//! Exposes the main client interface for interacting with the Arkiv network.

use std::sync::Arc;

use alloy::{
    eips::BlockNumberOrTag,
    network::Network,
    network::TransactionBuilder,
    primitives::Address,
    providers::{DynProvider, Provider, ProviderBuilder},
    rpc::client::ClientRef,
    signers::local::PrivateKeySigner,
    transports::http::reqwest::Url,
};
use bigdecimal::BigDecimal;
use bon::bon;
use nonce::NonceManager;
use tokio::sync::Mutex;

use crate::utils::wei_to_eth;

mod nonce;
mod read_only;

#[derive(Debug, Clone, Copy)]
pub struct Arkiv {
    _private: (),
}
impl Network for Arkiv {
    type TxType = alloy::consensus::TxType;

    type TxEnvelope = alloy::consensus::TxEnvelope;

    type UnsignedTx = alloy::consensus::TypedTransaction;

    type ReceiptEnvelope = alloy::consensus::ReceiptEnvelope;

    type Header = alloy::consensus::Header;

    type TransactionRequest = alloy::rpc::types::eth::transaction::TransactionRequest;

    type TransactionResponse = alloy::rpc::types::eth::Transaction;

    type ReceiptResponse = alloy::rpc::types::eth::TransactionReceipt;

    type HeaderResponse = alloy::rpc::types::eth::Header;

    type BlockResponse = alloy::rpc::types::eth::Block;
}
pub struct ArkivProvider<N: Network = Arkiv>(Arc<dyn Provider<N> + 'static>);

/// A read-only wrapper around [`DynProvider`] which provides methods
/// for interacting with the Arkiv Network.
#[derive(Clone)]
pub struct RoClient(DynProvider);

#[bon]
impl RoClient {
    /// Creates a new builder for `arkiv_sdk::RoClient` with the given wallet and RPC URL.
    /// Initializes the provider and sets up default configuration.
    #[builder]
    pub fn builder(rpc_url: Url, provider: Option<DynProvider>) -> Self {
        let provider =
            provider.unwrap_or_else(|| ProviderBuilder::new().connect_http(rpc_url).erased());

        Self(provider)
    }

    pub(crate) fn provider(&self) -> &DynProvider {
        &self.0
    }
}

/// A client for interacting with the Arkiv system.
/// Provides methods for account management, entity operations, balance queries, and event subscriptions.
///
/// # Example Usage
///
/// A client builder is provided for both [`ArkivClient`] and [`ArkivRoClient`],
/// however, an instance of [`ArkivClient`] can be dereferenced to [`ArkivRoClient`] like so:
///
/// ```rs
/// use arkiv_sdk::{Client, RoClient, PrivateKeySigner, Url};
///
/// let keypath = dirs::config_dir()
///     .ok_or("Failed to get config directory")?
///     .join("golembase")
///     .join("wallet.json");
/// let signer = PrivateKeySigner::decrypt_keystore(keypath, "password")?;
/// let url = Url::parse("http://localhost:8545")?;
///
/// let client = Client::builder()
///     .wallet(signer)
///     .rpc_url(url)
///     .build();
///
/// let ro_client: &RoClient = client.read_only();
/// ```
#[derive(Clone)]
pub struct Client {
    /// A read-only wrapped [`DynProvider`] arkiv client.
    pub(crate) ro_client: RoClient,
    /// The Ethereum address of the client owner.
    pub(crate) wallet: PrivateKeySigner,
    /// Nonce manager for tracking transaction nonces.
    pub(crate) nonce_manager: Arc<Mutex<NonceManager>>,
}

#[bon]
impl Client {
    /// Creates a new builder for [`arkive_sdk::Client`] with the given wallet and RPC URL.
    /// Initializes the provider and sets up default configuration.
    #[builder]
    pub fn builder(wallet: PrivateKeySigner, rpc_url: Url) -> Self {
        let ro_client = RoClient::builder()
            .rpc_url(rpc_url.clone())
            .provider(
                ProviderBuilder::new()
                    .wallet(wallet.clone())
                    .connect_http(rpc_url)
                    .erased(),
            )
            .build();

        Self {
            ro_client,
            wallet,
            nonce_manager: Arc::new(Mutex::new(NonceManager {
                base_nonce: 0,
                in_flight: 0,
            })),
        }
    }

    /// Returns a reference to a read-only client.
    pub fn read_only(&self) -> &RoClient {
        &self.ro_client
    }

    /// Returns a reference to the underlying JSON-RPC client.
    pub fn rpc_client(&self) -> ClientRef<'_> {
        self.read_only().provider().client()
    }

    /// The Ethereum address of the client owner.
    pub fn owner_address(&self) -> Address {
        self.wallet.address()
    }

    /// Gets the chain ID from the provider.
    /// Returns the chain ID as a `u64`.
    pub async fn get_chain_id(&self) -> anyhow::Result<u64> {
        self.read_only()
            .provider()
            .get_chain_id()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get chain ID: {e}"))
    }

    /// Gets an account's ETH balance as a `BigDecimal`.
    pub async fn get_balance(&self, account: Address) -> anyhow::Result<BigDecimal> {
        let balance = self.read_only().provider().get_balance(account).await?;
        Ok(wei_to_eth(balance))
    }

    /// Gets the current block number from the chain.
    /// Returns the latest block number as a `u64`.
    pub async fn get_current_block_number(&self) -> anyhow::Result<u64> {
        let latest_block = self
            .read_only()
            .provider()
            .get_block_by_number(BlockNumberOrTag::Latest)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Failed to get latest block"))?;
        Ok(latest_block.header.number)
    }
}
